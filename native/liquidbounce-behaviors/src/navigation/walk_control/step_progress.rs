// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent StepProgress.kt state, Movement and BridgeLine.
use super::{PROGRESS, horizontal, subtract};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    progress::ProgressWatch,
    walk_jump::{leap::Leap, vector_yaw},
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Movement {
    pub target: Option<Vec3>,
    pub sprint: bool,
    pub dive: bool,
    pub is_plain_walk: bool,
    pub safe_walk: bool,
}
pub struct StepProgress {
    pub leap: Leap,
    pub jump_ticks: i32,
    pub watch: ProgressWatch,
    pub acting: Option<BlockPos>,
    acting_ticks: i32,
}
impl Default for StepProgress {
    fn default() -> Self {
        Self {
            leap: Leap::default(),
            jump_ticks: 0,
            watch: ProgressWatch::new(PROGRESS, 40),
            acting: None,
            acting_ticks: 0,
        }
    }
}
impl StepProgress {
    pub fn is_due_to_predict(&self) -> bool {
        self.watch.idle_ticks() > 0 && self.watch.idle_ticks() % 5 == 0
    }
    pub fn acts_too_long(&mut self, pos: BlockPos) -> bool {
        if self.acting != Some(pos) {
            self.acting = Some(pos);
            self.acting_ticks = 0;
            return false;
        }
        self.acting_ticks = self.acting_ticks.wrapping_add(1);
        self.acting_ticks >= 60
    }
}
#[derive(Clone, Copy, Debug)]
pub struct BridgeLine {
    pub from: Vec3,
    pub to: Vec3,
    length: f64,
    dx: f64,
    dz: f64,
}
impl BridgeLine {
    pub fn new(from: Vec3, to: Vec3) -> Self {
        let length = horizontal(from, to);
        Self {
            from,
            to,
            length,
            dx: if length > 0.0 {
                (to.x - from.x) / length
            } else {
                0.0
            },
            dz: if length > 0.0 {
                (to.z - from.z) / length
            } else {
                0.0
            },
        }
    }
    pub fn heading(&self) -> f32 {
        let way = subtract(self.to, self.from);
        vector_yaw(way.x, way.z)
    }
    pub fn aim_from(&self, position: Vec3) -> Vec3 {
        let along = (position.x - self.from.x) * self.dx + (position.z - self.from.z) * self.dz;
        // Kotlin coerceIn preserves NaN; f64::clamp does likewise when bounds are valid.
        let ahead = (along + 2.0).clamp(0.0, self.length);
        Vec3::new(
            self.from.x + self.dx * ahead,
            self.to.y,
            self.from.z + self.dz * ahead,
        )
    }
}
