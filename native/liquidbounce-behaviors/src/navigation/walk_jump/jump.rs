// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent Jump.kt, including walking back and landing duration.
use crate::navigation::{geometry::Vec3, walk::costs::Costs};
#[derive(Clone, Debug)]
pub struct Jump {
    pub run_up: Vec3,
    pub takeoff: i32,
    pub jumps: bool,
    pub air_ticks: i32,
    pub margin: f64,
    pub hops: i32,
    pub aim: Vec3,
}
impl Jump {
    pub const RUN_UP_TOLERANCE: f64 = 0.15;
    pub fn ticks(&self, from: Vec3) -> f64 {
        let dx = self.run_up.x - from.x;
        let dz = self.run_up.z - from.z;
        let back = (dx * dx + dz * dz).sqrt();
        let back_ticks = if back > Self::RUN_UP_TOLERANCE {
            back * Costs::WALK + 4.0
        } else {
            0.0
        };
        back_ticks + f64::from(self.takeoff) + f64::from(self.air_ticks) + 2.0
    }
}
