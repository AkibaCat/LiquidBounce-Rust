// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ModuleNoFov.kt, Copyright (c) 2015-2026 CCBlueX.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoFovMode {
    #[default]
    Constant,
    Custom,
}

impl NoFovMode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Constant => "Constant",
            Self::Custom => "Custom",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomFov {
    pub base_fov: f32,
    pub limit: (f32, f32),
    pub multiplier: f32,
}

impl Default for CustomFov {
    fn default() -> Self {
        Self {
            base_fov: 1.0,
            limit: (0.0, 1.5),
            multiplier: 1.0,
        }
    }
}

/// Represents Kotlin's IllegalArgumentException for an empty Float range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatRangeError {
    pub start: f32,
    pub end_inclusive: f32,
}

impl CustomFov {
    pub const BASE_FOV_RANGE: (f32, f32) = (0.0, 1.5);
    pub const LIMIT_RANGE: (f32, f32) = (0.0, 1.5);
    pub const MULTIPLIER_RANGE: (f32, f32) = (0.1, 1.5);

    pub fn get_fov_multiplier(&self, original: f32) -> Result<f32, FloatRangeError> {
        let new_fov = (original - 1.0) * self.multiplier + self.base_fov;
        let (start, end_inclusive) = self.limit;
        // ClosedFloatRange.isEmpty = !(start <= end); NaN endpoints are empty.
        if start.partial_cmp(&end_inclusive).is_none() || start > end_inclusive {
            return Err(FloatRangeError {
                start,
                end_inclusive,
            });
        }
        // coerceIn(ClosedFloatingPointRange) uses lessThanOrEquals in both
        // directions, so equal values retain their own signed zero.
        Ok(if new_fov < start {
            start
        } else if new_fov > end_inclusive {
            end_inclusive
        } else {
            new_fov
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoFov {
    pub mode: NoFovMode,
    pub constant_fov: i32,
    pub custom: CustomFov,
}

impl Default for NoFov {
    fn default() -> Self {
        Self {
            mode: NoFovMode::Constant,
            constant_fov: 90,
            custom: CustomFov::default(),
        }
    }
}

impl NoFov {
    pub const CONSTANT_FOV_RANGE: (i32, i32) = (1, 179);

    pub fn get_fov(&self, original: i32) -> i32 {
        match self.mode {
            NoFovMode::Constant => self.constant_fov,
            NoFovMode::Custom => original,
        }
    }

    pub fn get_fov_multiplier(&self, original: f32) -> Result<f32, FloatRangeError> {
        match self.mode {
            NoFovMode::Constant => Ok(1.0),
            NoFovMode::Custom => self.custom.get_fov_multiplier(original),
        }
    }
}
