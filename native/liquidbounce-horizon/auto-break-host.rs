    #[allow(clippy::too_many_arguments)]
    pub(super) fn native_client_auto_break_key(&self, attack: bool, miss_time: i32,
        block_hit: bool, block_state_available: bool, air: bool, destroying: bool,
        pressed: &mut bool, click: &mut bool) {
        mod_api::dispatch_client_auto_break_key(self, mod_api::mods(), attack, miss_time,
            block_hit, block_state_available, air, destroying, pressed, click);
    }

    pub(super) fn client_auto_break_attack_key(&self, original: bool, miss_time: i32, destroying: bool) -> bool {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|player| player.native_client_queries_allowed()) {
            return original;
        }
        let hit = (!self.entity_targeted()).then(|| self.pick()).flatten();
        let block_hit = hit.is_some();
        let air = hit.is_none_or(|hit| self.res.facts().is_air(self.world.block(hit.pos[0], hit.pos[1], hit.pos[2])));
        let (mut pressed, mut click) = (original, false);
        self.native_client_auto_break_key(true, miss_time, block_hit, block_hit, air, destroying, &mut pressed, &mut click);
        if click {
            let key = self.settings.keys.get("key.attack");
            self.client_keymapping_clicks.borrow_mut().click(key, self.settings.keys.mappings_for(key));
        }
        pressed
    }

    pub(super) fn client_keymapping_consume(&self, mapping: &str) -> bool {
        self.client_keymapping_clicks.borrow_mut().consume(mapping)
    }

    pub(super) fn client_keymapping_screen(&self, screen: super::Screen) -> super::Screen {
        if screen != super::Screen::Playing {
            self.client_keymapping_clicks.borrow_mut().release_all();
        }
        screen
    }

    pub(super) fn client_keymapping_query_allowed(&self) -> bool {
        self.screen == super::Screen::Playing && self.title.is_none()
            && self.sleeping.is_none() && !self.screen_over_playing()
    }

    pub(super) fn client_keymapping_mouse_grabbed(&self) -> bool {
        // Script input already has a virtual pointer under Horizon's control API.
        self.captured || self.agent_pointer || self.scripted_run() || self.control.is_some()
    }

    /// The consumeClick phases before attack/use/pick. Nonconsumed mappings keep their counts.
    pub(super) fn client_keymapping_prefix(&mut self) {
        if !self.client_keymapping_query_allowed() || !self.native_window()
            || self.lan_acting.is_some() || self.lan_acting_uuid.is_some()
            || self.remote.as_ref().is_some_and(|remote| !remote.loaded_sent) {
            return;
        }
        while self.client_keymapping_consume("key.togglePerspective") { self.cycle_camera(); }
        while self.client_keymapping_consume("key.smoothCamera") { self.smooth_camera = !self.smooth_camera; }
        while self.client_keymapping_consume("key.toggleGui") { self.toggle_hide_gui(); }
        while self.client_keymapping_consume("key.advancements") {
            self.open_advancements(false);
            self.release_use_for_keybind_screen();
        }
        while self.client_keymapping_consume("key.socialInteractions") {
            if self.is_multiplayer_server() {
                self.open_social(false);
                self.release_use_for_keybind_screen();
            } else {
                let message = horizon_client::text::Text::translate("multiplayer.socialInteractions.not_available", &[]);
                self.titles.set_overlay(message.clone(), false);
                self.say_system_now(&message);
            }
        }
        while self.client_keymapping_consume("key.chat") { self.open_chat_method(""); }
        if self.client_keymapping_query_allowed() && self.client_keymapping_consume("key.command") {
            self.open_chat_method("/");
        }
        while self.client_keymapping_consume("key.toggleSpectatorShaderEffects") {
            self.client_keymapping_spectator_effect.set(!self.client_keymapping_spectator_effect.get());
        }
        for slot in 0..9 {
            if self.client_keymapping_consume(&format!("key.hotbar.{}",slot+1)) {
                let load = self.mapped_key_down("key.loadToolbarActivator");
                if self.spectator() { self.spectator_hotbar_key(slot as i32); }
                else if self.creative && (load || self.mapped_key_down("key.saveToolbarActivator")) {
                    self.hotbar_load_or_save(slot, load);
                } else { self.selected = slot; }
            }
        }
        while self.client_keymapping_consume("key.inventory") {
            if self.remote.is_some() && (self.riding_mount_with_menu() || self.riding_chest_boat()) {
                if !self.in_portal_transition() {
                    let id = self.remote.as_ref().map_or(0, |remote| remote.entity_id);
                    self.send(horizon_protocol::packets::play::Serverbound::PlayerCommand(
                        horizon_protocol::packets::play::PlayerCommand { id, action:5, data:0 }));
                }
            } else if !self.in_portal_transition() {
                if self.tutorial.open_inventory() { self.persist_tutorial_step(); }
                self.recipe_ui.reset_screen();
                if self.creative {
                    self.screen = super::Screen::Creative;
                    self.refresh_creative_tabs();
                    self.creative_screen.reopen();
                    self.refresh_saved_hotbars();
                } else { self.screen = super::Screen::Inventory; }
                self.craft_size = 2;
                self.capture(false);
                self.release_use_for_keybind_screen();
            }
        }
        while self.client_keymapping_consume("key.quickActions") {
            if self.has_quick_actions() { self.open_quick_actions(); }
        }
        while self.client_keymapping_consume("key.swapOffhand") {
            if self.spectator() { continue; }
            if self.remote.is_some() { self.remote_swap_hands(); }
            else {
                std::mem::swap(&mut self.offhand, &mut self.inventory[self.selected]);
                self.hand_stack_replaced(self.selected);
                self.hand_stack_replaced(super::shield::OFFHAND_SLOT);
                let (offhand, main) = (self.offhand.clone(), self.inventory[self.selected].clone());
                self.player_on_equip("offhand", main.as_ref(), offhand.as_ref());
                self.player_on_equip("mainhand", offhand.as_ref(), main.as_ref());
                self.using = None;
            }
        }
        while self.client_keymapping_consume("key.drop") {
            if self.spectator() { continue; }
            if self.remote.is_some() { self.remote_drop(self.has_control_down()); }
            else { self.drop_held(self.has_control_down()); }
        }
        for (mapping, button) in [
            ("key.attack", horizon_client::interaction_input::Button::Attack),
            ("key.use", horizon_client::interaction_input::Button::Use),
            ("key.pickItem", horizon_client::interaction_input::Button::Pick),
        ] {
            while self.client_keymapping_consume(mapping) {
                if let Some(remote) = self.remote.as_mut() { remote.input.press(button); }
                else {
                    match button {
                        horizon_client::interaction_input::Button::Attack => self.attack_clicks = self.attack_clicks.wrapping_add(1),
                        horizon_client::interaction_input::Button::Use => self.use_clicks = self.use_clicks.wrapping_add(1),
                        horizon_client::interaction_input::Button::Pick => self.pick_clicks = self.pick_clicks.wrapping_add(1),
                    }
                }
            }
        }
    }

    pub(super) fn client_keymapping_spectator_clicks(&mut self, was_using: bool) {
        if !was_using && self.spectator() {
            while self.client_keymapping_consume("key.spectatorHotbar") { self.spectator_action_key(); }
        }
    }
