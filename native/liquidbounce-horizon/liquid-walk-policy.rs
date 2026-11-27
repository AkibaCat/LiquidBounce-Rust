// SPDX-License-Identifier: GPL-3.0-or-later
//! Global BlockShapeEvent policy with actual local-player state on its thread.
use crate::liquid_walk_model::{self as model, Context, Mode};
use horizon_core::geometry::VoxelShape;
use std::{
    cell::Cell,
    sync::atomic::{AtomicU8, Ordering},
};
static MODE: AtomicU8 = AtomicU8::new(0);
thread_local! {
 static SHIFT:Cell<bool>=const{Cell::new(false)};
 static SHAPE:Cell<bool>=const{Cell::new(false)};
 static CONTEXT:Cell<Context>=const{Cell::new([false;4])};
 static IN_WATER:Cell<bool>=const{Cell::new(false)};
}
pub fn set_mode(code: u8) {
    MODE.store(code, Ordering::Release);
}
pub fn mode() -> Option<Mode> {
    Mode::from_code(MODE.load(Ordering::Acquire))
}
pub fn physical_shift(value: bool) {
    SHIFT.set(value);
}
pub fn shift() -> bool {
    SHIFT.get()
}
pub fn active() -> bool {
    matches!(mode(), Some(Mode::Vanilla | Mode::Ncp | Mode::Verus))
}
pub fn player(fall: f64, fire: bool, inside: bool) {
    SHAPE.set(model::shape_allowed(mode(), shift(), fall, fire, inside));
}
pub fn context(value: Context, in_water: bool) {
    CONTEXT.set(value);
    IN_WATER.set(in_water);
}
pub fn packet_context() -> (Context, bool) {
    (CONTEXT.get(), IN_WATER.get())
}
pub fn input_jump(sneak: bool, inside: bool) -> bool {
    model::input_jump(mode(), sneak, inside)
}
pub fn cancels_jump(standing: bool) -> bool {
    model::cancels_jump(mode(), standing)
}
pub fn shape(liquid: bool, original: VoxelShape) -> VoxelShape {
    if liquid && active() && SHAPE.get() && !crate::block_walk::shape_changes_suppressed() {
        VoxelShape::block()
    } else {
        original
    }
}
pub fn clear() {
    set_mode(0);
    SHIFT.set(false);
    SHAPE.set(false);
    CONTEXT.set([false; 4]);
    IN_WATER.set(false);
}
