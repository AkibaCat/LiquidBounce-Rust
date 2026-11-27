// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAirPlace and its inherited ScrollAdjustValueGroup.
use crate::camera_clip::ModifierKey;
#[path = "air_place_target.rs"]
mod target_math;
pub use target_math::target;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub preview: bool,
    pub outline_only: bool,
    pub color: i32,
    pub outline_color: i32,
    pub place_in_liquid: bool,
    pub custom_range: bool,
    pub range: f32,
    pub scroll_adjust: bool,
    pub modifier: ModifierKey,
    pub sensitivity: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            preview: true,
            outline_only: false,
            color: 0x684577ff,
            outline_color: -1,
            place_in_liquid: false,
            custom_range: false,
            range: 3.,
            scroll_adjust: true,
            modifier: ModifierKey::from_name("key.keyboard.left.alt").expect("pinned LeftAlt"),
            sensitivity: 0.5,
        }
    }
}
impl Settings {
    pub fn can_scroll(&self, running: bool, modifier_down: bool) -> bool {
        running
            && self.custom_range
            && self.scroll_adjust
            && (self.modifier.is_unknown() || modifier_down)
    }
    pub fn scroll(&mut self, running: bool, modifier_down: bool, vertical: f64) {
        if self.can_scroll(running, modifier_down) {
            self.range = (self.range + vertical as f32 * self.sensitivity).clamp(1., 4.5);
        }
    }
}
