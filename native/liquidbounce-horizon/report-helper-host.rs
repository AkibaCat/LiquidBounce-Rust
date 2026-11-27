    pub(super) fn native_client_report_helper_session(&self) -> u64 {
        // AuthSession is the real account object, currently installed once by
        // Horizon's launcher. No credentials or token enter the module state.
        horizon_client::auth::current().map_or(0, |session| std::sync::Arc::as_ptr(&session) as usize as u64)
    }
    pub(super) fn native_client_report_helper_players(&self) -> Option<serde_json::Value> {
        use serde_json::json;
        if !self.client_auto_tool_player() { return None; }
        let mut players = self.client_report_helper_world_players();
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
        for guest in self.lan.iter().flat_map(|lan| lan.hub.guests.iter())
            .filter(|guest| guest.placed && guest.dimension == self.dim.id() && guest.motion.is_some() && guest.entity_id != local) {
            if !players.iter().any(|p| p["id"].as_i64() == Some(i64::from(guest.entity_id))) {
                players.push(json!({"id":guest.entity_id,"name":guest.name,"local":false}));
            }
        }
        Some(json!({"self":self.own_profile_name(),"players":players}))
    }
    /// A genuine setScreen HEAD event, before assignment or native screen init.
    /// Only its accepted replacement can acquire this unique object token.
    pub(super) fn client_report_helper_screen_event(&mut self, next: Screen, accepted: bool) {
        self.client_report_helper_pending.set(None);
        if !self.client_auto_tool_player() || !matches!(next,Screen::Inventory|Screen::Creative) { return; }
        let token = self.client_report_helper_next.get().checked_add(1).expect("ReportHelper screen token exhausted");
        self.client_report_helper_next.set(token);
        if accepted { self.client_report_helper_pending.set(Some(token)); }
        self.native_client_report_helper_screen(token);
    }
    pub(super) fn native_client_report_helper_screen(&self, token: u64) {
        mod_api::dispatch_client_report_helper_screen(self,mod_api::mods(),token);
    }
    pub(super) fn native_client_report_helper_current(&self, token: u64) -> bool {
        self.client_auto_tool_player() && self.client_report_helper_current.get() == Some(token)
            && matches!(self.screen,Screen::Inventory|Screen::Creative)
    }
    pub(super) fn native_client_report_helper_screen_snapshot(&mut self, token: u64) -> Option<serde_json::Value> {
        if !self.native_client_report_helper_current(token) { return None; }
        // A cancelled newer menu may coexist with the old screen. Screen.menu,
        // its non-player slot identities and syncId all come from that view.
        Some(self.with_gui_closer_view(|game| {
            let mapping = game.client_inventory_mapping(false);
            let mut slots = Vec::new();
            for (index, internal) in mapping.into_iter().enumerate() {
                if internal.is_some_and(|slot| slot < 36 || (500..=504).contains(&slot)) { continue; }
                let stack = internal.and_then(|slot| game.slot_view(slot));
                slots.push(serde_json::json!({"index":index,"item":stack.as_ref().map_or("minecraft:air",|stack|stack.item.as_ref()),
                    "empty":stack.as_ref().is_none_or(|stack|stack.count == 0 || stack.item.as_ref() == "minecraft:air")}));
            }
            serde_json::json!({"container":game.client_inventory_container(),"slots":slots})
        }))
    }
    pub(super) fn native_client_report_helper_close(&mut self) {
        if !self.client_auto_tool_player() { return; }
        self.with_gui_closer_active(|game| {
            // LocalPlayer.clientSideCloseContainer deliberately does not send
            // ServerboundContainerClosePacket (LocalPlayer-26.3 javap).
            if game.remote.is_some() {
                game.mod_browser_closed();
                game.reset_inventory_clicks();
                game.remote_close_local();
            } else { game.close_container(); }
            game.client_inventory_screen_replaced();
            game.screen = Screen::Playing;
            game.capture(true);
        });
    }
