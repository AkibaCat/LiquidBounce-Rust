// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleElytraSwap at LiquidBounce ae0eb396. The shared InventoryManager owns
//! the atomic pickup chain, live constraints, suspension, and completion tail.

use crate::vomit::Constraints;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Item {
    pub empty: bool,
    pub glider: bool,
    pub next_damage_will_break: bool,
    /// Actual ItemTags.CHEST_ARMOR membership, independent of EQUIPPABLE.
    pub chest_armor: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ElytraSwap {
    pub constraints: Constraints,
    /// Captured once per enabling, retained when a suspended plan is cancelled.
    pub wants_glider: Option<bool>,
}

impl ElytraSwap {
    pub fn on_enabled(&mut self) {
        self.wants_glider = None;
    }

    /// Slots are queried lazily in the original Hotbar + Inventory + OFFHAND
    /// order. An empty chain reaches enabled=false immediately; a nonempty
    /// chain reaches that tail only after InventoryManager completes perform.
    pub fn pickups(&mut self, chest: Item, mut stack: impl FnMut(u8) -> Item) -> Vec<u8> {
        let wants = *self.wants_glider.get_or_insert(!chest.glider);
        if chest.glider == wants {
            return Vec::new();
        }
        let source = (0..36).chain(std::iter::once(40)).find(|slot| {
            let item = stack(*slot);
            if wants {
                item.glider && !item.next_damage_will_break
            } else {
                item.chest_armor
            }
        });
        let Some(source) = source else {
            return Vec::new();
        };
        let mut actions = vec![source, 38];
        if !chest.empty {
            actions.push(source);
        }
        actions
    }
}
