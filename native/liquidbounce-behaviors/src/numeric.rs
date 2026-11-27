// SPDX-License-Identifier: GPL-3.0-or-later
// JVM-width arithmetic used by the LiquidBounce port. Mth 26.3 was inspected
// from the local Minecraft 26.3 deobfuscated JAR using javap -c.

/// Kotlin coerceIn(0F, 1F) preserves NaN and negative zero.
pub(crate) fn unit_clamp(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

/// Mth.lerpInt = start + floor(factor * (end - start)), not a cast of the
/// complete interpolated value. JVM Int arithmetic wraps; d2i saturates and
/// converts NaN to zero, matching Rust's float-to-integer conversion.
pub(crate) fn lerp_int(factor: f32, start: i32, end: i32) -> i32 {
    let displacement = factor * end.wrapping_sub(start) as f32;
    start.wrapping_add((displacement as f64).floor() as i32)
}

/// Keep the three JVM floating-point operations separate; do not use mul_add.
pub(crate) fn lerp_float(factor: f32, start: f32, end: f32) -> f32 {
    start + factor * (end - start)
}
