// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ModuleFullBright.kt and ModeValueGroup.kt,
// Copyright (c) 2015-2026 CCBlueX.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FullBrightMode {
    #[default]
    Gamma,
    NightVision,
}

impl FullBrightMode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Gamma => "Gamma",
            Self::NightVision => "NightVision",
        }
    }
}

/// Apply these to the actual local player. The original two-argument
/// MobEffectInstance constructor gives amplifier=0, ambient=false,
/// visible=true and showIcon=true. Removal intentionally does not restore a
/// preexisting legitimate Night Vision effect, matching upstream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NightVisionIntent {
    Apply {
        duration_ticks: i32,
        amplifier: i32,
        ambient: bool,
        visible: bool,
        show_icon: bool,
    },
    Remove,
}

impl NightVisionIntent {
    pub const REAPPLY: Self = Self::Apply {
        duration_ticks: 1337,
        amplifier: 0,
        ambient: false,
        visible: true,
        show_icon: true,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GammaBrightness {
    /// Original UI metadata: 1..15, default 15. No extra runtime clamp.
    pub brightness: i32,
    gamma: f32,
}

impl Default for GammaBrightness {
    fn default() -> Self {
        Self {
            brightness: 15,
            gamma: 0.0,
        }
    }
}

impl GammaBrightness {
    pub const BRIGHTNESS_RANGE: (i32, i32) = (1, 15);

    pub const fn gamma(&self) -> f32 {
        self.gamma
    }

    pub fn enable(&mut self, vanilla_option_gamma: f64) {
        self.gamma = vanilla_option_gamma as f32;
    }

    /// Call only for an accepted PlayerPostTickEvent with this mode running.
    pub fn on_player_post_tick(&mut self) {
        if self.gamma < self.brightness as f32 {
            // Kotlin coerceAtMost preserves NaN rather than f32::min's
            // number-preferred NaN behavior.
            let next = self.gamma + 0.1_f32;
            self.gamma = if next > self.brightness as f32 {
                self.brightness as f32
            } else {
                next
            };
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FullBright {
    pub gamma: GammaBrightness,
    mode: FullBrightMode,
}

impl FullBright {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn mode(&self) -> FullBrightMode {
        self.mode
    }

    /// Mirrors the selected child's enable(), not the general module runtime.
    pub fn on_enabled(&mut self, vanilla_option_gamma: f64) {
        if self.mode == FullBrightMode::Gamma {
            self.gamma.enable(vanilla_option_gamma);
        }
    }

    /// The parent child-state update invokes disable even if not running.
    pub fn on_disabled(&mut self) -> Option<NightVisionIntent> {
        match self.mode {
            FullBrightMode::Gamma => None,
            FullBrightMode::NightVision => Some(NightVisionIntent::Remove),
        }
    }

    /// Synchronous sink preserves old-mode removal before active mode commit.
    /// parent_running must come from the actual module listener/world gates.
    /// General ValueChanged events/config listeners belong to the bridge.
    pub fn switch_mode(
        &mut self,
        next: FullBrightMode,
        parent_running: bool,
        vanilla_option_gamma: f64,
        mut emit: impl FnMut(NightVisionIntent),
    ) {
        if self.mode == next {
            return;
        }
        if parent_running && let Some(intent) = self.on_disabled() {
            emit(intent);
        }
        self.mode = next;
        if parent_running {
            self.on_enabled(vanilla_option_gamma);
        }
    }

    pub fn on_player_post_tick(&mut self, parent_running: bool) -> Option<NightVisionIntent> {
        if !parent_running {
            return None;
        }
        match self.mode {
            FullBrightMode::Gamma => {
                self.gamma.on_player_post_tick();
                None
            }
            FullBrightMode::NightVision => Some(NightVisionIntent::REAPPLY),
        }
    }

    /// Override brightnessOption at the original lightmap extraction point.
    /// Return None so the caller can preserve vanilla or subsequent XRay logic.
    pub fn brightness_override(&self, parent_running: bool) -> Option<f32> {
        (parent_running && self.mode == FullBrightMode::Gamma).then_some(self.gamma.gamma())
    }
}
