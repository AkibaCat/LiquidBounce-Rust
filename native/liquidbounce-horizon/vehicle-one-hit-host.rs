    /// The actual entity registry key. Current protocol 777 has no legacy
    /// transport; keep the original <=1.8 policy separate from combat rules.
    pub(super) fn native_client_vehicle_one_hit_context(&self, target: u32) -> Option<serde_json::Value> {
        if !self.native_client_slot_owner() { return None; }
        let kind = self.native_entity_kind(self.native_entity(target)?)?;
        Some(serde_json::json!({"kind":kind,"legacy":horizon_core::version::PROTOCOL_VERSION <= 47}))
    }
    /// Ordinary network.send, without recursively dispatching AttackEntityEvent.
    pub(super) fn native_client_vehicle_one_hit_attack(&mut self, target: i32) {
        if !self.native_client_slot_owner() { return; }
        self.send(horizon_protocol::packets::play::Serverbound::Attack(
            horizon_protocol::packets::play::Attack { id: target }));
    }
    /// SwingMode.HIDE_SERVER.attackSwing: resolve the effective main hand's real
    /// attack_animation, then LivingEntity.swing with no Punch packet. The
    /// ordinary swing_hand path still observes NoSwing's client cancellation.
    pub(super) fn native_client_vehicle_one_hit_swing(&mut self) {
        if !self.native_client_slot_owner() { return; }
        use horizon_client::first_person::{Hand, SwingAnimation};
        let mut animation = SwingAnimation::DEFAULT;
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
    }
