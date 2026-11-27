// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce Easing.kt, Copyright (c) 2015-2026 CCBlueX.
// The original identifies https://easings.net as its formula source.

use crate::numeric::unit_clamp;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    Linear,
    #[default]
    QuadIn,
    QuadOut,
    QuadInOut,
    ExponentialIn,
    ExponentialOut,
    None,
}

impl Easing {
    pub const ALL: [Self; 7] = [
        Self::Linear,
        Self::QuadIn,
        Self::QuadOut,
        Self::QuadInOut,
        Self::ExponentialIn,
        Self::ExponentialOut,
        Self::None,
    ];

    pub const fn tag(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::QuadIn => "QuadIn",
            Self::QuadOut => "QuadOut",
            Self::QuadInOut => "QuadInOut",
            Self::ExponentialIn => "ExponentialIn",
            Self::ExponentialOut => "ExponentialOut",
            Self::None => "None",
        }
    }

    /// Match the original expressions and f32 operation order. Float.pow in
    /// Kotlin promotes both operands to double before calling Math.pow, then
    /// converts back to float; native f32::powf would change that behavior.
    pub fn transform(self, x: f32) -> f32 {
        match self {
            Self::Linear => x,
            Self::QuadIn => x * x,
            Self::QuadOut => 1.0 - (1.0 - x) * (1.0 - x),
            Self::QuadInOut => 2.0 * (1.0 - x) * x * x + x * x,
            Self::ExponentialIn => {
                if x == 0.0 {
                    0.0
                } else {
                    2.0_f64.powf((10.0_f32 * x - 10.0) as f64) as f32
                }
            }
            Self::ExponentialOut => {
                if x == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f64.powf((-10.0_f32 * x) as f64) as f32
                }
            }
            Self::None => 1.0,
        }
    }

    pub fn get_factor(self, start_time: i64, current_time: i64, time: f32) -> f32 {
        if self == Self::None {
            return 1.0;
        }
        let delta = current_time.wrapping_sub(start_time);
        self.transform(unit_clamp((delta as f64 / time as f64) as f32))
    }
}
