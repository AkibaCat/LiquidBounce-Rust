// SPDX-License-Identifier: GPL-3.0-or-later
//! Process-wide BlockShapeEvent policy; populated from the actual block classes.
use crate::{
    avoid_hazards_model::{self as model, Flags},
    registry::BlockStateId,
};
use horizon_core::geometry::{Aabb, Vec3, VoxelShape};
use std::sync::{
    OnceLock, RwLock,
    atomic::{AtomicU32, Ordering},
};
static MASK: AtomicU32 = AtomicU32::new(0);
static FLAGS: OnceLock<RwLock<Vec<Flags>>> = OnceLock::new();
pub fn initialize(flags: Vec<Flags>) {
    *FLAGS
        .get_or_init(Default::default)
        .write()
        .expect("hazard registry") = flags;
}
pub fn set_mask(mask: u32) {
    MASK.store(mask, Ordering::Release);
}
pub fn active() -> bool {
    MASK.load(Ordering::Acquire) != 0
}
pub fn shape(state: BlockStateId, below: BlockStateId, shape: VoxelShape) -> VoxelShape {
    if crate::block_walk::shape_changes_suppressed() {
        return shape;
    }
    let mask = MASK.load(Ordering::Acquire);
    if mask == 0 {
        return shape;
    }
    let Some(flags) = FLAGS.get() else {
        return shape;
    };
    let flags = flags.read().expect("hazard registry");
    let current = flags.get(state.0 as usize).copied().unwrap_or_default();
    let below = flags.get(below.0 as usize).copied().unwrap_or_default();
    match model::shape(mask, current, below) {
        Some(1.) => VoxelShape::block(),
        Some(height) => VoxelShape::from_box(Aabb::new(Vec3::ZERO, Vec3::new(1., height, 1.)))
            .expect("hazard cap"),
        None => shape,
    }
}
