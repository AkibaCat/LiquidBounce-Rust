// SPDX-License-Identifier: GPL-3.0-or-later
//! Original mutable rocket burn and inventory accounting.
#[derive(Clone, Copy, Debug)]
pub struct Rockets {
    left: i32,
    burn_ticks: i32,
    boost_ticks: i32,
}
impl Rockets {
    pub const fn new(left: i32, burn_ticks: i32) -> Self {
        Self {
            left,
            burn_ticks,
            boost_ticks: 0,
        }
    }
    pub fn left(&self) -> i32 {
        self.left
    }
    pub fn boost_ticks(&self) -> i32 {
        self.boost_ticks
    }
    pub fn can_fire(&self) -> bool {
        self.boost_ticks == 0 && self.left > 0
    }
    pub fn fire(&mut self) {
        self.left = self.left.wrapping_sub(1);
        self.boost_ticks = self.burn_ticks;
    }
    pub fn tick(&mut self) -> bool {
        if self.boost_ticks <= 0 {
            return false;
        }
        self.boost_ticks = self.boost_ticks.wrapping_sub(1);
        true
    }
}
