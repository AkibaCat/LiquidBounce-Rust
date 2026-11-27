// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac simulation sources. Native world binding and full
//! simulation validation remain separate from each translated source unit.
pub mod block_colliders;
pub mod block_grid;
pub mod collisions;
pub mod edge_backoff;
mod motion_1_8;
pub mod motion_traits;
pub mod player_motion;
pub mod sine_table;
pub mod support;

fn java_min_f32(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a == 0.0 && b == 0.0 {
        f32::from_bits(a.to_bits() | b.to_bits())
    } else {
        a.min(b)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mechanics {
    #[default]
    Modern,
    V1_8,
}
impl Mechanics {
    pub const fn max_packets_per_tick(self) -> i32 {
        match self {
            Self::Modern => 5,
            Self::V1_8 => i32::MAX,
        }
    }
    pub const fn is_move_judged_per_tick(self) -> bool {
        matches!(self, Self::Modern)
    }
    pub fn axis_step_order(self, movement: super::geometry::Vec3) -> [super::aabb::Axis; 3] {
        use super::aabb::Axis::{X, Y, Z};
        if self == Self::Modern && movement.x.abs() < movement.z.abs() {
            [Y, Z, X]
        } else {
            [Y, X, Z]
        }
    }
}
