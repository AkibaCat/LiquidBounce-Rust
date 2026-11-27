    /// The logical Sprint key mapping and actual sprinting flag are separate
    /// source guards. Reading the key is not conditioned on screen/focus.
    pub(super) fn native_client_derp_state(&self) -> Option<serde_json::Value> {
        if !self.native_client_slot_owner() { return None; }
        let player = self.player.as_ref()?;
        Some(serde_json::json!({"yaw":self.yaw,"pitch":self.pitch,"sprintKey":self.keys.held(horizon_client::actions::Action::Sprint),"sprinting":player.sprinting}))
    }
