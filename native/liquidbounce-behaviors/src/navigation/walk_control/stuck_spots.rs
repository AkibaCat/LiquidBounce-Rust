// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete StuckSpots from Agent Planning.kt. clear intentionally keeps version.
use crate::navigation::geometry::{BlockPos, Vec3};
use std::collections::HashSet;
#[derive(Default)]
pub struct StuckSpots {
    pub blocks: HashSet<i64>,
    version: i32,
    goal: Option<Vec3>,
    age: i32,
}
impl StuckSpots {
    pub fn version(&self) -> i32 {
        self.version
    }
    pub fn add(&mut self, pos: BlockPos) -> bool {
        if !self.blocks.insert(pos.packed()) {
            return false;
        }
        self.version = self.version.wrapping_add(1);
        true
    }
    pub fn clear(&mut self) {
        self.blocks.clear();
    }
    pub fn tick(&mut self, goal: Vec3) {
        let outdated = !self.blocks.is_empty() && {
            self.age = self.age.wrapping_add(1);
            self.age > 600
                || self
                    .goal
                    .is_none_or(|old| old.distance_sqr(goal) > 8.0 * 8.0)
        };
        if outdated {
            self.blocks.clear();
            self.version = self.version.wrapping_add(1);
        }
        if self.blocks.is_empty() {
            self.goal = Some(goal);
            self.age = 0;
        }
    }
}
