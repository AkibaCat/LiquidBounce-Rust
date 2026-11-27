// SPDX-License-Identifier: GPL-3.0-or-later
//! Process-wide BlockShapeEvent policy, shared by client and integrated server.
use horizon_core::geometry::VoxelShape;
use std::{
    collections::BTreeSet,
    sync::{
        RwLock,
        atomic::{AtomicBool, Ordering},
    },
};
static BLOCKS: RwLock<BTreeSet<String>> = RwLock::new(BTreeSet::new());
static ACTIVE: AtomicBool = AtomicBool::new(false);
static SUPPRESSED: AtomicBool = AtomicBool::new(false);

pub fn sync(blocks: BTreeSet<String>) {
    let mut current = BLOCKS.write().unwrap_or_else(|error| error.into_inner());
    if *current != blocks {
        *current = blocks;
    }
    ACTIVE.store(!current.is_empty(), Ordering::Relaxed);
}
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed) && !shape_changes_suppressed()
}
pub fn shape_changes_suppressed() -> bool {
    SUPPRESSED.load(Ordering::Relaxed)
}
pub fn forces_full_block(name: &str) -> bool {
    active()
        && BLOCKS
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .contains(name)
}
pub fn shape(name: &str, original: VoxelShape) -> VoxelShape {
    if forces_full_block(name) {
        VoxelShape::block()
    } else {
        original
    }
}
/// ShapeFlag.noShapeChange/withoutShapeChanges, including nesting and unwinding.
pub fn without_shape_changes<T>(action: impl FnOnce() -> T) -> T {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            SUPPRESSED.store(self.0, Ordering::Relaxed);
        }
    }
    let _reset = Reset(SUPPRESSED.swap(true, Ordering::Relaxed));
    action()
}
