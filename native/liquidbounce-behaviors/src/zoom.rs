// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ModuleZoom.kt, Copyright (c) 2015-2026 CCBlueX.

use crate::numeric::{lerp_float, lerp_int, unit_clamp};
use crate::{Chronometer, Easing, MillisClock};

/// These ranges are the original UI metadata, not extra clamps. Upstream's
/// RangedValue.set does not enforce its metadata range, so loaded raw values
/// (including invalid duration factors) retain their original behavior here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoomSettings {
    pub zoom: i32,
    pub scroll_enabled: bool,
    pub scroll_speed: f32,
    pub transition: Easing,
    pub duration_factor: f32,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            zoom: 30,
            scroll_enabled: true,
            scroll_speed: 2.0,
            transition: Easing::QuadIn,
            duration_factor: 2.0,
        }
    }
}

impl ZoomSettings {
    pub const ZOOM_RANGE: (i32, i32) = (10, 150);
    pub const SCROLL_SPEED_RANGE: (f32, f32) = (0.5, 8.0);
    pub const DURATION_FACTOR_RANGE: (f32, f32) = (0.0, 10.0);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZoomContext {
    pub in_game: bool,
    /// Actual ClientModule.running, including its listener/world/enable gates.
    pub running: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoomSnapshot {
    pub target_fov: i32,
    pub previous_fov: i32,
    pub scaled_difference: f64,
    pub disable_animation_finished: bool,
    pub last_update_ms: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollOutcome {
    /// Cancel the original MouseScrollInHotbarEvent, including zero-step events.
    pub cancel_hotbar_scroll: bool,
}

#[derive(Clone, Debug)]
pub struct Zoom {
    pub settings: ZoomSettings,
    target_fov: i32,
    previous_fov: i32,
    scaled_difference: f64,
    disable_animation_finished: bool,
    chronometer: Chronometer,
}

impl Default for Zoom {
    fn default() -> Self {
        Self::new(ZoomSettings::default())
    }
}

impl Zoom {
    pub fn new(settings: ZoomSettings) -> Self {
        Self {
            settings,
            target_fov: 0,
            previous_fov: 0,
            scaled_difference: 0.0,
            disable_animation_finished: true,
            chronometer: Chronometer::default(),
        }
    }

    pub fn snapshot(&self) -> ZoomSnapshot {
        ZoomSnapshot {
            target_fov: self.target_fov,
            previous_fov: self.previous_fov,
            scaled_difference: self.scaled_difference,
            disable_animation_finished: self.disable_animation_finished,
            last_update_ms: self.chronometer.last_update(),
        }
    }

    /// Invoke at the original onEnabled point, before committing enabled=true.
    /// default_fov must already include NoFOV when that module is running.
    /// Does not change the external module enabled/running state.
    pub fn on_enabled(&mut self, default_fov: i32, clock: &mut impl MillisClock) {
        self.previous_fov = self.get_fov(false, default_fov, clock);
        self.target_fov = self.settings.zoom;
        self.reset(clock);
    }

    /// Invoke at the original onDisabled point, before committing enabled=false.
    /// Retain both upstream reset calls and their individual clock samples.
    pub fn on_disabled(&mut self, default_fov: i32, clock: &mut impl MillisClock) {
        self.on_disabled_with_default_fov(|| default_fov, clock);
    }

    /// Provider variant preserves the original default-FOV read position:
    /// after sampling the current animated FOV and the first clock reset.
    pub fn on_disabled_with_default_fov(
        &mut self,
        mut default_fov: impl FnMut() -> i32,
        clock: &mut impl MillisClock,
    ) {
        self.previous_fov = self.get_fov(true, 0, clock);
        self.chronometer.reset(clock);
        self.target_fov = default_fov();
        self.reset(clock);
        self.disable_animation_finished = false;
    }

    pub fn get_fov(&mut self, enabled: bool, original: i32, clock: &mut impl MillisClock) -> i32 {
        if !enabled && self.disable_animation_finished {
            return original;
        }
        let factor = self.progress(clock);
        if !enabled && factor == 1.0 {
            self.disable_animation_finished = true;
        }
        lerp_int(
            self.settings.transition.transform(factor),
            self.previous_fov,
            self.target_fov,
        )
    }

    pub fn apply_fractional_fov(
        &self,
        vanilla_fov: f32,
        context: ZoomContext,
        clock: &mut impl MillisClock,
    ) -> f32 {
        if !context.in_game || (!context.running && self.disable_animation_finished) {
            return vanilla_fov;
        }
        let eased = self.settings.transition.transform(self.progress(clock));
        let whole = lerp_int(eased, self.previous_fov, self.target_fov);
        if whole == 0 {
            return vanilla_fov;
        }
        vanilla_fov
            * (lerp_float(eased, self.previous_fov as f32, self.target_fov as f32) / whole as f32)
    }

    /// Feed the original integer hotbar wheel event, not raw OS scroll delta.
    /// running is the actual parent module running gate; Scroll's own setting
    /// supplies the child gate. No synthetic input is created here.
    pub fn on_hotbar_scroll(
        &mut self,
        steps: i32,
        running: bool,
        clock: &mut impl MillisClock,
    ) -> ScrollOutcome {
        if !running || !self.settings.scroll_enabled {
            return ScrollOutcome::default();
        }
        self.previous_fov = self.get_fov(true, 0, clock);
        // kotlin.math.round(Float) = Math.rint(double).toFloat(): ties to even.
        let amount = (steps as f32 * self.settings.scroll_speed).round_ties_even() as i32;
        self.target_fov = self.target_fov.wrapping_sub(amount).clamp(1, 179);
        self.reset(clock);
        ScrollOutcome {
            cancel_hotbar_scroll: true,
        }
    }

    pub fn progress(&self, clock: &mut impl MillisClock) -> f32 {
        if self.scaled_difference <= 0.0 || !self.scaled_difference.is_finite() {
            1.0
        } else {
            unit_clamp((self.chronometer.elapsed(clock) as f64 / self.scaled_difference) as f32)
        }
    }

    fn reset(&mut self, clock: &mut impl MillisClock) {
        self.chronometer.reset(clock);
        self.scaled_difference = self.settings.duration_factor as f64
            * self
                .target_fov
                .wrapping_sub(self.previous_fov)
                .wrapping_abs() as f64;
    }
}
