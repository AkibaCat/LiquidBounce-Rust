// SPDX-License-Identifier: GPL-3.0-or-later
//! Minimal world-module state used while the full upstream planners are being ported.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldNoop {
    pub active: bool,
    pub ticks: u64,
}
impl WorldNoop {
    pub fn apply(&mut self, active: bool) {
        self.active = active;
    }
    pub fn clear(&mut self) {
        self.active = false;
        self.ticks = 0;
    }
    pub fn tick(&mut self, running: bool) {
        if running {
            self.active = true;
            self.ticks = self.ticks.saturating_add(1);
        } else {
            self.clear();
        }
    }
}
