// SPDX-License-Identifier: GPL-3.0-or-later
//! Source Normal/Expand control decisions, independent of world queries.
pub fn normal_radius(freeze: bool, down: bool) -> i32 {
    if freeze {
        4
    } else if down {
        2
    } else {
        1
    }
}
pub fn reverse_rotation(yaw: f32, pitch: f32) -> [f32; 2] {
    [
        (yaw / 45.0).round_ties_even() * 45.0,
        if pitch < 45.0 { 45.0 } else { pitch },
    ]
}
/// Java/Kotlin's Float arithmetic and truncating conversion are significant at
/// diagonal headings. The caller supplies Kotlin sin/cos(Float) results.
pub fn expand_position(position: [f64; 3], xz: [f32; 2], step: i32) -> [i32; 3] {
    let [x, y, z] = position.map(|v| v.floor() as i32);
    [
        x.wrapping_add((xz[0] * step as f32) as i32),
        y,
        z.wrapping_add((xz[1] * step as f32) as i32),
    ]
}
/// Target height is resolved lazily by the callback. In Hypixel SameY the
/// resolution changes jumps, so candidates after a successful placement must
/// never resolve their heights.
pub fn find_expand<T>(
    position: [f64; 3],
    xz: [f32; 2],
    length: i32,
    mut find: impl FnMut([i32; 3]) -> Option<T>,
) -> Option<T> {
    for step in 0..=length {
        if let Some(target) = find(expand_position(position, xz, step)) {
            return Some(target);
        }
    }
    None
}
