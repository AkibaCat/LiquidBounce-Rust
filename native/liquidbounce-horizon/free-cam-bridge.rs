// SPDX-License-Identifier: GPL-3.0-or-later
//! Local-player policy seam, usable while physics owns the player.
use std::sync::OnceLock;
static QUERY: OnceLock<fn(u8, f64, f64, &mut u32)> = OnceLock::new();
pub fn install(callback: fn(u8, f64, f64, &mut u32)) {
    let _ = QUERY.set(callback);
}
fn client_free_cam_policy(kind: u8, x: f64, z: f64, flags: &mut u32) {
    if let Some(callback) = QUERY.get() {
        callback(kind, x, z, flags);
    }
}
#[inline(never)]
pub fn native_client_free_cam_policy(kind: u8, x: f64, z: f64, flags: &mut u32) {
    client_free_cam_policy(kind, x, z, flags);
}
pub fn flags() -> u32 {
    let mut flags = 0;
    native_client_free_cam_policy(0, 0., 0., &mut flags);
    flags
}
pub fn active() -> bool {
    flags() & 1 != 0
}
pub fn allow_rotation() -> bool {
    let flags = flags();
    flags & 1 == 0 || flags & 4 != 0
}
pub fn moved(x: f64, z: f64) {
    native_client_free_cam_policy(1, x, z, &mut 0);
}
