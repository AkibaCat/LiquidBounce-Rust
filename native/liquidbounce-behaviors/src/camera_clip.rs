// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleCameraClip and ScrollAdjustValueGroup, LiquidBounce ae0eb396.
use crate::Easing;

#[path = "camera_clip_keys.rs"]
mod keys;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModifierKey {
    pub keyboard: bool,
    pub value: i32,
    pub name: String,
}
impl Default for ModifierKey {
    fn default() -> Self {
        Self::from_name("key.keyboard.left.control").expect("pinned key exists")
    }
}
impl ModifierKey {
    /// InputConstants.getKey resolves named keys before numeric suffixes.
    /// Its numeric fallback canonicalizes existing codes to their known name.
    pub fn from_name(name: &str) -> Option<Self> {
        for (keyboard, table, prefix) in [
            (true, keys::KEYBOARD, "key.keyboard"),
            (false, keys::MOUSE, "key.mouse"),
        ] {
            if let Some((value, name)) = table.iter().find(|(_, key)| *key == name) {
                return Some(Self {
                    keyboard,
                    value: *value,
                    name: (*name).into(),
                });
            }
            // The JVM tests startsWith(prefix), then drops prefix.length+1;
            // it does not require that the discarded character is a period.
            if name.starts_with(prefix) {
                let suffix = name.get(prefix.len() + 1..)?;
                if suffix.is_empty()
                    || suffix
                        .bytes()
                        .any(|b| !b.is_ascii_digit() && b != b'-' && b != b'+')
                {
                    return None;
                }
                let value: i32 = suffix.parse().ok()?;
                let canonical = table
                    .iter()
                    .find(|(code, _)| *code == value)
                    .map(|(_, key)| (*key).to_owned())
                    .unwrap_or_else(|| format!("{prefix}.{value}"));
                return Some(Self {
                    keyboard,
                    value,
                    name: canonical,
                });
            }
        }
        None
    }
    pub fn is_unknown(&self) -> bool {
        self.keyboard && self.value == 0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CameraClip {
    pub camera_distance: f32,
    pub animation_enabled: bool,
    pub animation_speed: f32,
    pub animation_easing: Easing,
    pub scroll_enabled: bool,
    pub modifier: ModifierKey,
    pub sensitivity: f32,
    pub remember_scrolled: bool,
    pub require_free_look: bool,
    pub scrolled_distance: f32,
}
impl Default for CameraClip {
    fn default() -> Self {
        Self {
            camera_distance: 4.0,
            animation_enabled: true,
            animation_speed: 0.6,
            animation_easing: Easing::Linear,
            scroll_enabled: true,
            modifier: ModifierKey::default(),
            sensitivity: 0.3,
            remember_scrolled: false,
            require_free_look: false,
            scrolled_distance: 4.0,
        }
    }
}
impl CameraClip {
    /// This setter really uses coerceIn(1f..48f); imported settings do not.
    fn set_scrolled_distance(&mut self, value: f32) {
        self.scrolled_distance = value.clamp(1.0, 48.0);
    }
    pub fn distance(&self, running: bool) -> f32 {
        if running && self.scroll_enabled {
            self.scrolled_distance
        } else {
            self.camera_distance
        }
    }
    /// Return whether reset changed the persisted CameraDistance value.
    pub fn reset(&mut self) -> bool {
        if self.remember_scrolled && self.scrolled_distance != self.camera_distance {
            self.camera_distance = self.scrolled_distance;
            true
        } else {
            self.set_scrolled_distance(self.camera_distance);
            false
        }
    }
    /// Parent enabling updates enabled children; disabled parents do not call
    /// the child's onEnabled when merely changing its Enabled setting.
    pub fn parent_enabled(&mut self) -> bool {
        self.scroll_enabled && self.reset()
    }
    pub fn set_scroll_enabled(&mut self, value: bool, parent_enabled: bool, in_game: bool) -> bool {
        let changed = value != self.scroll_enabled;
        self.scroll_enabled = value;
        changed && value && parent_enabled && in_game && self.reset()
    }
    /// Normal-priority listener, before AutoF5 and QuickPerspectiveSwap.
    /// Disabling Animation intentionally leaves the vanilla distance intact.
    pub fn perspective(
        &self,
        running: bool,
        last_perspective: u8,
        perspective: u8,
        last_distance: &mut f32,
        distance: &mut f32,
        no_clip: &mut bool,
    ) {
        if !running {
            return;
        }
        *no_clip = true;
        if !self.animation_enabled {
            return;
        }
        if last_perspective != perspective {
            *last_distance = 0.0;
        }
        if perspective != 0 {
            let factor = self.animation_easing.transform(self.animation_speed);
            *distance = *last_distance + factor * (self.distance(running) - *last_distance);
        }
    }
    /// READ_FINAL_STATE listener sees the perspective after later modules.
    pub fn final_perspective(&mut self, running: bool, perspective: u8) -> bool {
        running && self.scroll_enabled && perspective == 0 && self.reset()
    }
    pub fn can_scroll(
        &self,
        running: bool,
        modifier_down: bool,
        raw_perspective: u8,
        free_look: bool,
    ) -> bool {
        running
            && self.scroll_enabled
            && (self.modifier.is_unknown() || modifier_down)
            && (!self.require_free_look || free_look)
            && (raw_perspective != 0 || free_look)
    }
    /// Raw double vertical is narrowed before multiplying the float setting.
    /// The raw event is not canceled; only the later hotbar event is canceled.
    pub fn scroll(
        &mut self,
        running: bool,
        modifier_down: bool,
        raw_perspective: u8,
        free_look: bool,
        vertical: f64,
    ) {
        if self.can_scroll(running, modifier_down, raw_perspective, free_look) {
            let delta = vertical as f32 * self.sensitivity;
            self.set_scrolled_distance(self.scrolled_distance + delta);
        }
    }
    pub fn keyboard_release(&mut self, running: bool, scan: i32) -> bool {
        running
            && self.scroll_enabled
            && self.modifier.keyboard
            && self.modifier.value == scan
            && self.reset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keys_use_sdl_and_input_constants_name_rules() {
        assert_eq!(ModifierKey::default().value, 224);
        assert!(
            ModifierKey::from_name("key.keyboard.unknown")
                .unwrap()
                .is_unknown()
        );
        assert_eq!(ModifierKey::from_name("key.keyboard.0").unwrap().value, 39);
        assert_eq!(
            ModifierKey::from_name("key.keyboard.224").unwrap().name,
            "key.keyboard.left.control"
        );
        assert_eq!(
            ModifierKey::from_name("key.mouse.1").unwrap().name,
            "key.mouse.left"
        );
        assert!(!ModifierKey::from_name("key.mouse.0").unwrap().is_unknown());
        assert_eq!(ModifierKey::from_name("key.keyboardx004").unwrap().value, 4);
        assert!(ModifierKey::from_name("left.control").is_none());
        assert!(ModifierKey::from_name("key.keyboard.2147483648").is_none());
    }
    #[test]
    fn scroll_gate_uses_raw_pov_and_freelook_dependency() {
        let mut c = CameraClip::default();
        assert!(!c.can_scroll(false, true, 1, false));
        assert!(!c.can_scroll(true, false, 1, false));
        assert!(!c.can_scroll(true, true, 0, false));
        assert!(c.can_scroll(true, true, 1, false));
        c.require_free_look = true;
        assert!(!c.can_scroll(true, true, 1, false));
        assert!(c.can_scroll(true, true, 0, true));
        c.modifier = ModifierKey::from_name("key.keyboard.unknown").unwrap();
        assert!(c.can_scroll(true, false, 0, true));
        c.scroll_enabled = false;
        assert!(!c.can_scroll(true, true, 1, true));
    }
    #[test]
    fn imported_values_are_raw_but_scroll_has_the_source_setter_clamp() {
        let mut c = CameraClip {
            camera_distance: 100.0,
            ..Default::default()
        };
        c.reset();
        assert_eq!(c.scrolled_distance, 48.0);
        c.scroll(true, true, 1, false, -1000.);
        assert_eq!(c.scrolled_distance, 1.0);
        c.scroll(true, true, 1, false, 1000.);
        assert_eq!(c.scrolled_distance, 48.0);
        c.scroll(true, true, 1, false, f64::NAN);
        assert!(c.scrolled_distance.is_nan());
        c.reset();
        assert_eq!(c.scrolled_distance, 48.0);
    }
    #[test]
    fn reset_remember_release_and_child_enable_obey_their_actual_lifecycles() {
        let mut c = CameraClip::default();
        c.scroll(true, true, 1, false, 10.);
        assert_eq!(c.scrolled_distance, 7.0);
        assert!(!c.keyboard_release(true, 225));
        assert_eq!(c.scrolled_distance, 7.0);
        c.keyboard_release(true, 224);
        assert_eq!(c.scrolled_distance, 4.0);
        c.remember_scrolled = true;
        c.scroll(true, true, 1, false, 10.);
        assert!(c.final_perspective(true, 0));
        assert_eq!(c.camera_distance, 7.0);
        c.camera_distance = 2.;
        c.scroll_enabled = false;
        c.set_scroll_enabled(true, false, true);
        assert_eq!(c.camera_distance, 2.);
        assert!(c.parent_enabled());
        assert_eq!(c.camera_distance, 7.);
        c.remember_scrolled = false;
        c.camera_distance = 3.;
        c.scroll_enabled = false;
        c.set_scroll_enabled(true, true, true);
        assert_eq!(c.scrolled_distance, 3.);
    }
    #[test]
    fn animation_reads_previous_final_state_and_disabled_animation_keeps_base() {
        let mut c = CameraClip::default();
        let mut last = 4.;
        let mut distance = 4.;
        let mut clip = false;
        c.perspective(true, 0, 1, &mut last, &mut distance, &mut clip);
        assert_eq!(last, 0.);
        assert_eq!(distance, 2.4);
        assert!(clip);
        last = distance;
        c.perspective(true, 1, 1, &mut last, &mut distance, &mut clip);
        assert_eq!(distance, 2.4f32 + 0.6f32 * (4. - 2.4f32));
        c.animation_enabled = false;
        c.camera_distance = 40.;
        distance = 8.;
        c.perspective(true, 1, 1, &mut last, &mut distance, &mut clip);
        assert_eq!(distance, 8.);
        c.animation_enabled = true;
        distance = 8.;
        last = 9.;
        c.perspective(true, 1, 0, &mut last, &mut distance, &mut clip);
        assert_eq!(last, 0.);
        assert_eq!(distance, 8.);
    }
}
