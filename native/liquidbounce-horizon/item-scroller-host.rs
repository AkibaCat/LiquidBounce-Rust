    pub(super) fn native_client_item_scroller_query(&self, shift: bool, left_pressed: bool, allowed: &mut bool) {
        mod_api::dispatch_client_item_scroller_query(self, mod_api::mods(), shift, left_pressed, allowed);
    }
    pub(super) fn native_client_item_scroller_reset(&self, now: i64) {
        mod_api::dispatch_client_item_scroller_reset(self, mod_api::mods(), now);
    }
    fn client_item_scroller_wall_time() -> i64 {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(elapsed) => elapsed.as_millis() as i64,
            Err(error) => (error.duration().as_millis() as i64).wrapping_neg(),
        }
    }
    fn client_item_scroller_allowed(&self) -> bool {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() {
            return false;
        }
        // InputConstants.isKeyDown asks physical Shift keys, independently of sneak bindings.
        let shift = self.physical_keys.contains(&KeyCode::ShiftLeft) || self.physical_keys.contains(&KeyCode::ShiftRight);
        let left = self.physical_buttons.contains(&MouseButton::Left);
        let mut allowed = false;
        self.native_client_item_scroller_query(shift, left, &mut allowed);
        allowed
    }
    fn client_item_scroller_hovered(&self, batch: &HudBatch) -> Option<usize> {
        if self.creative_inventory_open() && Self::creative_trash_hovered(batch, self.cursor) {
            return Some(1023);
        }
        self.slot_at(batch, self.cursor.0, self.cursor.1)
    }
    pub(super) fn client_item_scroller_cancel_click(&self) -> bool {
        let Some(renderer) = self.renderer.as_ref() else { return false; };
        let batch = HudBatch::new([renderer.config.width, renderer.config.height]);
        self.client_item_scroller_hovered(&batch).is_some() && self.client_item_scroller_allowed()
    }
    pub(super) fn client_item_scroller_render(&mut self, batch: &HudBatch) {
        // The mixin returns before canPerformScroll when carried is nonempty or no slot is hovered.
        if self.carried.as_ref().is_some_and(|stack| stack.count > 0 && &*stack.item != "minecraft:air") { return; }
        let Some(slot) = self.client_item_scroller_hovered(batch) else { return; };
        if !self.client_item_scroller_allowed() { return; }
        self.last_quick_moved = if slot == 1023 { None }
            else if self.creative_tab_open() && slot >= super::Creative::DISPLAY {
                let first = self.creative_screen.first_row(&self.pack.names);
                self.creative_screen.items(&self.pack.names).get(first * 9 + slot - super::Creative::DISPLAY).cloned()
            } else { self.slot_view(slot) }.filter(|stack| stack.count > 0 && &*stack.item != "minecraft:air");
        if slot == 1023 {
            // CreativeModeInventoryScreen.slotClicked(QUICK_MOVE) clears all 46 InventoryMenu slots,
            // and sends every slot, even those already empty. It does not clear the cursor.
            let before = self.player_equipment();
            self.inventory.fill(None);
            self.armor.fill(None);
            self.offhand = None;
            self.craft_grid[..4].fill(None);
            if let Some(remote) = self.remote.as_mut() { remote.craft_result = None; }
            self.player_equipment_changed(&before);
            self.apply_equipment();
            if self.remote.is_some() {
                for slot in 0..46 {
                    let item = self.to_network(&None);
                    self.send(horizon_protocol::packets::play::Serverbound::SetCreativeModeSlot(
                        horizon_protocol::packets::play::SetCreativeModeSlot { slot, item }));
                }
                self.remote_synced();
            }
        } else {
            // Exact InputConstants.MOUSE_BUTTON_LEFT at MC26.3 is SDL value 1.
            let done = self.menu_click(Some(slot), 1, horizon_client::menu::Input::QuickMove);
            if done && self.remote.is_none() && let Some(key) = self.container { self.container_changed(key); }
        }
        self.inv_clicks.client_skip_next_release();
        self.last_inv_click = Some((std::time::Instant::now(), Some(slot), false));
        self.native_client_item_scroller_reset(Self::client_item_scroller_wall_time());
    }
