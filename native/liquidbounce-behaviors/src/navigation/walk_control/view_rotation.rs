// SPDX-License-Identifier: GPL-3.0-or-later
//! Exact common ViewRotation, including float angle and sine-table rounding.
use crate::navigation::{geometry::Vec3, simulation::sine_table, walk_jump::wrap_degrees};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewRotation {
    pub y_rot: f32,
    pub x_rot: f32,
}
impl ViewRotation {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const fn new(y_rot: f32, x_rot: f32) -> Self {
        Self { y_rot, x_rot }
    }
    pub fn view_vector(self) -> Vec3 {
        let x = wrap_degrees(self.x_rot) * 0.017453292_f32;
        let y = -wrap_degrees(self.y_rot) * 0.017453292_f32;
        let x_cos = sine_table::cos(x);
        Vec3::new(
            f64::from(sine_table::sin(y) * x_cos),
            f64::from(-sine_table::sin(x)),
            f64::from(sine_table::cos(y) * x_cos),
        )
    }
    pub fn from_vector(vector: Vec3) -> Self {
        let horizontal = (vector.x * vector.x + vector.z * vector.z).sqrt();
        // Agent's Double.toDegrees multiplies by its *float* RAD_TO_DEG.
        let rad_to_deg = f64::from(180.0_f32 / std::f32::consts::PI);
        let y_rot = (vector.z.atan2(vector.x) * rad_to_deg) as f32 - 90.0_f32;
        let x_rot = -((vector.y.atan2(horizontal) * rad_to_deg) as f32);
        Self::new(wrap_degrees(y_rot), wrap_degrees(x_rot))
    }
}
