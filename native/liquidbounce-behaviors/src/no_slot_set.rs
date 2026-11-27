// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleNoSlotSet.kt at LiquidBounce ae0eb396.
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Default)]
pub struct NoSlotSet {
    running: AtomicBool,
}

impl NoSlotSet {
    pub fn set_running(&self, running: bool) -> bool {
        self.running.swap(running, Ordering::AcqRel) != running
    }
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}
