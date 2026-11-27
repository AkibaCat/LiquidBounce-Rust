// SPDX-License-Identifier: GPL-3.0-or-later
// Synchronous real local PlayerJumpEvent HEAD; never reads a post-tick position.
static JUMP: std::sync::OnceLock<fn([f64; 3])> = std::sync::OnceLock::new();
pub fn install(callback: fn([f64; 3])) {
    let _ = JUMP.set(callback);
}
fn client_jump_effect_jump(position: [f64; 3]) {
    if let Some(callback) = JUMP.get() {
        callback(position);
    }
}
#[inline(never)]
pub fn native_client_jump_effect_jump(position: [f64; 3]) {
    client_jump_effect_jump(position);
}
pub fn jump(position: [f64; 3]) {
    native_client_jump_effect_jump(position);
}
