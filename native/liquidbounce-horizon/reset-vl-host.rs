    pub(super) fn native_client_reset_vl_ground(&self) -> Option<bool> {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return None; }
        Some(self.player.as_ref().filter(|player| player.native_client_queries_allowed())?.ground.on_ground)
    }
    pub(super) fn native_client_reset_vl_velocity(&mut self, y: f64) {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return; }
        if let Some(player) = self.player.as_mut().filter(|player| player.native_client_queries_allowed()) {
            player.velocity = horizon_core::geometry::Vec3::new(0.0, y, 0.0);
        }
    }
