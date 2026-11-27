// SPDX-License-Identifier: GPL-3.0-or-later
/// Vec3.normalize's tiny-vector rule and the source's fused eye.fma.
pub fn target(eye: [f64; 3], hit: [f64; 3], distance: f32) -> [f64; 3] {
    let delta = std::array::from_fn::<_, 3, _>(|i| hit[i] - eye[i]);
    let length = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
    let direction = if length < 1.0e-5 {
        [0.; 3]
    } else {
        delta.map(|v| v / length)
    };
    std::array::from_fn(|i| f64::from(distance).mul_add(direction[i], eye[i]))
}
