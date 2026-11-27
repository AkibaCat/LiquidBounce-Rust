// SPDX-License-Identifier: GPL-3.0-or-later
//! Unabridged ModuleVomit planning at LiquidBounce ae0eb396.
//! InventoryManager owns execution, live constraints, waits and expected-menu
//! guards. Producing a Plan alone does not implement or register this module.

use crate::random::ThreadRandom;
use std::hash::{BuildHasher, RandomState};

pub const REQUIREMENTS: [&str; 6] = [
    "NoMovement",
    "NoRotation",
    "NotUsingItem",
    "NotBreaking",
    "NotDuringCombat",
    "InventoryOpen",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Constraints {
    pub start_delay: [i32; 2],
    pub click_delay: [i32; 2],
    pub close_delay: [i32; 2],
    pub miss_chance: [i32; 2],
    pub slot_multiplier: f32,
    pub millisecond_delays: bool,
    pub requirements: u8,
}
impl Default for Constraints {
    fn default() -> Self {
        Self {
            start_delay: [50, 100],
            click_delay: [100, 200],
            close_delay: [50, 100],
            miss_chance: [0, 0],
            slot_multiplier: 0.2,
            millisecond_delays: true,
            requirements: 0,
        }
    }
}
impl Constraints {
    pub fn travel_millis(&self, delay: [i32; 2], slots: f32, random: &mut ThreadRandom) -> i32 {
        // JVM float->int has saturating/NaN semantics, as Rust's `as i32` does.
        (random.next_i32_inclusive(delay[0], delay[1]) as f32 * slots * self.slot_multiplier) as i32
    }
}

/// RandomSource.create(seed) -> LegacyRandomSource: an independent Java 48-bit
/// stream, never Kotlin's thread-local stream used for choosing storage slots.
#[derive(Clone, Copy, Debug)]
pub struct LegacyRandom {
    seed: u64,
}
impl LegacyRandom {
    const MASK: u64 = (1 << 48) - 1;
    pub fn seeded(seed: u64) -> Self {
        Self {
            seed: (seed ^ 0x5deece66d) & Self::MASK,
        }
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(0x5deece66d).wrapping_add(11) & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }
    pub fn next_int(&mut self, bound: i32) -> Result<i32, &'static str> {
        if bound <= 0 {
            return Err("LegacyRandomSource bound must be positive");
        }
        if bound & (bound - 1) == 0 {
            return Ok(((i64::from(bound) * i64::from(self.next(31))) >> 31) as i32);
        }
        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return Ok(value);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// Real player inventory index: hotbar 0..8; storage 9..35.
    Player(u8),
    /// Slot.index in the actual ContainerScreen menu.
    Container(i32),
}
impl Slot {
    pub fn server_id(self, menu_slot_count: Option<i32>) -> i32 {
        match (self, menu_slot_count) {
            (Self::Container(index), _) => index,
            (Self::Player(index), None) if index < 9 => 36 + i32::from(index),
            (Self::Player(index), None) => i32::from(index),
            (Self::Player(index), Some(count)) if index < 9 => count - 9 + i32::from(index),
            (Self::Player(index), Some(count)) => count - 36 + i32::from(index) - 9,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Container {
    pub sync_id: i32,
    pub slot_count: i32,
    /// Nonempty slots whose Slot.container !== player.inventory, in menu order.
    pub nonempty_slots: Vec<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub creative: bool,
    /// Only storage and hotbar. Armor and offhand are deliberately excluded.
    pub player_nonempty: [bool; 36],
    /// Only the source's ContainerScreen subtype, not any generic menu screen.
    pub container: Option<Container>,
    pub block_registry_size: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    CreativeThrow {
        block_index: i32,
        count: i32,
    },
    CreativeFill {
        block_index: i32,
        count: i32,
        slot: Slot,
    },
    Throw {
        screen: Option<(i32, i32)>,
        slot: Slot,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// `requires { player.isCreative == isCreative }`, live at every suspension.
    pub creative: bool,
    /// One non-interruptible InventoryAction.Chain in original order.
    pub actions: Vec<Action>,
}
impl Plan {
    pub fn guard(&self, running: bool, creative: bool) -> bool {
        running && creative == self.creative
    }
}

#[derive(Clone, Debug)]
pub struct Vomit {
    pub constraints: Constraints,
    pub block_random: LegacyRandom,
}
impl Default for Vomit {
    fn default() -> Self {
        Self {
            constraints: Constraints::default(),
            block_random: LegacyRandom::seeded(RandomState::new().hash_one("ModuleVomit")),
        }
    }
}
impl Vomit {
    pub fn plan(
        &mut self,
        snapshot: &Snapshot,
        random: &mut ThreadRandom,
    ) -> Result<Option<Plan>, &'static str> {
        let slots = |nonempty| {
            (9u8..36)
                .chain(0..9)
                .filter(|index| snapshot.player_nonempty[usize::from(*index)] == nonempty)
                .map(Slot::Player)
                .collect::<Vec<_>>()
        };
        let actions = if snapshot.creative {
            // This random draw precedes empty-slot detection, including a full inventory.
            let block_index = self.block_random.next_int(snapshot.block_registry_size)?;
            let empty = slots(false);
            if empty.is_empty() {
                vec![Action::CreativeThrow {
                    block_index,
                    count: 64,
                }]
            } else if self.constraints.click_delay[1] <= 0 {
                empty
                    .iter()
                    .map(|slot| Action::CreativeFill {
                        block_index,
                        count: 64,
                        slot: *slot,
                    })
                    .chain(empty.iter().map(|slot| Action::Throw {
                        screen: None,
                        slot: *slot,
                    }))
                    .collect()
            } else {
                let slot = empty[random.next_i32_bound(empty.len() as i32) as usize];
                vec![
                    Action::CreativeFill {
                        block_index,
                        count: 64,
                        slot,
                    },
                    Action::Throw { screen: None, slot },
                ]
            }
        } else {
            let candidates = slots(true);
            let slot = if candidates.is_empty() {
                let Some(container) = &snapshot.container else {
                    return Ok(None);
                };
                if container.nonempty_slots.is_empty() {
                    return Ok(None);
                }
                Slot::Container(
                    container.nonempty_slots
                        [random.next_i32_bound(container.nonempty_slots.len() as i32) as usize],
                )
            } else {
                candidates[random.next_i32_bound(candidates.len() as i32) as usize]
            };
            vec![Action::Throw {
                screen: snapshot
                    .container
                    .as_ref()
                    .map(|c| (c.sync_id, c.slot_count)),
                slot,
            }]
        };
        Ok(Some(Plan {
            creative: snapshot.creative,
            actions,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_fill_then_full_throw_and_one_shared_stack_choice() {
        let mut module = Vomit::default();
        module.constraints.click_delay = [99, 0];
        let snapshot = Snapshot {
            creative: true,
            player_nonempty: [false; 36],
            container: None,
            block_registry_size: 100,
        };
        let mut random = ThreadRandom::seeded(123, 1);
        let mut untouched = random;
        let plan = module.plan(&snapshot, &mut random).unwrap().unwrap();
        assert_eq!(plan.actions.len(), 72);
        assert!(matches!(
            plan.actions[0],
            Action::CreativeFill {
                slot: Slot::Player(9),
                ..
            }
        ));
        assert!(matches!(
            plan.actions[35],
            Action::CreativeFill {
                slot: Slot::Player(8),
                ..
            }
        ));
        assert_eq!(
            plan.actions[36],
            Action::Throw {
                screen: None,
                slot: Slot::Player(9)
            }
        );
        assert_eq!(random.next_i32(), untouched.next_i32());
    }
    #[test]
    fn source_player_menu_slot_mapping_and_creative_guard() {
        for index in 0u8..36 {
            let slot = Slot::Player(index);
            assert_eq!(
                slot.server_id(None),
                if index < 9 {
                    36 + i32::from(index)
                } else {
                    i32::from(index)
                }
            );
            assert_eq!(
                slot.server_id(Some(90)),
                if index < 9 {
                    81 + i32::from(index)
                } else {
                    45 + i32::from(index)
                }
            );
        }
        let plan = Plan {
            creative: true,
            actions: vec![],
        };
        assert!(plan.guard(true, true));
        assert!(!plan.guard(true, false));
        assert!(!plan.guard(false, true));
    }
    #[test]
    fn legacy_power_of_two_singleton_draw_and_invalid_bound() {
        let mut rng = LegacyRandom::seeded(123);
        let before = rng.seed();
        assert_eq!(rng.next_int(1), Ok(0));
        assert_ne!(rng.seed(), before);
        let before = rng.seed();
        assert!(rng.next_int(0).is_err());
        assert_eq!(rng.seed(), before);
    }
}
