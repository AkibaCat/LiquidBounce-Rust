// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from ModuleNoMissCooldown.kt and MixinMinecraft.java at ae0eb396.
//! Settings and decisions at Minecraft.startAttack's actual injection phases.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoMissCooldown {
    pub remove_attack_cooldown: bool,
    pub cancel_attack_on_miss: bool,
}

impl Default for NoMissCooldown {
    fn default() -> Self {
        Self {
            remove_attack_cooldown: true,
            cancel_attack_on_miss: false,
        }
    }
}

impl NoMissCooldown {
    pub fn queries(self, running: bool) -> (bool, bool) {
        (
            running && self.remove_attack_cooldown,
            running && self.cancel_attack_on_miss,
        )
    }
}
