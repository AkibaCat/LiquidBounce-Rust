// SPDX-License-Identifier: GPL-3.0-or-later
use liquidbounce_behaviors::sword_block::Settings;
use std::sync::atomic::{AtomicU32, Ordering};
/// Renderer and native prediction callbacks must not reenter the actor mutex.
pub struct SwordBlockService(AtomicU32);
impl Default for SwordBlockService {
    fn default() -> Self {
        Self(AtomicU32::new(Settings::default().flags(false)))
    }
}
impl SwordBlockService {
    pub fn flags(&self) -> u32 {
        self.0.load(Ordering::Acquire)
    }
    pub fn settings(&self) -> Settings {
        Settings::from_flags(self.flags())
    }
    pub fn configure(&self, s: Settings) {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |flags| {
                Some(s.flags(flags & 1 != 0))
            })
            .unwrap();
    }
    pub fn set_running(&self, running: bool) {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |flags| {
                Some((flags & !1) | u32::from(running))
            })
            .unwrap();
    }
}
