// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleDankBobbing, LiquidBounce ae0eb396. The renderer owns its matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DankBobbing {
    pub motion: f32,
}
impl Default for DankBobbing {
    fn default() -> Self {
        Self { motion: 5.0 }
    }
}
impl DankBobbing {
    pub fn motion(self, running: bool) -> Option<f32> {
        running.then_some(self.motion)
    }
}
