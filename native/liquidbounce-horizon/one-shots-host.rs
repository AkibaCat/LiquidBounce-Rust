    /// NameCollector needs every online PlayerInfo, including unlisted entries.
    pub(super) fn native_client_name_collector_names(&self) -> Vec<String> {
        if !self.native_client_slot_owner() { return Vec::new(); }
        self.remote.as_ref().map_or_else(Vec::new, |remote| remote.liquidbounce_online_player_names())
    }
    pub(super) fn native_client_auto_leave_context(&self) -> Option<[f64; 4]> {
        if !self.native_client_slot_owner() { return None; }
        let player = self.player.as_ref()?;
        let protected = [self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref), self.offhand.as_ref()]
            .into_iter().flatten().filter(|stack| stack.count > 0).any(|stack| {
                let view = horizon_client::item_render::stack_view(stack);
                self.pack.item_renderer.models.components_view(&view).get("minecraft:death_protection").is_some()
            });
        Some([f64::from(player.health.health()), if self.creative {1.0} else {0.0}, if self.has_singleplayer_server() {1.0} else {0.0}, if protected {1.0} else {0.0}])
    }
    /// Ordinary packets go through Game::send and the shared PacketPipeline.
    /// Kick's onEnabled performs the integrated-server guard before this call.
    pub(super) fn native_client_kick(&mut self, mode: i32, random_a: i32, random_b: i32) {
        use horizon_protocol::packets::play::{self, Serverbound};
        if !self.native_client_slot_owner() { return; }
        let Some(player) = self.player.as_ref() else { return; };
        let id = self.remote.as_ref().map_or(0, |remote| remote.entity_id);
        match mode {
            0 => { if let Err(error) = self.leave_server(Some(String::new())) { eprintln!("[LiquidBounce] Kick: {error}"); } }
            1 => self.send(Serverbound::MovePlayerPos(play::MovePlayerPos {
                x: f64::NAN, y: f64::NEG_INFINITY, z: f64::INFINITY,
                flags: u8::from(!player.ground.on_ground) | (u8::from(!player.collision.horizontal) << 1),
            })),
            2 => self.send(Serverbound::Attack(play::Attack { id })),
            3 => self.send_signed_chat(&format!("§{random_a}§{random_b}§")),
            4 => self.send(Serverbound::SetCarriedItem(play::SetCarriedItem { slot: -1 })),
            5 => self.send(Serverbound::Interact(play::Interact { id, hand: 0, location: [0.0; 3], secondary: player.shift_key_down })),
            _ => {}
        }
    }
