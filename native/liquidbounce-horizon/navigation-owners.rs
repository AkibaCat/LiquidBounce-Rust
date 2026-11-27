// SPDX-License-Identifier: GPL-3.0-or-later
//! Independent navigation owners share physical input holds and hotbar selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Owner {
    #[default]
    AutoWalk,
    FightBot,
}
impl Owner {
    pub const fn index(self) -> usize {
        match self {
            Self::AutoWalk => 0,
            Self::FightBot => 1,
        }
    }
    pub const fn bit(self) -> u8 {
        1 << self.index()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Holds(u8);
impl Holds {
    pub const fn empty() -> Self {
        Self(0)
    }
    pub fn any(self) -> bool {
        self.0 != 0
    }
    pub fn owned(self, owner: Owner) -> bool {
        self.0 & owner.bit() != 0
    }
    pub fn releasing_last(self, owner: Owner) -> bool {
        self.0 == owner.bit()
    }
    pub fn set(&mut self, owner: Owner, value: bool) {
        if value {
            self.0 |= owner.bit();
        } else {
            self.0 &= !owner.bit();
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Claim {
    slot: usize,
    order: u64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Slots {
    baseline: Option<usize>,
    claims: [Option<Claim>; 2],
    order: u64,
}
impl Slots {
    pub fn claim(&mut self, owner: Owner, slot: usize, current: usize) {
        self.baseline.get_or_insert(current);
        self.order = self.order.wrapping_add(1);
        self.claims[owner.index()] = Some(Claim {
            slot,
            order: self.order,
        });
    }
    pub fn release(&mut self, owner: Owner) -> Option<usize> {
        self.claims[owner.index()].take()?;
        if let Some(claim) = self.claims.iter().flatten().max_by_key(|c| c.order) {
            Some(claim.slot)
        } else {
            self.baseline.take()
        }
    }
}
