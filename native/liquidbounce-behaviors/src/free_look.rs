// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396 ModuleFreeLook.kt, RotationUtil.kt and
// MixinCamera.java, Copyright (c) 2015-2026 CCBlueX.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PerspectiveChoice {
    Front,
    #[default]
    Back,
}
impl PerspectiveChoice {
    pub const ALL: [Self; 2] = [Self::Front, Self::Back];
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Front => "Front",
            Self::Back => "Back",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|choice| choice.tag().eq_ignore_ascii_case(tag))
    }
    pub const fn code(self) -> u8 {
        match self {
            Self::Front => 2,
            Self::Back => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeLook {
    pub perspective: PerspectiveChoice,
    pub sense_boost: f32,
    pub no_pitch_limit: bool,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
}
impl Default for FreeLook {
    fn default() -> Self {
        Self {
            perspective: PerspectiveChoice::Back,
            sense_boost: 1.0,
            no_pitch_limit: true,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
        }
    }
}
impl FreeLook {
    pub const DISABLE_ON_QUIT: bool = true;
    pub const HOLD_BIND: bool = true;
    pub const SENSE_BOOST_RANGE: (f32, f32) = (0.1, 2.0);
    /// ClientModule.onEnabled reads the player's current angles immediately;
    /// reconfiguration while enabled must not reset this accumulated camera.
    pub fn on_enabled(&mut self, player_rotation: [f32; 2]) {
        [self.camera_yaw, self.camera_pitch] = player_rotation;
    }
    pub fn perspective(self, running: bool, original: u8) -> u8 {
        if running {
            self.perspective.code()
        } else {
            original
        }
    }
    /// MouseHandler has already applied sensitivity, inversion and zoom.
    /// True cancels only its final Entity.turn; other event listeners still run.
    pub fn mouse_rotation(&mut self, running: bool, cursor_x: f64, cursor_y: f64) -> bool {
        if !running {
            return false;
        }
        // Modern RotationUtil narrows the cursor delta BEFORE multiplying.
        // Keep its two Float multiplications separate, without clamping imports.
        let delta_yaw = cursor_x as f32 * 0.15f32;
        let delta_pitch = cursor_y as f32 * 0.15f32;
        self.camera_yaw += delta_yaw * self.sense_boost;
        self.camera_pitch += delta_pitch * self.sense_boost;
        if !self.no_pitch_limit {
            self.camera_pitch = self.camera_pitch.clamp(-90.0, 90.0);
        }
        true
    }
    /// MixinCamera sets these final angles and cancels the remaining alignment.
    pub fn camera_rotation(self, running: bool) -> Option<[f32; 2]> {
        running.then(|| match self.perspective {
            PerspectiveChoice::Front => [self.camera_yaw + 180.0, -self.camera_pitch],
            PerspectiveChoice::Back => [self.camera_yaw, self.camera_pitch],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_enable_copies_player_angles_and_settings_edits_preserve_them() {
        let mut module = FreeLook::default();
        module.on_enabled([725.0, -100.0]);
        assert_eq!(module.camera_rotation(true), Some([725.0, -100.0]));
        module.perspective = PerspectiveChoice::Front;
        module.sense_boost = 0.5;
        assert_eq!(module.camera_rotation(true), Some([905.0, 100.0]));
        assert!(module.mouse_rotation(true, 20.0, 40.0));
        assert_eq!([module.camera_yaw, module.camera_pitch], [726.5, -97.0]);
        module.on_enabled([-45.0, 30.0]);
        assert_eq!(module.camera_rotation(true), Some([135.0, -30.0]));
    }
    #[test]
    fn running_gates_events_without_touching_raw_perspective_or_player_rotation() {
        let mut module = FreeLook::default();
        module.on_enabled([10.0, 20.0]);
        let unchanged = module;
        assert!(!module.mouse_rotation(false, 300.0, 800.0));
        assert_eq!(module, unchanged);
        assert_eq!(module.camera_rotation(false), None);
        for raw in 0..=2 {
            assert_eq!(module.perspective(false, raw), raw);
            assert_eq!(module.perspective(true, raw), 1);
        }
        module.perspective = PerspectiveChoice::Front;
        assert_eq!(module.perspective(true, 0), 2);
    }
    #[test]
    fn pitch_limit_is_applied_only_on_mouse_input_and_imports_keep_their_values() {
        let mut module = FreeLook {
            no_pitch_limit: false,
            sense_boost: 4.0,
            ..Default::default()
        };
        module.on_enabled([0.0, 500.0]);
        assert_eq!(module.camera_pitch, 500.0);
        module.mouse_rotation(true, 20.0, 0.0);
        assert_eq!([module.camera_yaw, module.camera_pitch], [12.0, 90.0]);
        module.mouse_rotation(true, 0.0, -1000.0);
        assert_eq!(module.camera_pitch, -90.0);
        module.no_pitch_limit = true;
        module.mouse_rotation(true, 0.0, -1000.0);
        assert_eq!(module.camera_pitch, -690.0);
    }
}
