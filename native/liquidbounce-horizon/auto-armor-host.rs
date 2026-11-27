// SPDX-License-Identifier: GPL-3.0-or-later
    pub(super) fn native_client_auto_armor_context(&self)->Option<String> {
        if !self.client_auto_tool_player(){return None;}
        Some(serde_json::json!({"spectator":self.spectator(),
            "armor":self.player.as_ref()?.attributes.computed_value_of("minecraft:armor").unwrap_or(0.).floor() as i32}).to_string())
    }
    pub(super) fn native_client_auto_armor_screen(&mut self,open:bool)->bool {
        if !self.client_auto_tool_player(){return false;}
        if open {
            if self.screen!=Screen::Playing{return false;}
            if !self.client_gui_closer_accept_screen(Screen::Inventory){return false;}
            self.recipe_ui.reset_screen();
            self.client_inventory_screen_replaced();
            let screen=self.client_keymapping_screen(Screen::Inventory);
            let screen=self.native_client_replenish_screen(screen);
            self.screen=self.native_client_chest_stealer_screen(screen);
            self.craft_size=2;
            self.capture(false);
            self.release_use_for_keybind_screen();
        } else {self.native_client_inventory_close(self.client_inventory_container(),true);}
        true
    }
