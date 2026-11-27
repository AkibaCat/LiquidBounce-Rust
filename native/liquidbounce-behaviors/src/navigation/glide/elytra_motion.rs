// SPDX-License-Identifier: GPL-3.0-or-later
//! Original one-tick ElytraMotion, including the game's float direction table.
use super::super::{
    flight_math, geometry::Vec3, simulation::sine_table, walk_jump::java_math::fdlibm_atan2,
};
pub fn update(delta: Vec3, y_rot: f32, x_rot: f32, boosted: bool) -> Vec3 {
    let y_angle = -y_rot * f32::from_bits(0x3c8e_fa35) - std::f32::consts::PI;
    let x_angle = -x_rot * f32::from_bits(0x3c8e_fa35);
    let y_cos = sine_table::cos(y_angle);
    let y_sin = sine_table::sin(y_angle);
    let x_cos = -sine_table::cos(x_angle);
    let look = Vec3::new(
        f64::from(y_sin * x_cos),
        f64::from(sine_table::sin(x_angle)),
        f64::from(y_cos * x_cos),
    );
    let lean = f64::from(x_rot) * 0.017453292519943295;
    let look_horizontal = (look.x * look.x + look.z * look.z).sqrt();
    let move_horizontal = (delta.x * delta.x + delta.z * delta.z).sqrt();
    let lift = flight_math::cos(lean) * flight_math::cos(lean);
    let mut x = delta.x;
    let mut y = delta.y + 0.08 * (-1.0 + lift * 0.75);
    let mut z = delta.z;
    if look_horizontal > 0.0 {
        if y < 0.0 {
            let convert = y * -0.1 * lift;
            x += look.x * convert / look_horizontal;
            y += convert;
            z += look.z * convert / look_horizontal;
        }
        if lean < 0.0 {
            let convert = move_horizontal * -flight_math::sin(lean) * 0.04;
            x -= look.x * convert / look_horizontal;
            y += convert * 3.2;
            z -= look.z * convert / look_horizontal;
        }
        x += (look.x / look_horizontal * move_horizontal - x) * 0.1;
        z += (look.z / look_horizontal * move_horizontal - z) * 0.1;
    }
    x *= f64::from(0.99_f32);
    y *= f64::from(0.98_f32);
    z *= f64::from(0.99_f32);
    if boosted {
        x += look.x * 0.1 + (look.x * 1.5 - x) * 0.5;
        y += look.y * 0.1 + (look.y * 1.5 - y) * 0.5;
        z += look.z * 0.1 + (look.z * 1.5 - z) * 0.5;
    }
    Vec3::new(x, y, z)
}
pub fn y_rot_towards(dx: f64, dz: f64) -> f32 {
    let mut wrapped = (fdlibm_atan2(-dx, dz) * 57.29577951308232) % 360.0;
    if wrapped.is_nan() {
        // HotSpot Math.atan2 returns its canonical positive NaN.
        return f32::NAN;
    }
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped as f32
}
pub fn x_rot_towards(up: f64, ahead: f64) -> f32 {
    (-(fdlibm_atan2(up, ahead) * 57.29577951308232)) as f32
}
