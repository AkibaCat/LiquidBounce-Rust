// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Setbacks.kt, including same-tick packet suppression.
use crate::navigation::geometry::{BlockPos, Vec3};
#[derive(Default)]
pub struct Setbacks {
    spot: Vec<Vec3>,
    last_tick: i64,
}
impl Setbacks {
    pub fn refused(&mut self, at: Vec3, tick: i64) -> Vec<BlockPos> {
        if tick == self.last_tick && !self.spot.is_empty() {
            return Vec::new();
        }
        let joins = tick.wrapping_sub(self.last_tick) <= 60
            && self.spot.iter().any(|p| p.distance_sqr(at) < 1.5 * 1.5);
        if !joins {
            self.spot.clear();
        }
        self.spot.push(at);
        self.last_tick = tick;
        let mut result = Vec::new();
        if self.spot.len() > 1 {
            for point in &self.spot {
                let block = BlockPos::containing(*point);
                if !result.contains(&block) {
                    result.push(block);
                }
            }
        }
        result
    }
}
