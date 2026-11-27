// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTridentBoost / MixinTridentItem, LiquidBounce ae0eb396.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TridentBoost {
    pub horizontal_multiplier: f32,
    pub vertical_multiplier: f32,
    pub on_land: bool,
}

impl Default for TridentBoost {
    fn default() -> Self {
        Self {
            horizontal_multiplier: 2.0,
            vertical_multiplier: 2.0,
            on_land: true,
        }
    }
}
impl TridentBoost {
    pub fn water_requirement(&self, running: bool, original: bool) -> bool {
        original || (running && self.on_land)
    }
    /// The original push arguments are doubles; multipliers widen from Float
    /// before multiplication. Existing velocity is never scaled.
    pub fn impulse(&self, running: bool, original: [f64; 3]) -> [f64; 3] {
        if !running {
            return original;
        }
        [
            original[0] * f64::from(self.horizontal_multiplier),
            original[1] * f64::from(self.vertical_multiplier),
            original[2] * f64::from(self.horizontal_multiplier),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wet_check_and_only_impulse_arguments_follow_independent_settings() {
        let boost = TridentBoost {
            horizontal_multiplier: 0.5,
            vertical_multiplier: 4.0,
            on_land: true,
        };
        assert_eq!(boost.impulse(true, [3.0, -2.0, 5.0]), [1.5, -8.0, 2.5]);
        assert_eq!(boost.impulse(false, [3.0, -2.0, 5.0]), [3.0, -2.0, 5.0]);
        assert!(boost.water_requirement(true, false));
        assert!(!boost.water_requirement(false, false));
        let boost = TridentBoost {
            on_land: false,
            ..boost
        };
        assert!(!boost.water_requirement(true, false));
        assert!(boost.water_requirement(false, true));
        assert!(boost.water_requirement(true, true));
    }
}
