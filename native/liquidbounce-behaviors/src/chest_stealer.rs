// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleChestStealer's selection and atomic click chains at the pinned source.
use crate::{
    chest_cleaner::ChestCleaner,
    inventory_cleaner::{ArmorSettings, Categorization, CleanupPlan, Item},
    inventory_manager::{Action, Input, Screen, Slot},
    random::ThreadRandom,
};
use std::collections::{BTreeSet, VecDeque};
pub const SELECTIONS: [&str; 4] = ["Value", "Distance", "Index", "Random"];
pub const STARTS: [&str; 4] = ["Default", "Random", "MaxSlot", "MinSlot"];
#[derive(Clone, Debug)]
pub struct Aura {
    pub enabled: bool,
    pub range: f32,
    pub walls: f32,
    pub delay: i32,
    pub hide_swing: bool,
    pub not_combat: bool,
    pub track_manual: bool,
    pub pause_on: u8,
    pub await_container: bool,
    pub timeout: i32,
    pub max_retries: i32,
    pub blocks: BTreeSet<String>,
}
impl Default for Aura {
    fn default() -> Self {
        Self {
            enabled: true,
            range: 3.,
            walls: 0.,
            delay: 5,
            hide_swing: false,
            not_combat: true,
            track_manual: true,
            pause_on: 0,
            await_container: true,
            timeout: 10,
            max_retries: 4,
            blocks: BTreeSet::new(),
        }
    }
}
impl Aura {
    pub fn running(&self, parent: bool, combat: bool, using: bool) -> bool {
        parent
            && self.enabled
            && !(combat && (self.not_combat || self.pause_on & 1 != 0)
                || using && self.pause_on & 2 != 0)
    }
}
#[derive(Clone, Debug)]
pub struct SilentScreen {
    pub enabled: bool,
    pub unlock: bool,
    pub draw: bool,
    pub texture: bool,
    pub color: i32,
    pub outline: i32,
    pub margin: f32,
    pub radius: f32,
    pub gradient: f32,
    pub glow: f32,
    pub scale: f32,
    pub offset: [f64; 3],
    pub title: bool,
}
impl Default for SilentScreen {
    fn default() -> Self {
        Self {
            enabled: false,
            unlock: false,
            draw: true,
            texture: false,
            color: i32::MIN,
            outline: 0,
            margin: 2.,
            radius: 4.,
            gradient: 0.3,
            glow: 0.3,
            scale: 1.5,
            offset: [0.; 3],
            title: false,
        }
    }
}
#[derive(Clone, Debug)]
pub struct ChestStealer {
    pub inventory: ChestCleaner,
    pub selection: usize,
    pub start: usize,
    pub factor: [f32; 2],
    pub descending: bool,
    pub drag: bool,
    pub quick_swaps: bool,
    pub throw_full: bool,
    pub aura: Aura,
    pub silent: SilentScreen,
}
impl Default for ChestStealer {
    fn default() -> Self {
        Self {
            inventory: ChestCleaner::default(),
            selection: 0,
            start: 0,
            factor: [1.; 2],
            descending: false,
            drag: false,
            quick_swaps: true,
            throw_full: true,
            aura: Default::default(),
            silent: Default::default(),
        }
    }
}
pub fn value(item: &Item, legacy: bool, baseline: f64, armor: &ArmorSettings) -> i32 {
    Categorization::default_value(item, legacy, baseline, armor)
}
pub fn item(items: &[Item], slot: Slot) -> Option<&Item> {
    items.iter().find(|i| i.slot == slot && !i.empty())
}
fn click(screen: &Screen, slot: Slot, input: Input, button: i32) -> Action {
    Action::Click {
        screen: Some(screen.clone()),
        slot,
        input,
        button,
    }
}
impl ChestStealer {
    pub fn select(
        &self,
        slots: &mut [Slot],
        items: &[Item],
        legacy: bool,
        baseline: f64,
        armor: &ArmorSettings,
        random: &mut ThreadRandom,
    ) {
        match self.selection {
            0 => slots.sort_by_key(|s| {
                std::cmp::Reverse(
                    item(items, *s).map_or(i32::MIN, |i| value(i, legacy, baseline, armor)),
                )
            }),
            2 => {
                slots.sort_by_key(|s| match s {
                    Slot::Container(i) => *i,
                    _ => -1,
                });
                if self.descending {
                    slots.reverse();
                }
            }
            3 => {
                for i in (1..slots.len()).rev() {
                    let j = random.next_i32_bound((i + 1) as i32) as usize;
                    slots.swap(i, j);
                }
            }
            1 if slots.len() > 1 => {
                let index = match self.start {
                    1 => random.next_i32_bound(slots.len() as i32) as usize,
                    2 => slots
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, s)| **s)
                        .map_or(0, |(i, _)| i),
                    3 => slots
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, s)| **s)
                        .map_or(0, |(i, _)| i),
                    _ => 0,
                };
                slots.swap(0, index);
                if slots.len() <= 2 {
                    return;
                }
                for i in 0..slots.len() - 1 {
                    let mut best = i + 1;
                    let mut shortest = f32::INFINITY;
                    for j in i + 1..slots.len() {
                        let id = |s: Slot| match s {
                            Slot::Container(i) => i,
                            _ => 0,
                        };
                        let (a, b) = (id(slots[i]), id(slots[j]));
                        let dx = a % 9 - b % 9;
                        let dy = a / 9 - b / 9;
                        let distance = (dx * dx + dy * dy) as f32;
                        let f = if self.factor[0] > self.factor[1] {
                            1.
                        } else if self.factor[0] >= self.factor[1] {
                            self.factor[0]
                        } else {
                            random.next_f32_between(self.factor[0], self.factor[1].next_up())
                        };
                        let distance = distance * f;
                        if distance < shortest {
                            shortest = distance;
                            best = j;
                        }
                    }
                    slots.swap(i + 1, best);
                }
            }
            _ => {}
        }
    }
    pub fn quick_swap(
        &self,
        items: &[Item],
        useful: &BTreeSet<Slot>,
        screen: &Screen,
        from: Slot,
        to: Slot,
        offhand: bool,
    ) -> Option<Vec<Action>> {
        if !matches!(from, Slot::Container(_))
            || !matches!(to, Slot::Player(0..=8 | 40))
            || to == Slot::Player(40) && !offhand
        {
            return None;
        }
        let Slot::Player(button) = to else {
            return None;
        };
        let swap = |from| click(screen, from, Input::Swap, i32::from(button));
        let actions = if item(items, to).is_none() {
            vec![swap(from)]
        } else if useful.contains(&to) {
            let empty = (9..36)
                .map(Slot::Player)
                .find(|s| item(items, *s).is_none())?;
            vec![swap(empty), swap(from)]
        } else if to == Slot::Player(40) {
            vec![swap(from)]
        } else {
            vec![click(screen, to, Input::Throw, 1), swap(from)]
        };
        actions
            .iter()
            .all(|a| match a {
                Action::Click { slot, .. } => slot.server_id(Some(screen)).is_some(),
                _ => true,
            })
            .then_some(actions)
    }
    pub fn pick(&self, items: &[Item], screen: &Screen, from: Slot) -> Option<Vec<Action>> {
        let source = item(items, from)?;
        let capacity = (0..36).map(Slot::Player).fold(0i32, |sum, slot| {
            sum.wrapping_add(match item(items, slot) {
                None => source.maximum,
                Some(target) if target.same(source) => target.maximum.wrapping_sub(target.count),
                _ => 0,
            })
        });
        if capacity == 0 {
            return None;
        }
        if !self.drag {
            return Some(vec![click(screen, from, Input::QuickMove, 0)]);
        }
        let mut targets = (0..36)
            .map(Slot::Player)
            .filter(|s| item(items, *s).is_none_or(|t| t.same(source)))
            .collect::<Vec<_>>();
        let remaining = |s: Slot| {
            source
                .maximum
                .wrapping_sub(source.count)
                .wrapping_sub(item(items, s).map_or(0, |t| t.count))
        };
        let mut actions = vec![click(screen, from, Input::Pickup, 0)];
        if let Some(target) = targets.iter().copied().find(|s| remaining(*s) >= 0) {
            actions.push(click(screen, target, Input::Pickup, 0));
        } else {
            targets.sort_by_key(|s| remaining(*s));
            let mut count = source.count;
            for target in targets {
                if count < 0 {
                    break;
                }
                count = count.wrapping_add(remaining(target));
                actions.push(click(screen, target, Input::Pickup, 0));
            }
        }
        if capacity < source.count {
            actions.push(click(screen, from, Input::Pickup, 0));
        }
        Some(actions)
    }
    pub fn throw_next(
        &self,
        items: &[Item],
        screen: &Screen,
        slots: &mut VecDeque<Slot>,
    ) -> Option<Vec<Action>> {
        let slot = std::iter::from_fn(|| slots.pop_front())
            .find(|s| item(items, *s).is_some() && s.server_id(Some(screen)).is_some())?;
        self.throw_full
            .then(|| vec![click(screen, slot, Input::Throw, 1)])
    }
}
pub fn required_space(items: &[Item], plan: &CleanupPlan, collect: usize) -> i32 {
    let free = (0..36)
        .map(Slot::Player)
        .filter(|s| item(items, *s).is_none())
        .count() as i32;
    let gained = plan.mergeable.iter().fold(0i32, |sum, group| {
        let values = group
            .iter()
            .filter_map(|s| item(items, *s))
            .collect::<Vec<_>>();
        let count = values.iter().fold(0i32, |sum, i| sum.wrapping_add(i.count));
        let maximum = values.first().map_or(1, |i| i.default_maximum);
        let merged = (f64::from(count) / f64::from(maximum)).ceil() as i32;
        let chest = values
            .iter()
            .filter(|i| matches!(i.slot, Slot::Container(_)))
            .count() as i32;
        sum.wrapping_add((group.len() as i32 - merged).min(chest))
    });
    (collect as i32)
        .wrapping_sub(free)
        .wrapping_sub(gained)
        .max(0)
}
