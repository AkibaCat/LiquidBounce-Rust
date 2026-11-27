// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{inventory::InventoryActor, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    chest_cleaner::ChestCleaner,
    inventory_manager::{Action, Context, Input, Outcome, OwnerState, Request, Slot},
};
use serde_json::Value;
use std::collections::VecDeque;
pub const OWNER: u64 = 0x4348_4553_5443_4c4e;
#[derive(Default)]
pub struct ChestCleanerRuntime {
    pub settings: ChestCleaner,
    pub pending: Option<u64>,
    actions: VecDeque<Action>,
}
impl ChestCleanerRuntime {
    pub fn clear(&mut self) {
        self.pending = None;
        self.actions.clear();
    }
    pub fn owner(&self, running: bool) -> (u64, OwnerState) {
        (
            OWNER,
            OwnerState {
                running,
                guards_pass: true,
                constraints: self.settings.constraints.clone(),
            },
        )
    }
    fn submit_next(&mut self, inventory: &mut InventoryActor, context: &Context) {
        let Some(id) = self.pending else {
            return;
        };
        if let Some(action) = self.actions.pop_front() {
            inventory.manager.submit(Request::new(
                id,
                OWNER,
                0,
                vec![action],
                context,
                &inventory.manager,
            ));
        } else {
            self.pending = None;
            inventory.manager.end_plan(id);
        }
    }
    pub fn outcome(
        &mut self,
        inventory: &mut InventoryActor,
        outcome: &Outcome,
        context: &Context,
    ) {
        match *outcome {
            Outcome::Completed { request } if self.pending == Some(request) => {
                self.submit_next(inventory, context)
            }
            Outcome::Cancelled { request } if self.pending == Some(request) => {
                self.clear();
                inventory.manager.end_plan(request);
            }
            _ => {}
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn chest_cleaner_plan(&self, host: &dyn Host, context: &Context) {
        let Some(screen) = context.screen.as_ref() else {
            return;
        };
        {
            let state = self.state();
            let b = state.behaviors.borrow();
            if !b.query_running(&state.registry, "ChestCleaner")
                || b.chest_cleaner.pending.is_some()
            {
                return;
            }
        }
        let Some(payload) = host.client_chest_cleaner_context() else {
            return;
        };
        let Ok(v) = serde_json::from_str::<Value>(&payload) else {
            return;
        };
        let translated: Vec<Vec<String>> =
            serde_json::from_value(v["translated"].clone()).unwrap_or_default();
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let b = &mut *b;
        let settings = &b.chest_cleaner.settings;
        if !settings.accepts_screen(
            context.inventory_screen,
            v["menu"].as_str(),
            v["title"].as_str().unwrap_or_default(),
            &translated,
        ) {
            return;
        }
        let mut actions = VecDeque::new();
        for &slot in &screen.non_player_slots {
            let Some(stack) = usize::try_from(slot)
                .ok()
                .and_then(|slot| context.menu.stacks.get(slot))
            else {
                continue;
            };
            let item = serde_json::from_str::<Value>(&stack.components)
                .ok()
                .and_then(|v| v[0].as_str().map(str::to_owned))
                .unwrap_or_default();
            if settings.selects(&item, stack.empty()) {
                actions.push_back(Action::Click {
                    screen: Some(screen.clone()),
                    slot: Slot::Container(slot),
                    button: 1,
                    input: Input::Throw,
                });
            }
        }
        if settings.auto_close {
            actions.push_back(Action::CloseScreen {
                container_id: screen.container_id,
            });
        }
        if actions.is_empty() {
            return;
        }
        b.chest_cleaner.pending = Some(b.inventory.id());
        b.chest_cleaner.actions = actions;
        b.chest_cleaner.submit_next(&mut b.inventory, context);
    }
}
