// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleExtendedFirework and MixinFireworkRocketEntity, ae0eb396.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Custom,
}
impl Mode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Custom => "Custom",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "Normal" => Some(Self::Normal),
            "Custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

pub const NORMAL_MULTIPLIER: [f64; 3] = [0.1, 1.65, 0.5];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtendedFirework {
    pub mode: Mode,
    pub custom: [f64; 3],
}
impl Default for ExtendedFirework {
    fn default() -> Self {
        Self {
            mode: Mode::Normal,
            custom: NORMAL_MULTIPLIER,
        }
    }
}
impl ExtendedFirework {
    pub fn multiplier(&self) -> [f64; 3] {
        match self.mode {
            Mode::Normal => NORMAL_MULTIPLIER,
            Mode::Custom => self.custom,
        }
    }
    /// ModifyArgs replaces the three add arguments. The original movement
    /// must be added only after this expression, preserving JVM rounding.
    pub fn impulse(&self, look: [f64; 3], movement: [f64; 3]) -> [f64; 3] {
        let [acceleration, target, convergence] = self.multiplier();
        std::array::from_fn(|i| {
            look[i] * acceleration + (look[i] * target - movement[i]) * convergence
        })
    }
}

/// Rotation.directionVector uses Vec3.directionFromRotation, which differs
/// from Entity.calculateViewVector in its Float angle/PI arithmetic.
pub fn direction_from_rotation(
    pitch: f32,
    yaw: f32,
    mut sin_cos: impl FnMut(f64) -> [f32; 2],
) -> [f64; 3] {
    let [sy, cy] = sin_cos(f64::from(
        -yaw * (std::f32::consts::PI / 180.0) - std::f32::consts::PI,
    ));
    let [sp, cp] = sin_cos(f64::from(-pitch * (std::f32::consts::PI / 180.0)));
    [f64::from(sy * -cp), f64::from(sp), f64::from(cy * -cp)]
}
