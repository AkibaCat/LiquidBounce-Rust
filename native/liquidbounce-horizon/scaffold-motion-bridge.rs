// SPDX-License-Identifier: GPL-3.0-or-later
//! Synchronous final PlayerJumpEvent and PlayerAfterJumpEvent boundaries.
use std::sync::OnceLock;
type Jump = fn(f64, f32);
type After = fn(f32, bool, bool, bool, bool, &mut f64, &mut f64, &mut f64);
static JUMP: OnceLock<Jump> = OnceLock::new();
static AFTER: OnceLock<After> = OnceLock::new();
pub fn install(jump: Jump, after: After) {
    let _ = JUMP.set(jump);
    let _ = AFTER.set(after);
}
#[inline(never)]
fn client_scaffold_jump(y: f64, motion: f32) {
    if let Some(f) = JUMP.get() {
        f(y, motion);
    }
}
#[inline(never)]
fn client_scaffold_after_jump(
    yaw: f32,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    x: &mut f64,
    y: &mut f64,
    z: &mut f64,
) {
    if let Some(f) = AFTER.get() {
        f(yaw, forward, backward, left, right, x, y, z);
    }
}
#[inline(never)]
pub fn native_client_scaffold_jump(y: f64, motion: f32) {
    client_scaffold_jump(y, motion);
}
#[inline(never)]
pub fn native_client_scaffold_after_jump(
    yaw: f32,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    x: &mut f64,
    y: &mut f64,
    z: &mut f64,
) {
    client_scaffold_after_jump(yaw, forward, backward, left, right, x, y, z);
}
pub fn jump(y: f64, motion: f32) {
    native_client_scaffold_jump(y, motion);
}
pub fn after_jump(yaw: f32, velocity: &mut horizon_core::geometry::Vec3) {
    let k = crate::reverse_step_bridge::keys();
    native_client_scaffold_after_jump(
        yaw,
        k[0],
        k[1],
        k[2],
        k[3],
        &mut velocity.x,
        &mut velocity.y,
        &mut velocity.z,
    );
}
