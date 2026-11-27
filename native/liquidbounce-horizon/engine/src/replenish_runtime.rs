// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{inventory::InventoryActor, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    inventory_manager::{Action, Context, Input, MenuState, Outcome, OwnerState, Request, Slot},
    replenish::{Item, Replenish, Step},
};
use serde_json::Value;
pub const OWNER: u64 = 0x5245_504c_454e_4953;
#[derive(Default)]
pub struct ReplenishRuntime {
    pub settings: Replenish,
    pending: Option<u64>,
    expected: Option<(MenuState, usize)>,
    waiting: bool,
    callback: u64,
}
fn items(context: &Context) -> Vec<Item> {
    context
        .player_stacks
        .iter()
        .map(|s| {
            let v = serde_json::from_str::<Value>(&s.components).unwrap_or(Value::Null);
            Item {
                components: s.components.clone(),
                count: s.count,
                maximum: v[1]["minecraft:max_stack_size"].as_i64().unwrap_or(1) as i32,
            }
        })
        .collect()
}
impl ReplenishRuntime {
    pub fn running(&self, parent: bool, c: &Context) -> bool {
        self.settings
            .running(parent, c.screen.is_some(), c.inventory_screen)
    }
    pub fn owner(&self, parent: bool, c: &Context) -> (u64, OwnerState) {
        (
            OWNER,
            OwnerState {
                running: self.running(parent, c),
                guards_pass: true,
                constraints: self.settings.constraints.clone(),
            },
        )
    }
    pub fn cancel(&mut self, inventory: &mut InventoryActor) {
        inventory.manager.cancel_pending(OWNER);
        if let Some(id) = self.pending.take() {
            inventory.manager.end_plan(id);
        }
        self.settings.cancel();
        self.expected = None;
        self.waiting = false;
        self.callback = self.callback.wrapping_add(1);
    }
    pub fn world_change(&mut self) {
        self.pending = None;
        self.expected = None;
        self.waiting = false;
        self.callback = self.callback.wrapping_add(1);
        self.settings.world_change();
    }
    fn drive(&mut self, inventory: &mut InventoryActor, c: &Context) -> Option<u64> {
        let id = self.pending?;
        match self.settings.next(&items(c), c.menu.carried.empty(), c.now) {
            Step::Chain(clicks) => {
                let actions = clicks
                    .into_iter()
                    .map(|click| Action::Click {
                        screen: None,
                        slot: Slot::Player(click.slot),
                        button: click.button,
                        input: match click.input {
                            liquidbounce_behaviors::replenish::Input::Pickup => Input::Pickup,
                            liquidbounce_behaviors::replenish::Input::PickupAll => Input::PickupAll,
                            liquidbounce_behaviors::replenish::Input::Swap => Input::Swap,
                        },
                    })
                    .collect();
                inventory.manager.submit(Request::new(
                    id,
                    OWNER,
                    0,
                    actions,
                    c,
                    &inventory.manager,
                ));
                None
            }
            Step::Wait => {
                if self.waiting {
                    return None;
                }
                self.waiting = true;
                self.callback = self.callback.wrapping_add(1);
                Some(self.callback)
            }
            Step::Done => {
                self.pending = None;
                self.expected = None;
                inventory.manager.end_plan(id);
                None
            }
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn replenish_screen(&self, payload: &str) {
        let v = serde_json::from_str::<Value>(payload).unwrap_or(Value::Null);
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        if b.replenish.settings.running(
            b.query_running(&state.registry, "Replenish"),
            v["abstract"] == true,
            v["inventory"] == true,
        ) {
            b.replenish.settings.clear_tracked();
        }
    }
    pub(crate) fn replenish_plan(&self, c: &Context) {
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let b = &mut *b;
        if !b
            .replenish
            .running(b.query_running(&state.registry, "Replenish"), c)
            || b.replenish.pending.is_some()
            || !b.replenish.settings.begin(c.now)
        {
            return;
        }
        b.replenish.pending = Some(b.inventory.id());
        if let Some(callback) = b.replenish.drive(&mut b.inventory, c) {
            b.coroutine_ticker
                .register(crate::coroutine_ticker::WaitOwner::Replenish, callback);
        }
    }
    pub(crate) fn replenish_outcome(&self, outcome: &Outcome, c: &Context) {
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let b = &mut *b;
        match outcome {
            Outcome::Completed { request } if b.replenish.pending == Some(*request) => {
                b.replenish.expected = Some((c.menu.clone(), b.inventory.manager.locked_count()));
                if let Some(callback) = b.replenish.drive(&mut b.inventory, c) {
                    b.coroutine_ticker
                        .register(crate::coroutine_ticker::WaitOwner::Replenish, callback);
                }
            }
            Outcome::Cancelled { request } if b.replenish.pending == Some(*request) => {
                b.replenish.cancel(&mut b.inventory)
            }
            _ => {}
        }
    }
    pub(crate) fn replenish_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        callback: u64,
    ) -> bool {
        let c = crate::inventory::Snapshot::parse(&host.client_inventory_snapshot(), false, false)
            .ok()
            .flatten()
            .map(|s| s.context);
        let keep = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let b = &mut *b;
            if callback != b.replenish.callback || !b.replenish.waiting {
                return false;
            }
            let valid = in_game
                && c.as_ref().is_some_and(|c| {
                    b.replenish
                        .running(b.query_running(&state.registry, "Replenish"), c)
                        && b.replenish.expected.as_ref().is_none_or(|(menu, locked)| {
                            *locked == b.inventory.manager.locked_count() && menu.matches(&c.menu)
                        })
                });
            if !valid {
                b.replenish.cancel(&mut b.inventory);
                return false;
            }
            let c = c.as_ref().unwrap();
            if !b.replenish.settings.ready(c.now) {
                return true;
            }
            b.replenish.waiting = false;
            if let Some(next) = b.replenish.drive(&mut b.inventory, c) {
                b.coroutine_ticker
                    .register(crate::coroutine_ticker::WaitOwner::Replenish, next);
            }
            false
        };
        self.inventory_frame(host);
        keep
    }
}
