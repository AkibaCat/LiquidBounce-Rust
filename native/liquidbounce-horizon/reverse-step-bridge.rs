// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual jump and final-key observations, suppressed in prediction by caller.
use crate::reverse_step::ReverseStep;
use std::cell::{Cell, RefCell};
thread_local! {
 static STATE:RefCell<ReverseStep>=RefCell::new(ReverseStep::default());
 static KEYS:Cell<[bool;7]>=const{Cell::new([false;7])};
}
pub fn jump() {
    STATE.with(|s| s.borrow_mut().jump());
}
pub fn observe(running: bool, vertical: f64, on_ground: bool) -> ReverseStep {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.observe(running, vertical, on_ground);
        *s
    })
}
pub fn set_keys(keys: [bool; 7]) {
    KEYS.set(keys);
}
pub fn keys() -> [bool; 7] {
    KEYS.get()
}
pub fn clear() {
    KEYS.set([false; 7]);
    STATE.with(|s| *s.borrow_mut() = ReverseStep::default());
}
