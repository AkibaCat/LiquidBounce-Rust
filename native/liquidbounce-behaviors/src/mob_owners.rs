// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleMobOwners, LiquidBounce ae0eb396 (CCBlueX, 2015-2026).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MobOwners {
    pub projectiles: bool,
}
impl MobOwners {
    pub fn policy(&self, running: bool) -> u8 {
        if running {
            1 | if self.projectiles { 2 } else { 0 }
        } else {
            0
        }
    }
}
