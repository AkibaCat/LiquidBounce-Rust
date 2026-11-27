    /// MixinMinecraft.handleKeybinds: an enforced AutoBlock hand only prevents
    /// releaseUsingItem. The physical key still starts and repeats item use.
    pub(super) fn client_kill_aura_use_down(&self) -> bool {
        let (mut enforced, mut pose) = (-1, false);
        self.native_client_kill_aura_state(&mut enforced, &mut pose);
        self.client_use_key_down() || matches!(enforced, 0 | 1)
    }

    pub(super) fn client_kill_aura_sword(&self, stack: &horizon_client::items::ItemStack) -> bool {
        self.items().item_tags.get("minecraft:swords")
            .is_some_and(|tag| tag.contains(&*stack.item))
    }

    /// shouldAnimateSwordBlock(localPlayer): the callback has already checked
    /// AutoBlock.running, wantsBlockPose and real BLOCK animation/SwordBlock.
    pub(super) fn client_kill_aura_block_pose(&self) -> bool {
        let (mut enforced, mut pose) = (-1, false);
        self.native_client_kill_aura_state(&mut enforced, &mut pose);
        pose && self.inventory.get(self.selected).and_then(Option::as_ref)
            .is_some_and(|stack| self.client_kill_aura_sword(stack))
    }
