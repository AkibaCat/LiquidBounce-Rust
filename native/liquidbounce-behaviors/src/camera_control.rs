// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396 ModuleAspect.kt, ModuleSmoothCamera.kt,
// ModuleQuickPerspectiveSwap.kt and MixinCamera.java, CCBlueX 2015-2026.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Perspective {
    #[default]
    FirstPerson,
    ThirdPersonBack,
    ThirdPersonFront,
}
impl Perspective {
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::FirstPerson),
            1 => Some(Self::ThirdPersonBack),
            2 => Some(Self::ThirdPersonFront),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aspect {
    pub ratio: i32,
}
impl Default for Aspect {
    fn default() -> Self {
        Self { ratio: 100 }
    }
}
impl Aspect {
    pub const RATIO_RANGE: (i32, i32) = (1, 300);
    /// Raw imported Int values retain the original getter's float conversion.
    pub fn ratio_multiplier(self) -> f32 {
        self.ratio as f32 / 100.0
    }
    pub fn percentage(self, running: bool) -> i32 {
        if running { self.ratio } else { 100 }
    }
    pub fn projection_width(self, running: bool, width: f32) -> f32 {
        if running {
            width / self.ratio_multiplier()
        } else {
            width
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmoothCamera {
    pub enable_first_pov: bool,
    pub reset_on_perspective_change: bool,
    pub horizontal_factor: f32,
    pub vertical_factor: f32,
    pub smooth_pos: [f64; 3],
    pub last_perspective: Perspective,
}
impl Default for SmoothCamera {
    fn default() -> Self {
        Self {
            enable_first_pov: false,
            reset_on_perspective_change: true,
            horizontal_factor: 0.9,
            vertical_factor: 0.93,
            smooth_pos: [0.0; 3],
            last_perspective: Perspective::FirstPerson,
        }
    }
}
impl SmoothCamera {
    pub const FACTOR_RANGE: (f32, f32) = (0.0, 1.0);
    pub fn on_disabled(&mut self) {
        self.smooth_pos = [0.0; 3];
    }
    /// Called at Camera.setPosition, before detached collision clipping. The
    /// perspective is the stored vanilla option, exactly as ModuleSmoothCamera
    /// reads it; temporary PerspectiveEvent overrides do not change this option.
    pub fn camera_update(
        &mut self,
        running: bool,
        free_look: bool,
        perspective: Perspective,
        pos: [f64; 3],
    ) -> [f64; 3] {
        // MixinCamera exits before cameraUpdate when FreeLook owns the camera.
        if free_look {
            return pos;
        }
        if !running {
            self.last_perspective = perspective;
            return pos;
        }
        if self.reset_on_perspective_change && self.last_perspective != perspective {
            self.smooth_pos = pos;
            self.last_perspective = perspective;
            return self.smooth_pos;
        }
        self.last_perspective = perspective;
        if !self.enable_first_pov && perspective == Perspective::FirstPerson {
            self.smooth_pos = pos;
            return self.smooth_pos;
        }
        // Vec3.isLikelyZero -> Mth.equal(lengthSqr, 0D). Mth's double
        // overload uses the promoted 1e-5f constant, not an exact 1e-5 double.
        let squared = self.smooth_pos[0] * self.smooth_pos[0]
            + self.smooth_pos[1] * self.smooth_pos[1]
            + self.smooth_pos[2] * self.smooth_pos[2];
        if squared.abs() < f64::from(1.0e-5f32) {
            self.smooth_pos = pos;
        }
        for (axis, value) in pos.into_iter().enumerate() {
            let factor = if axis == 1 {
                self.vertical_factor
            } else {
                self.horizontal_factor
            };
            // Kotlin evaluates 1 - factor as Float, then promotes each product
            // to Double because Vec3 components are Double. Do not fuse them.
            self.smooth_pos[axis] =
                self.smooth_pos[axis] * f64::from(factor) + value * f64::from(1.0f32 - factor);
        }
        self.smooth_pos
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuickPerspectiveSwap {
    pub rear_view: bool,
}
impl QuickPerspectiveSwap {
    pub const DISABLE_ON_QUIT: bool = true;
    pub const HOLD_BIND: bool = true;
    pub fn perspective(
        self,
        running: bool,
        free_look: bool,
        free_cam: bool,
        original: Perspective,
    ) -> Perspective {
        if !running || free_look || free_cam {
            return original;
        }
        match original {
            Perspective::FirstPerson => {
                if self.rear_view {
                    Perspective::ThirdPersonBack
                } else {
                    Perspective::ThirdPersonFront
                }
            }
            Perspective::ThirdPersonBack => Perspective::ThirdPersonFront,
            Perspective::ThirdPersonFront => Perspective::ThirdPersonBack,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aspect_uses_original_float_arithmetic_without_import_clamping() {
        let aspect = Aspect { ratio: 150 };
        assert_eq!(aspect.projection_width(true, 1920.0), 1280.0);
        assert_eq!(aspect.projection_width(false, 1920.0), 1920.0);
        assert_eq!(
            Aspect { ratio: 0 }.projection_width(true, 1.0),
            f32::INFINITY
        );
        assert_eq!(Aspect { ratio: -100 }.projection_width(true, 1.0), -1.0);
    }
    #[test]
    fn smoothing_has_source_defaults_and_separate_horizontal_vertical_weights() {
        let mut camera = SmoothCamera::default();
        assert_eq!(
            camera.camera_update(true, false, Perspective::ThirdPersonBack, [10., 20., 30.]),
            [10., 20., 30.]
        );
        let result =
            camera.camera_update(true, false, Perspective::ThirdPersonBack, [20., 40., 60.]);
        assert_eq!(
            result[0],
            10. * f64::from(0.9f32) + 20. * f64::from(1.0f32 - 0.9f32)
        );
        assert_eq!(
            result[1],
            20. * f64::from(0.93f32) + 40. * f64::from(1.0f32 - 0.93f32)
        );
        assert!(result[1] < 22.0);
    }
    #[test]
    fn first_person_perspective_reset_disable_and_freelook_follow_source() {
        let mut camera = SmoothCamera::default();
        let pos = [1., 2., 3.];
        assert_eq!(
            camera.camera_update(true, false, Perspective::FirstPerson, pos),
            pos
        );
        assert_eq!(
            camera.camera_update(true, false, Perspective::ThirdPersonFront, [10.; 3]),
            [10.; 3]
        );
        let snapshot = camera;
        assert_eq!(
            camera.camera_update(true, true, Perspective::FirstPerson, [8.; 3]),
            [8.; 3]
        );
        assert_eq!(camera, snapshot);
        camera.on_disabled();
        assert_eq!(camera.smooth_pos, [0.; 3]);
        camera.camera_update(false, false, Perspective::FirstPerson, pos);
        assert_eq!(camera.last_perspective, Perspective::FirstPerson);
        assert_eq!(camera.smooth_pos, [0.; 3]);
    }
    #[test]
    fn zero_sentinel_and_factor_endpoints_preserve_near_origin_behavior() {
        let mut camera = SmoothCamera {
            enable_first_pov: true,
            horizontal_factor: 1.0,
            vertical_factor: 0.0,
            ..Default::default()
        };
        assert_eq!(
            camera.camera_update(true, false, Perspective::FirstPerson, [1.; 3]),
            [1.; 3]
        );
        assert_eq!(
            camera.camera_update(true, false, Perspective::FirstPerson, [2.; 3]),
            [1., 2., 1.]
        );
        camera.smooth_pos = [0.001; 3];
        assert_eq!(
            camera.camera_update(true, false, Perspective::FirstPerson, [9.; 3]),
            [9.; 3]
        );
    }
    #[test]
    fn hold_swap_reads_current_vanilla_choice_without_changing_it() {
        let normal = QuickPerspectiveSwap::default();
        let rear = QuickPerspectiveSwap { rear_view: true };
        assert_eq!(
            normal.perspective(true, false, false, Perspective::FirstPerson),
            Perspective::ThirdPersonFront
        );
        assert_eq!(
            rear.perspective(true, false, false, Perspective::FirstPerson),
            Perspective::ThirdPersonBack
        );
        for original in [
            Perspective::FirstPerson,
            Perspective::ThirdPersonBack,
            Perspective::ThirdPersonFront,
        ] {
            assert_eq!(normal.perspective(false, false, false, original), original);
            assert_eq!(normal.perspective(true, true, false, original), original);
            assert_eq!(normal.perspective(true, false, true, original), original);
        }
        assert_eq!(
            normal.perspective(true, false, false, Perspective::ThirdPersonBack),
            Perspective::ThirdPersonFront
        );
        assert_eq!(
            normal.perspective(true, false, false, Perspective::ThirdPersonFront),
            Perspective::ThirdPersonBack
        );
    }
}
