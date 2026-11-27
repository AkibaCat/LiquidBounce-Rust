// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Costs.kt, preserving the float ground-drag multiplication.
use super::WalkState;
use std::sync::LazyLock;

pub struct Costs;
static FALL_TICKS: LazyLock<[f64; 257]> = LazyLock::new(|| {
    let mut ticks = [0.0; 257];
    let (mut distance, mut velocity, mut tick) = (0.0, 0.0, 0);
    for (blocks, result) in ticks.iter_mut().enumerate() {
        while distance < blocks as f64 {
            velocity = (velocity + 0.08) * 0.98;
            distance += velocity;
            tick += 1;
        }
        *result = f64::from(tick);
    }
    ticks
});
impl Costs {
    pub const WALK: f64 = 20.0 / 4.317;
    pub const SPRINT: f64 = 20.0 / 5.612;
    pub const SWIM: f64 = 20.0 / 2.2;
    pub const CLIMB_UP: f64 = 20.0 / 2.35;
    pub const CLIMB_DOWN: f64 = 20.0 / 3.0;
    pub const SNEAK: f64 = 20.0 / 1.295;
    pub const SPRINT_SWIM: f64 = 20.0 / 3.918;
    pub const PLACE: f64 = 20.0;
    pub const RUN_BRIDGE: f64 = 3.0;
    pub const NINJA_BRIDGE: f64 = 20.0 / 2.7 - Self::SPRINT;
    pub const GODBRIDGE: f64 = 20.0 / 3.5 - Self::SPRINT;
    pub const BREAK_OVERHEAD: f64 = 2.0;
    pub const AVOID: f64 = 50.0;
    pub const JUMP_PENALTY: f64 = 20.0;
    pub const OPEN: f64 = 10.0;
    pub const HURT: f64 = 100.0;
    pub fn fall(blocks: i32) -> f64 {
        FALL_TICKS[blocks.clamp(0, 256) as usize]
    }
    pub fn jump() -> f64 {
        Self::fall(2) - Self::fall(1)
    }
    pub fn floor_factor<B: WalkState>(block: &B) -> f64 {
        let slowdown = f64::from(block.speed_factor()) * if block.is_slime() { 0.4 } else { 1.0 };
        if slowdown < 1.0 {
            Self::ground_speed(0.6, 1.0) / Self::ground_speed(block.friction(), slowdown)
        } else {
            1.0
        }
    }
    pub fn bounce<B: WalkState>(block: &B, blocks: i32) -> f64 {
        if block.is_slime() {
            4.0 * Self::fall(blocks)
        } else {
            0.0
        }
    }
    fn ground_speed(friction: f32, slowdown: f64) -> f64 {
        let drag = f64::from(friction * 0.91_f32);
        0.216 / (drag * drag * drag) / (1.0 - slowdown * drag)
    }
}
