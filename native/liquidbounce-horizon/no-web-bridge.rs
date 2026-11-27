// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::OnceLock;
type Collision = fn([i32; 3], bool, bool, i32, &mut String);
static COLLISION: OnceLock<Collision> = OnceLock::new();
pub fn install(callback: Collision) {
    let _ = COLLISION.set(callback);
}
pub fn collision(pos: [i32; 3], moving: bool, ground: bool, tick: i32) -> String {
    let mut result = String::new();
    native_client_no_web_collision(pos, moving, ground, tick, &mut result);
    result
}
fn client_no_web_collision(
    pos: [i32; 3],
    moving: bool,
    ground: bool,
    tick: i32,
    result: &mut String,
) {
    if let Some(f) = COLLISION.get() {
        f(pos, moving, ground, tick, result);
    }
}
#[inline(never)]
fn native_client_no_web_collision(
    pos: [i32; 3],
    moving: bool,
    ground: bool,
    tick: i32,
    result: &mut String,
) {
    client_no_web_collision(pos, moving, ground, tick, result);
}
