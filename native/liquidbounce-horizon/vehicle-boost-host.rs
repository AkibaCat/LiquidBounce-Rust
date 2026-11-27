    /// Real local passenger state and current player yaw at GameTick.
    pub(super) fn native_client_vehicle_boost_state(&self) -> Option<[f64;2]> {
        if !self.native_client_slot_owner() { return None; }
        Some([f64::from(self.yaw), if self.riding.is_some() {1.0} else {0.0}])
    }
    /// Use Horizon's Java Math implementation rather than the Windows CRT.
    pub(super) fn native_client_java_sin_cos(&self, angle: f64) -> [f64;2] {
        [horizon_core::jmath::sin(angle), horizon_core::jmath::cos(angle)]
    }
    /// The module assigns deltaMovement; it does not add an impulse.
    pub(super) fn native_client_vehicle_boost_velocity(&mut self, velocity:[f64;3]) -> bool {
        if !self.native_client_slot_owner() { return false; }
        let Some(player)=self.player.as_mut() else { return false; };
        player.velocity=horizon_core::geometry::Vec3::new(velocity[0],velocity[1],velocity[2]);
        true
    }
