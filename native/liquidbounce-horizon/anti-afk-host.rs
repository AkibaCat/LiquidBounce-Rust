    /// Actual local player's current swing/ground/rotation, including screens.
    pub(super) fn native_client_anti_afk_state(&self) -> Option<serde_json::Value> {
        if !self.native_client_slot_owner() { return None; }
        let player = self.player.as_ref()?;
        Some(serde_json::json!({"swinging":self.swing.swinging(),"onGround":player.ground.on_ground,"yaw":self.yaw,"pitch":self.pitch}))
    }
    /// Options.keyUp.isDown = true. Use the actual logical Forward mapping;
    /// the normal key release/screen/focus handlers own its later release.
    pub(super) fn native_client_anti_afk_forward(&mut self) -> bool {
        if !self.native_client_slot_owner() { return false; }
        if let Some(key) = horizon_client::actions::Action::Forward.key() { self.keys.insert(key); }
        true
    }
    /// Inventory.selected's raw field assignment, not SilentHotbar selection
    /// and not a key event. The next ordinary carried update synchronizes it.
    pub(super) fn native_client_anti_afk_select_slot(&mut self, slot: i32) -> bool {
        if !self.native_client_slot_owner() || !(0..9).contains(&slot) { return false; }
        self.selected = slot as usize;
        true
    }
    /// Direct Entity.yRot/xRot writes. These do not snapshot old rotations or
    /// reset interpolation, and do not submit a RotationManager request.
    pub(super) fn native_client_anti_afk_rotation(&mut self, yaw: Option<f32>, pitch: Option<f32>) -> bool {
        if !self.native_client_slot_owner() { return false; }
        if let Some(yaw) = yaw.filter(|value| value.is_finite()) { self.yaw = yaw; }
        if let Some(pitch) = pitch.filter(|value| value.is_finite()) { self.pitch = horizon_client::packet_bridge::set_x_rot(pitch); }
        if let Some(player) = self.player.as_mut() { player.yaw = self.yaw; player.pitch = self.pitch; }
        if let Some(model) = self.player_model.as_mut() { model.yaw = self.yaw; model.head_pitch = self.pitch; }
        true
    }
    /// SwingMode.DO_NOT_HIDE.attackSwing: main hand's real attack_animation
    /// component, local LivingEntity swing, then ordinary Punch even when the
    /// current local animation refuses to restart. No carried sync is added.
    pub(super) fn native_client_anti_afk_swing(&mut self) -> bool {
        if !self.native_client_slot_owner() { return false; }
        use horizon_client::first_person::{Hand, SwingAnimation};
        let mut animation = SwingAnimation::DEFAULT;
        // getMainHandItem uses the current effective inventory slot, while
        // this operation's hand remains MAIN_HAND even after offhand use.
        if let Some(stack) = self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref) {
            let view = horizon_client::item_render::stack_view(stack);
            let components = self.pack.item_renderer.models.components_view(&view);
            if let Some(value) = components.get("minecraft:attack_animation") {
                animation.kind = match value.get("type").and_then(serde_json::Value::as_str).unwrap_or("whack") {
                    "none" => 0, "whack" => 1, "stab" => 2, _ => 1,
                };
                animation.duration = value.get("duration").and_then(serde_json::Value::as_i64).unwrap_or(6) as i32;
            }
        }
        self.swing_hand(Hand::Main, animation);
        self.send(horizon_protocol::packets::play::Serverbound::Punch(horizon_protocol::packets::Empty));
        true
    }
