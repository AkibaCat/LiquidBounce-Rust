// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete MotionTraits calculations from Minecraft Agent 0abf69ac.
use super::Mechanics;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTraits {
    pub walking_speed: f32,
    pub jump_strength: f32,
    pub jump_boost_power: f32,
    pub mechanics: Mechanics,
    pub jump_power: f64,
    pub jump_height: f64,
}
impl Default for MotionTraits {
    fn default() -> Self {
        Self::new(0.1, 0.42, 0.0, Mechanics::Modern)
    }
}
impl MotionTraits {
    pub fn new(
        walking_speed: f32,
        jump_strength: f32,
        jump_boost_power: f32,
        mechanics: Mechanics,
    ) -> Self {
        let jump_power = f64::from(jump_strength + jump_boost_power);
        Self {
            walking_speed,
            jump_strength,
            jump_boost_power,
            mechanics,
            jump_power,
            jump_height: Self::rise(jump_power),
        }
    }
    pub fn rise(delta_y: f64) -> f64 {
        let mut motion = delta_y;
        let mut height = 0.0;
        while motion > 0.0 {
            height += motion;
            motion = (motion - 0.08) * f64::from(0.98_f32);
        }
        height
    }
    /// Input attributes are the actual live values before the original float
    /// casts. The adapter captures these on the game thread.
    pub fn capture(
        movement_speed: f64,
        jump_strength: f64,
        jump_boost_power: f32,
        is_sprinting: bool,
        mechanics: Mechanics,
    ) -> Self {
        let speed = movement_speed as f32;
        Self::new(
            if is_sprinting { speed / 1.3_f32 } else { speed },
            jump_strength as f32,
            jump_boost_power,
            mechanics,
        )
    }
}
