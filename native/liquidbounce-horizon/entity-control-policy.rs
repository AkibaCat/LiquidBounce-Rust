// SPDX-License-Identifier: GPL-3.0-or-later
//! The original Mob mixin reads process-wide module running state. Keep that
//! getter separate from the actual saddle equipment and its rendered model.
use std::sync::atomic::{AtomicU8, Ordering};

static POLICY: AtomicU8 = AtomicU8::new(0);

/// Only the real native window updates this snapshot after its pre-tick
/// lifecycle/config work; server instances never replace the window policy.
pub fn sync(saddled: bool, jump_strength: bool) {
    POLICY.store(
        u8::from(saddled) | (u8::from(jump_strength) << 1),
        Ordering::Relaxed,
    );
}
pub fn is_saddled(original: bool) -> bool {
    original || POLICY.load(Ordering::Relaxed) & 1 != 0
}
pub fn force_jump_strength() -> bool {
    POLICY.load(Ordering::Relaxed) & 2 != 0
}
pub fn jump_scale(original: f32, force: bool) -> f32 {
    if force { 1.0 } else { original }
}
