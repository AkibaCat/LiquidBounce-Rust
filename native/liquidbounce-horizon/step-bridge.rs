// SPDX-License-Identifier: GPL-3.0-or-later
//! Synchronous Step callbacks run while the real MotionWorld owns its body.
use std::{
    cell::Cell,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};
static HEIGHT: OnceLock<fn(&mut f32)> = OnceLock::new();
static SUCCESS: OnceLock<fn(f64, f64, &mut String)> = OnceLock::new();
static LEGIT: AtomicBool = AtomicBool::new(false);
static SPOOF: AtomicBool = AtomicBool::new(false);
thread_local! {static AIR_TICKS:Cell<i32>=const{Cell::new(0)};}
pub fn install(height: fn(&mut f32), success: fn(f64, f64, &mut String)) {
    let _ = HEIGHT.set(height);
    let _ = SUCCESS.set(success);
}
pub fn set_flags(legit: bool, spoof: bool) {
    LEGIT.store(legit, Ordering::Release);
    SPOOF.store(spoof, Ordering::Release);
}
pub fn legit() -> bool {
    LEGIT.load(Ordering::Acquire)
}
pub fn moved(ground: bool) {
    AIR_TICKS.with(|v| v.set(if ground { 0 } else { v.get().wrapping_add(1) }));
}
pub fn ground(original: bool) -> bool {
    original || SPOOF.load(Ordering::Acquire) && AIR_TICKS.with(|v| v.get() == 8)
}
pub fn clear() {
    set_flags(false, false);
    AIR_TICKS.with(|v| v.set(0));
}
fn client_step_height(height: &mut f32) {
    if let Some(f) = HEIGHT.get() {
        f(height);
    }
}
#[inline(never)]
pub fn native_client_step_height(height: &mut f32) {
    client_step_height(height);
}
pub fn height(height: &mut f32) {
    native_client_step_height(height);
}
fn client_step_success(adjusted_y: f64, player_y: f64, result: &mut String) {
    if let Some(f) = SUCCESS.get() {
        f(adjusted_y, player_y, result);
    }
}
#[inline(never)]
pub fn native_client_step_success(adjusted_y: f64, player_y: f64, result: &mut String) {
    client_step_success(adjusted_y, player_y, result);
}
pub fn success(adjusted_y: f64, player_y: f64) -> String {
    let mut v = String::new();
    native_client_step_success(adjusted_y, player_y, &mut v);
    v
}
