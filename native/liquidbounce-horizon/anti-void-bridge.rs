// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::OnceLock;
static SHAPE: OnceLock<fn(&mut bool, &mut f64)> = OnceLock::new();
pub fn install(callback: fn(&mut bool, &mut f64)) {
    let _ = SHAPE.set(callback);
}
fn client_anti_void_shape(active: &mut bool, floor: &mut f64) {
    if let Some(callback) = SHAPE.get() {
        callback(active, floor);
    }
}
#[inline(never)]
pub fn native_client_anti_void_shape(active: &mut bool, floor: &mut f64) {
    client_anti_void_shape(active, floor);
}
pub fn ghost_floor() -> Option<f64> {
    let (mut active, mut floor) = (false, 0.);
    native_client_anti_void_shape(&mut active, &mut floor);
    active.then_some(floor)
}
