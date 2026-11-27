    pub(super) fn native_client_sword_block_state(&self, flags: &mut u32) {
        if self.native_window() && self.native_client_slot_owner() {
            mod_api::dispatch_client_sword_block_state(self, mod_api::mods(), flags);
        }
    }
    pub(super) fn client_sword_block_flags(&self) -> u32 {
        // ApplyToThirdPersonView is true even while SwordBlock is disabled:
        // the original AvatarRenderer mixin also gates KillAura's fake pose.
        let mut flags = 8;
        self.native_client_sword_block_state(&mut flags);
        flags
    }
    pub(super) fn client_sword_block_pose(&self) -> bool {
        use horizon_client::sword_block::Settings;
        let flags = self.client_sword_block_flags();
        let main = self.inventory.get(self.selected).and_then(Option::as_ref);
        let sword = main.is_some_and(|s| self.client_kill_aura_sword(s));
        let used = self.using.and_then(|(slot, _)| self.stack_in(slot).map(|s|(slot,s)));
        Settings::from_flags(flags).animate(flags & 1 != 0, sword,
            self.client_kill_aura_block_pose(), true, self.right,
            used.map(|(slot,_)|slot==super::shield::OFFHAND_SLOT),
            used.is_some_and(|(slot,s)|slot==super::shield::OFFHAND_SLOT && &*s.item=="minecraft:shield"),
            true)
    }
    pub(super) fn client_sword_block_hide_stack(&self, off: Option<&horizon_client::items::ItemStack>) -> bool {
        let flags = self.client_sword_block_flags();
        horizon_client::sword_block::Settings::from_flags(flags).hide_offhand(flags & 1 != 0,
            off.is_some_and(|s|&*s.item=="minecraft:shield"),
            self.inventory.get(self.selected).and_then(Option::as_ref)
                .is_some_and(|s|self.client_kill_aura_sword(s)))
    }
    pub(super) fn client_sword_block_hide_offhand(&self) -> bool {
        self.client_sword_block_hide_stack(self.offhand.as_ref())
    }
