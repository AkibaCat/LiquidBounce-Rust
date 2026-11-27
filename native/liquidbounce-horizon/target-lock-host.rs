    /// Genuine AbstractClientPlayer identity: local player or remote level-player
    /// profile plus its actual client entity. Tab-list presence is irrelevant.
    pub(super) fn native_client_target_lock_player(&self, entity: i32) -> String {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return "null".into(); }
        let Some(local) = self.player.as_ref().filter(|p| p.native_client_queries_allowed()) else { return "null".into(); };
        let local_id = self.remote.as_ref().map_or(0, |r| r.entity_id);
        let (name, position, removed) = if entity == local_id {
            (self.remote.as_ref().map_or_else(|| self.acting_player_name(), |r| r.profile_name.as_str()), [local.position.x,local.position.y,local.position.z], false)
        } else if let Some(guest) = self.lan.iter().flat_map(|h| h.hub.guests.iter()).find(|g| g.placed && g.entity_id == entity && g.dimension == self.dim.id()) {
            (guest.name.as_str(), guest.position, false)
        } else {
            let Some(name) = self.remote_player_name(entity as u32) else { return "null".into(); };
            let Some(player) = self.mobs.iter().find(|m| m.id as i32 == entity && horizon_client::command::args::normalize(&m.kind.id) == "minecraft:player") else { return "null".into(); };
            (name, [player.position.x,player.position.y,player.position.z], player.removed)
        };
        let dx = position[0] - local.position.x;
        let dy = position[1] - local.position.y;
        let dz = position[2] - local.position.z;
        serde_json::json!({"id":entity,"name":name,"removed":removed,"distance_squared_bits":(dx*dx+dy*dy+dz*dz).to_bits().to_string()}).to_string()
    }
    pub(super) fn native_client_target_lock_dead(&self) -> Option<bool> {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return None; }
        Some(self.player.as_ref().filter(|p| p.native_client_queries_allowed())?.health.dead_or_dying())
    }
