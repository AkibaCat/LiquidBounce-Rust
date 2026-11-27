// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ModuleKeepSprint and MixinPlayer at ae0eb396.
// Copyright (c) 2015-2026 CCBlueX.

use crate::random::bounded_f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeepSprint {
    pub motion: [f32; 2],
    pub motion_when_hurt: [f32; 2],
    pub hurt_time: [i32; 2],
    pub chance: f32,
    pub sprinting: bool,
}
impl Default for KeepSprint {
    fn default() -> Self {
        Self {
            motion: [100.0; 2],
            motion_when_hurt: [100.0; 2],
            hurt_time: [1, 10],
            chance: 100.0,
            sprinting: false,
        }
    }
}
impl KeepSprint {
    pub fn post_tick(&mut self, running: bool, actual_sprinting: bool) {
        if running {
            self.sprinting = actual_sprinting;
        }
    }
    pub fn attack_sprinting(self, running: bool, original: bool) -> bool {
        if running { self.sprinting } else { original }
    }
    /// Mixin's setter wrapper records attempts even while the module is off.
    pub fn set_sprinting(&mut self, running: bool, requested: bool) -> bool {
        self.sprinting = requested;
        !running || requested
    }
    /// Draws use the source's short circuits and float arithmetic, including
    /// nextUp(endInclusive), before widening for the double percentage division.
    pub fn motion_multiplier(self, hurt_time: i32, mut next_unit: impl FnMut() -> f32) -> f64 {
        let passes = match self.chance {
            0.0 => false,
            100.0 => true,
            chance => bounded_f32(next_unit(), 0.0, 100.0) < chance,
        };
        if !passes {
            return 0.6;
        }
        let [start, end] = if (self.hurt_time[0]..=self.hurt_time[1]).contains(&hurt_time) {
            self.motion_when_hurt
        } else {
            self.motion
        };
        let motion = if start >= end {
            start
        } else {
            bounded_f32(next_unit(), start, end.next_up())
        };
        f64::from(motion) / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_sprint_state_prevents_repeated_sprint_knockback_until_post_tick() {
        let mut state = KeepSprint::default();
        assert!(!state.attack_sprinting(true, true));
        state.post_tick(true, true);
        assert!(state.attack_sprinting(true, false));
        assert!(!state.set_sprinting(true, false));
        assert!(!state.attack_sprinting(true, true));
        state.post_tick(false, true);
        assert!(!state.attack_sprinting(true, true));
        state.post_tick(true, true);
        assert!(state.attack_sprinting(true, false));
        assert!(state.set_sprinting(false, false));
        assert!(!state.sprinting);
        assert!(state.set_sprinting(true, true));
    }
    #[test]
    fn chance_short_circuits_rng_and_hurt_range_is_inclusive() {
        let mut state = KeepSprint {
            motion: [25.0; 2],
            motion_when_hurt: [75.0; 2],
            ..KeepSprint::default()
        };
        let unused = || -> f32 { panic!("constant endpoints do not draw") };
        assert_eq!(state.motion_multiplier(0, unused), 0.25);
        assert_eq!(state.motion_multiplier(1, unused), 0.75);
        assert_eq!(state.motion_multiplier(10, unused), 0.75);
        assert_eq!(state.motion_multiplier(11, unused), 0.25);
        state.chance = 0.0;
        assert_eq!(state.motion_multiplier(5, unused), 0.6);
        state.chance = 50.0;
        assert_eq!(state.motion_multiplier(5, || 0.5), 0.6);
        assert_eq!(state.motion_multiplier(5, || 0.25), 0.75);
    }
    #[test]
    fn random_range_retains_float_rounding_and_inclusive_upper_endpoint() {
        let state = KeepSprint {
            motion: [20.0, 100.0],
            ..KeepSprint::default()
        };
        assert_eq!(state.motion_multiplier(0, || 0.0), 0.2);
        assert_eq!(state.motion_multiplier(0, || 1.0f32.next_down()), 1.0);
        let value = 0.37f32 * (100.0f32.next_up() - 20.0) + 20.0;
        assert_eq!(
            state.motion_multiplier(0, || 0.37).to_bits(),
            (f64::from(value) / 100.0).to_bits()
        );
    }
}
