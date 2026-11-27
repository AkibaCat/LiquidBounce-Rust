// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleMoreCarry.kt at LiquidBounce ae0eb396 (alias: XCarry).
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Default)]
pub struct MoreCarry {
    running: AtomicBool,
}

impl MoreCarry {
    pub fn set_running(&self, running: bool) -> bool {
        self.running.swap(running, Ordering::AcqRel) != running
    }

    /// All serverbound container closes, without filtering the container id.
    pub fn cancel_container_close(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}
