// SPDX-License-Identifier: GPL-3.0-or-later
//! Original Agent Mth.SIN table, pinned from the original artifact/JDK25.
//! See collect-simulation-sine-table.py and its source/table hash report.
const TABLE: &[u8; 262144] = include_bytes!("sine-table.bin");
fn at(index: usize) -> f32 {
    let start = index * 4;
    f32::from_le_bytes(
        TABLE[start..start + 4]
            .try_into()
            .expect("fixed sine table entry"),
    )
}
pub fn sin(angle: f32) -> f32 {
    at(((f64::from(angle) * 10430.378350470453) as i64 & 0xffff) as usize)
}
pub fn cos(angle: f32) -> f32 {
    at(((f64::from(angle) * 10430.378350470453 + 16384.0) as i64 & 0xffff) as usize)
}
pub fn sin_1_8(angle: f32) -> f32 {
    at(((angle * 10430.378_f32) as i32 & 0xffff) as usize)
}
pub fn cos_1_8(angle: f32) -> f32 {
    at(((angle * 10430.378_f32 + 16384.0_f32) as i32 & 0xffff) as usize)
}
pub fn radians(yaw: f32) -> f32 {
    yaw * 0.017453292_f32
}
