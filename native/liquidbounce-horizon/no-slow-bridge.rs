// SPDX-License-Identifier: GPL-3.0-or-later
pub use horizon_world::no_slow_policy::{configure, get, running};
use std::cell::{Cell, RefCell};
thread_local! {static LIVE:RefCell<Option<[f64;15]>>=const{RefCell::new(None)};static UNTRACKED:Cell<u32>=const{Cell::new(0)};static RECURSION:Cell<u32>=const{Cell::new(0)};}
pub fn context() -> Option<[f64; 15]> {
    LIVE.with(|v| *v.borrow())
}
pub fn set_context(value: Option<[f64; 15]>) {
    LIVE.with(|v| *v.borrow_mut() = value);
}
pub fn untracked() -> bool {
    UNTRACKED.with(|v| v.get() != 0)
}
pub fn without_tracking<T>(f: impl FnOnce() -> T) -> T {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            UNTRACKED.with(|v| v.set(v.get() - 1));
        }
    }
    UNTRACKED.with(|v| v.set(v.get() + 1));
    let _guard = Guard;
    f()
}
// The original nested Intave send reaches the same packet listener. End the
// recursion after the replacement reaches its fixed point (block centre).
pub fn intave_nested<T>(f: impl FnOnce() -> T) -> Option<T> {
    if RECURSION.with(|v| v.get() != 0) {
        return None;
    }
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            RECURSION.with(|v| v.set(v.get() - 1));
        }
    }
    RECURSION.with(|v| v.set(v.get() + 1));
    let _guard = Guard;
    Some(f())
}
pub fn modify_input(
    input: [f32; 2],
    item: Option<f32>,
    sneak: Option<f32>,
    shift: bool,
    owner: bool,
) -> [f32; 2] {
    if !owner {
        return crate::input::modify_input(input, item, sneak);
    }
    let p = get();
    if p[0] != 1. {
        return crate::input::modify_input(input, item, sneak);
    }
    let sneak = sneak.map(|v| {
        if shift && p[1] >= 0. {
            v.max(p[1] as f32)
        } else {
            v
        }
    });
    crate::input::modify_input_axes(
        input,
        item.map(|v| {
            [
                if p[10] >= 0. { p[10] as f32 } else { v },
                if p[9] >= 0. { p[9] as f32 } else { v },
            ]
        }),
        sneak,
    )
}
