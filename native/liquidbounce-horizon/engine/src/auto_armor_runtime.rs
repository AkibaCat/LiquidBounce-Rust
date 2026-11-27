// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    inventory::InventoryActor, inventory_cleaner_runtime::ItemSnapshot, runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_armor::{AutoArmor, Piece, SaveState, ScreenAction},
    inventory_cleaner::Item,
    inventory_manager::{Context, Outcome, OwnerState, Request, Slot},
    random::with_thread_random,
};
use std::collections::{BTreeSet, VecDeque};
pub const OWNER: u64 = 0x4155_544f_4152_4d52;
#[derive(Default)]
pub struct AutoArmorRuntime {
    pub settings: AutoArmor,
    pub pending: Option<u64>,
    pieces: VecDeque<Piece>,
    active: Option<(Piece, bool)>,
    pub save: SaveState,
    callback: u64,
    waiter: bool,
}
fn locked(items: &[Item], inventory: &InventoryActor, c: &Context) -> BTreeSet<Slot> {
    items
        .iter()
        .filter(|s| match s.slot {
            Slot::Player(i) => c
                .player_stacks
                .get(usize::from(i))
                .is_some_and(|s| inventory.manager.is_locked(s)),
            _ => false,
        })
        .map(|s| s.slot)
        .collect()
}
impl AutoArmorRuntime {
    pub fn clear(&mut self) {
        self.pending = None;
        self.pieces.clear();
        self.active = None;
        self.save.reset();
        self.waiter = false;
        self.callback = self.callback.wrapping_add(1);
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
    fn drive(&mut self, inventory: &mut InventoryActor, c: &Context, items: &[Item]) {
        let Some(id) = self.pending else {
            return;
        };
        let locks = locked(items, inventory, c);
        if let Some((piece, was_worn)) = self.active.take() {
            if was_worn && !items.iter().any(|s| s.slot == piece.target() && !s.empty()) {
                self.pieces.push_front(piece);
            }
        }
        while let Some(piece) = self.pieces.pop_front() {
            let piece = Piece {
                source: inventory.manager.slot_of(id, piece.source),
                ..piece
            };
            let is_worn = items.iter().any(|s| s.slot == piece.target() && !s.empty());
            let Some(action) = self.settings.action(
                piece,
                c,
                items,
                &locks,
                c.inventory_screen || inventory.manager.inventory_open_server_side,
            ) else {
                continue;
            };
            // Only a removal is followed by one equip action. The repeated
            // piece's target is empty, so its second completion does not repeat.
            self.active = Some((piece, is_worn));
            inventory.manager.submit(Request::new(
                id,
                OWNER,
                40,
                vec![action],
                c,
                &inventory.manager,
            ));
            return;
        }
        self.pending = None;
        inventory.manager.end_plan(id);
    }
}
impl LiquidBounceHooks {
    fn auto_armor_items(&self, host: &mut dyn Host) -> Option<Vec<Item>> {
        let policy = self
            .state()
            .behaviors
            .borrow()
            .item_policy
            .block_selection
            .to_string();
        Some(
            host.client_inventory_cleaner_context(&policy)
                .and_then(|s| ItemSnapshot::parse(&s))?
                .items
                .into_iter()
                .filter(|s| matches!(s.slot, Slot::Player(_)))
                .collect(),
        )
    }
    pub(crate) fn auto_armor_plan(&self, host: &mut dyn Host, c: &Context) {
        let running = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            b.item_policy.armor = b.auto_armor.settings.policy();
            b.query_running(&state.registry, "AutoArmor")
        };
        if !running {
            return;
        }
        let Some(facts) = host
            .client_auto_armor_context()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        else {
            return;
        };
        if facts["spectator"] == true {
            return;
        }
        let Some(items) = self.auto_armor_items(host) else {
            return;
        };
        let effect = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let b = &mut *b;
            let locks = locked(&items, &b.inventory, c);
            let replacement = b.auto_armor.settings.pieces(&items, &locks, true);
            let range = b.auto_armor.settings.constraints.close_delay;
            let effect = b.auto_armor.save.tick(
                &b.auto_armor.settings,
                c,
                false,
                facts["armor"].as_i64().unwrap_or(0) as i32,
                &items,
                &replacement,
                || with_thread_random(|r| r.next_i32_inclusive(range[0], range[1])),
            );
            if b.auto_armor.save.waiting.is_some() && !b.auto_armor.waiter {
                b.auto_armor.waiter = true;
                b.auto_armor.callback = b.auto_armor.callback.wrapping_add(1);
                b.coroutine_ticker.register(
                    crate::coroutine_ticker::WaitOwner::AutoArmor,
                    b.auto_armor.callback,
                );
            }
            if b.auto_armor.pending.is_none() {
                let pieces = b.auto_armor.settings.pieces(&items, &locks, false);
                if !pieces.is_empty() {
                    b.auto_armor.pending = Some(b.inventory.id());
                    b.auto_armor.pieces = pieces.into();
                    b.auto_armor.drive(&mut b.inventory, c, &items);
                }
            }
            effect
        };
        if let Some(effect) = effect {
            host.client_auto_armor_screen(effect == ScreenAction::Open);
        }
    }
    pub(crate) fn auto_armor_outcome(&self, host: &mut dyn Host, outcome: &Outcome, c: &Context) {
        let pending = self.state().behaviors.borrow().auto_armor.pending;
        match outcome {
            Outcome::Completed { request } if pending == Some(*request) => {
                let Some(items) = self.auto_armor_items(host) else {
                    return;
                };
                let state = self.state();
                let mut b = state.behaviors.borrow_mut();
                let b = &mut *b;
                b.auto_armor.drive(&mut b.inventory, c, &items);
            }
            Outcome::Cancelled { request } if pending == Some(*request) => {
                let state = self.state();
                let mut b = state.behaviors.borrow_mut();
                b.inventory.manager.end_plan(*request);
                b.auto_armor.pending = None;
                b.auto_armor.pieces.clear();
                b.auto_armor.active = None;
            }
            _ => {}
        }
    }
    pub(crate) fn auto_armor_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        callback: u64,
    ) -> bool {
        let context =
            crate::inventory::Snapshot::parse(&host.client_inventory_snapshot(), false, false)
                .ok()
                .flatten()
                .map(|s| s.context);
        let (effect, keep) = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let running = b.query_running(&state.registry, "AutoArmor");
            if callback != b.auto_armor.callback || !b.auto_armor.waiter {
                return false;
            }
            if !in_game || !running || context.is_none() {
                b.auto_armor.save.waiting = None;
                b.auto_armor.waiter = false;
                return false;
            }
            let result = b.auto_armor.save.resume(context.as_ref().unwrap());
            b.auto_armor.waiter = result.1;
            result
        };
        if let Some(effect) = effect {
            host.client_auto_armor_screen(effect == ScreenAction::Open);
        }
        keep
    }
}
