// SPDX-License-Identifier: GPL-3.0-or-later
//! Process-wide BlockShapeEvent: only empty shapes below the stored rescue floor.
use horizon_core::geometry::VoxelShape;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};
static FLOOR: OnceLock<fn() -> Option<f64>> = OnceLock::new();
static DEAD: AtomicBool = AtomicBool::new(true);
pub fn install(callback: fn() -> Option<f64>) {
    let _ = FLOOR.set(callback);
}
pub fn player_dead(dead: bool) {
    DEAD.store(dead, Ordering::Relaxed);
}
fn floor() -> Option<f64> {
    if crate::block_walk::shape_changes_suppressed() || DEAD.load(Ordering::Relaxed) {
        None
    } else {
        FLOOR.get().and_then(|callback| callback())
    }
}
pub fn active() -> bool {
    floor().is_some()
}
pub fn shape(y: i32, original: VoxelShape) -> VoxelShape {
    if original.is_empty() && floor().is_some_and(|floor| f64::from(y) < floor) {
        VoxelShape::block()
    } else {
        original
    }
}
