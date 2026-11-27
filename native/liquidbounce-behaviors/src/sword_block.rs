// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSwordBlock at LiquidBounce ae0eb396 and its renderer expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub only_visual: bool,
    pub fake_on_pressing: bool,
    pub hide_shield_slot: bool,
    pub third_person: bool,
    pub always_hide_shield: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            only_visual: false,
            fake_on_pressing: false,
            hide_shield_slot: false,
            third_person: true,
            always_hide_shield: false,
        }
    }
}
impl Settings {
    pub fn flags(self, running: bool) -> u32 {
        u32::from(running)
            | (u32::from(self.fake_on_pressing) << 1)
            | (u32::from(self.hide_shield_slot) << 2)
            | (u32::from(self.third_person) << 3)
            | (u32::from(self.always_hide_shield) << 4)
            | (u32::from(self.only_visual) << 5)
    }
    pub fn from_flags(flags: u32) -> Self {
        Self {
            only_visual: flags & 32 != 0,
            fake_on_pressing: flags & 2 != 0,
            hide_shield_slot: flags & 4 != 0,
            third_person: flags & 8 != 0,
            always_hide_shield: flags & 16 != 0,
        }
    }
    pub fn animate(
        self,
        running: bool,
        sword: bool,
        aura_pose: bool,
        local: bool,
        use_pressed: bool,
        used_hand: Option<bool>,
        used_off_shield: bool,
        blocks_attacks_existing: bool,
    ) -> bool {
        sword
            && ((aura_pose && local)
                || running
                    && (local && self.fake_on_pressing && use_pressed
                        || used_hand == Some(true) && used_off_shield
                        || used_hand == Some(false) && !blocks_attacks_existing))
    }
    pub fn hide_offhand(self, running: bool, off_shield: bool, sword: bool) -> bool {
        running && off_shield && (sword || self.always_hide_shield)
    }
    /// The only negotiated protocol in Horizon is 26.3. Its real
    /// blocks_attacks component makes the legacy shield-packet rewrite dormant.
    pub fn redirect_legacy_use(
        self,
        blocks_attacks_existing: bool,
        main: bool,
        sword: bool,
        block_animation: bool,
    ) -> bool {
        !self.only_visual && !blocks_attacks_existing && main && sword && !block_animation
    }
}
