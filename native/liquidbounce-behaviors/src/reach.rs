// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from ModuleReach and RangeValueGroup at LiquidBounce ae0eb396.
// Copyright (c) 2015-2026 CCBlueX.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reach {
    pub range_increase: f32,
    pub through_walls_range: f32,
    pub block_range_increase: f32,
}
impl Default for Reach {
    fn default() -> Self {
        Self {
            range_increase: 1.0,
            through_walls_range: 0.0,
            block_range_increase: 0.5,
        }
    }
}
impl Reach {
    pub fn interaction_range(self, original_attribute: Option<f64>) -> f32 {
        original_attribute.map_or(3.0, |v| v as f32) + self.range_increase
    }
    pub fn query_range(self, attribute: &str, original: f64) -> f64 {
        match attribute {
            "minecraft:entity_interaction_range" => {
                f64::from(self.interaction_range(Some(original)))
            }
            "minecraft:block_interaction_range" => original + f64::from(self.block_range_increase),
            _ => original,
        }
    }
    /// This is a setting onChange, not a dynamic query-time clamp. Changing
    /// RangeIncrease later must retain the already stored ThroughWallsRange.
    pub fn change_through_walls_range(&mut self, requested: f32, original_attribute: Option<f64>) {
        self.through_walls_range = self.interaction_range(original_attribute).min(requested);
    }
    pub fn adjust_attack_range(
        self,
        min: &mut f32,
        max: &mut f32,
        creative_min: &mut f32,
        creative_max: &mut f32,
    ) {
        *min = min.max(0.0);
        *max += self.range_increase;
        *creative_min = creative_min.max(0.0);
        *creative_max += self.range_increase;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn entity_range_narrows_before_addition_but_block_range_stays_double() {
        let state = Reach::default();
        let raw = 16_777_217.25;
        assert_eq!(
            state.query_range("minecraft:entity_interaction_range", raw),
            f64::from(raw as f32 + 1.0)
        );
        assert_eq!(
            state.query_range("minecraft:block_interaction_range", raw),
            raw + 0.5
        );
        assert_eq!(state.query_range("other", raw), raw);
    }
    #[test]
    fn through_walls_setting_clamps_only_when_written_with_no_player_fallback() {
        let mut state = Reach::default();
        state.change_through_walls_range(8.0, None);
        assert_eq!(state.through_walls_range, 4.0);
        state.range_increase = 0.0;
        assert_eq!(state.through_walls_range, 4.0);
        state.change_through_walls_range(8.0, Some(5.0));
        assert_eq!(state.through_walls_range, 5.0);
    }
    #[test]
    fn both_survival_and_creative_attack_bounds_use_float_arithmetic() {
        let mut values = (-2.0, 4.5, -1.0, 6.0);
        Reach::default().adjust_attack_range(
            &mut values.0,
            &mut values.1,
            &mut values.2,
            &mut values.3,
        );
        assert_eq!(values, (0.0, 5.5, 0.0, 7.0));
    }
}
