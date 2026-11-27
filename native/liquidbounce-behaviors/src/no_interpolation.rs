// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleNoInterpolation.kt at LiquidBounce ae0eb396, CCBlueX 2015-2026.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoInterpolation {
    pub reduce_by: i32,
}

impl Default for NoInterpolation {
    fn default() -> Self {
        Self { reduce_by: 3 }
    }
}

impl NoInterpolation {
    /// Kotlin Int subtraction wraps before kotlin.math.max. Config imports
    /// preserve the original raw value rather than clamping the UI range.
    pub fn steps(self, running: bool, original: i32) -> i32 {
        if running {
            original.wrapping_sub(self.reduce_by).max(0)
        } else {
            original
        }
    }
}
