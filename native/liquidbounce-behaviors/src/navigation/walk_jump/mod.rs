// SPDX-License-Identifier: GPL-3.0-or-later
//! Original Agent jump planner. World access is explicit to preserve the
//! original interleaving between candidate enumeration and simulation.
pub mod airtime;
pub mod assisted_jump;
pub(crate) mod java_math;
pub mod jump;
pub mod jump_targets;
pub mod jumps;
mod landings;
pub mod leap;
pub mod run_up;

pub(crate) fn yaw(x: f64, z: f64) -> f32 {
    (java_math::fdlibm_atan2(z, x) * f64::from(f32::from_bits(0x4265_2ee0)) - 90.0) as f32
}
pub(crate) fn vector_yaw(x: f64, z: f64) -> f32 {
    (java_math::fdlibm_atan2(-x, z) as f32) * f32::from_bits(0x4265_2ee0)
}
pub(crate) fn wrap_degrees(angle: f32) -> f32 {
    let mut wrapped = angle % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}
