// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleReplenish at ae0eb396. The shared InventoryManager executes each
//! emitted chain; complete() is called at its synchronous continuation tail.
use crate::vomit::Constraints;

pub const FEATURES: [&str; 3] = ["CleanUp", "UsePickupAll", "UseSwap"];
pub const INSIDE_OF: [&str; 2] = ["Chests", "Inventories"];
pub const HOTBAR: [u8; 10] = [40, 0, 1, 2, 3, 4, 5, 6, 7, 8];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Item {
    pub components: String,
    pub count: i32,
    pub maximum: i32,
}
impl Item {
    fn empty(&self) -> bool {
        self.count <= 0
    }
    fn tracked(&self) -> Self {
        Self {
            count: 1,
            ..self.clone()
        }
    }
    fn mergeable(&self, other: &Self) -> bool {
        !self.empty() && !other.empty() && self.components == other.components
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Pickup,
    PickupAll,
    Swap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Click {
    pub slot: u8,
    pub button: i32,
    pub input: Input,
}
impl Click {
    fn pickup(slot: u8) -> Self {
        Self {
            slot,
            button: 0,
            input: Input::Pickup,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Chain(Vec<Click>),
    Wait,
    Done,
}
#[derive(Clone, Debug)]
struct Refill {
    index: usize,
    desired: Item,
    sources: Vec<u8>,
    next: usize,
    needed: i32,
}
#[derive(Clone, Debug)]
pub struct Replenish {
    pub constraints: Constraints,
    pub threshold: i32,
    pub delay: i32,
    pub replenish_empty: bool,
    pub features: u32,
    pub inside_of: u32,
    tracked: [Option<Item>; 10],
    last_update: Option<i64>,
    scan: usize,
    active: bool,
    refill: Option<Refill>,
    wait_until: Option<i64>,
}
impl Default for Replenish {
    fn default() -> Self {
        Self {
            constraints: Default::default(),
            threshold: 5,
            delay: 40,
            replenish_empty: true,
            features: 1,
            inside_of: 0,
            tracked: Default::default(),
            last_update: None,
            scan: 0,
            active: false,
            refill: None,
            wait_until: None,
        }
    }
}
impl Replenish {
    pub fn running(&self, parent: bool, abstract_container: bool, inventory: bool) -> bool {
        parent
            && (self.inside_of & 1 != 0 || !abstract_container || inventory)
            && (self.inside_of & 2 != 0 || !inventory)
    }
    /// ScreenEvent and onEnabled clear item memory, not the chronometer.
    pub fn clear_tracked(&mut self) {
        self.tracked.fill(None);
    }
    pub fn cancel(&mut self) {
        self.active = false;
        self.refill = None;
        self.wait_until = None;
    }
    pub fn world_change(&mut self) {
        self.cancel();
        self.clear_tracked();
    }
    pub fn begin(&mut self, now: i64) -> bool {
        if self.active
            || self
                .last_update
                .is_some_and(|last| last.wrapping_add(i64::from(self.delay)) >= now)
        {
            return false;
        }
        self.last_update = Some(now);
        self.active = true;
        self.scan = 0;
        true
    }
    pub fn ready(&self, now: i64) -> bool {
        self.wait_until.is_none_or(|until| now >= until)
    }
    pub fn next(&mut self, stacks: &[Item], cursor_empty: bool, now: i64) -> Step {
        if !self.active {
            return Step::Done;
        }
        if !self.ready(now) {
            return Step::Wait;
        }
        self.wait_until = None;
        loop {
            if let Some(refill) = self.refill.as_mut() {
                if let Some(&slot) = refill.sources.get(refill.next) {
                    refill.next += 1;
                    let count = stacks.get(usize::from(slot)).map_or(0, |s| s.count);
                    refill.needed = refill.needed.wrapping_sub(count);
                    let target = HOTBAR[refill.index];
                    let mut actions = vec![Click::pickup(slot), Click::pickup(target)];
                    // Preserve the source's third click on the target itself.
                    if refill.needed < 0 {
                        actions.push(Click::pickup(target));
                    }
                    if refill.needed <= 0 {
                        refill.next = refill.sources.len();
                    }
                    return Step::Chain(actions);
                }
                let refill = self.refill.take().unwrap();
                self.tracked[refill.index] = Some(refill.desired.tracked());
                self.last_update = Some(now);
                if self.delay > 0 {
                    self.wait_until = Some(now.wrapping_add(i64::from(self.delay)));
                    return Step::Wait;
                }
            }
            if self.scan >= HOTBAR.len() || !cursor_empty {
                self.cancel();
                return Step::Done;
            }
            let index = self.scan;
            self.scan += 1;
            let slot = HOTBAR[index];
            let current = stacks.get(usize::from(slot)).cloned().unwrap_or_default();
            let nonempty = !current.empty();
            if !nonempty && !self.replenish_empty {
                self.tracked[index] = None;
                continue;
            }
            let desired = if nonempty {
                current
            } else {
                let Some(item) = self.tracked[index].clone() else {
                    continue;
                };
                item
            };
            if nonempty && (desired.maximum <= self.threshold || desired.count > self.threshold) {
                self.tracked[index] = Some(desired.tracked());
                continue;
            }
            let mut sources: Vec<_> = (9u8..36)
                .filter(|&slot| {
                    stacks
                        .get(usize::from(slot))
                        .is_some_and(|s| s.mergeable(&desired))
                })
                .collect();
            // Kotlin's stable sort, followed by reverse() for CleanUp.
            sources.sort_by(|a, b| {
                stacks[usize::from(*b)]
                    .count
                    .cmp(&stacks[usize::from(*a)].count)
            });
            let Some(&largest) = sources.first() else {
                self.tracked[index] = Some(desired.tracked());
                continue;
            };
            let largest_count = stacks[usize::from(largest)].count;
            if self.features & 1 != 0 {
                sources.reverse();
            }
            let count = if nonempty { desired.count } else { 0 };
            let special = if self.features & 4 != 0
                && largest_count > desired.count
                && largest_count > self.threshold
            {
                Some(vec![Click {
                    slot: largest,
                    button: i32::from(slot),
                    input: Input::Swap,
                }])
            } else if self.features & 2 != 0 && nonempty {
                Some(vec![
                    Click::pickup(slot),
                    Click {
                        slot,
                        button: 0,
                        input: Input::PickupAll,
                    },
                    Click::pickup(slot),
                ])
            } else {
                None
            };
            let next = if special.is_some() { sources.len() } else { 0 };
            self.refill = Some(Refill {
                index,
                needed: desired.maximum.wrapping_sub(count),
                desired,
                sources,
                next,
            });
            if let Some(actions) = special {
                return Step::Chain(actions);
            }
        }
    }
}
