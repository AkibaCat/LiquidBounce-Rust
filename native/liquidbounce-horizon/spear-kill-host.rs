/// Live used-stack components, mapped keys and ticksUsingItem.
pub(super) fn native_client_spear_kill_context(&self) -> Option<[f64; 7]> {
    if !self.native_client_slot_owner() {
        return None;
    }
    let used = self
        .using
        .and_then(|(slot, ticks)| self.stack_in(slot).map(|s| (s, ticks)));
    let (spear, kinetic, delay, duration, ticks) =
        used.map_or((false, false, 0, 0, 0), |(s, ticks)| {
            let spear = self
                .items()
                .item_tags
                .get("minecraft:spears")
                .is_some_and(|tag| tag.contains(&*s.item));
            let view = horizon_client::item_render::stack_view(s);
            let c = self.pack.item_renderer.models.components_view(&view);
            let k = c.get("minecraft:kinetic_weapon");
            (
                spear,
                k.is_some(),
                k.and_then(|k| k["delay_ticks"].as_i64()).unwrap_or(0) as i32,
                k.and_then(|k| k["damage_conditions"]["max_duration_ticks"].as_i64())
                    .unwrap_or(0) as i32,
                self.ticks_held(s, ticks),
            )
        });
    Some([
        u8::from(spear) as f64,
        u8::from(kinetic) as f64,
        f64::from(delay),
        f64::from(duration),
        f64::from(ticks),
        u8::from(self.client_use_key_down()) as f64,
        u8::from(self.left) as f64,
    ])
}
pub(super) fn native_client_spear_kill_restart(&mut self) {
    if !self.native_client_slot_owner() {
        return;
    }
    let Some((slot, _)) = self.using else {
        return;
    };
    let hand = if slot == super::shield::OFFHAND_SLOT {
        1
    } else {
        0
    };
    self.native_client_ensure_carried();
    if self.remote.is_some() {
        self.release_remote_use();
    } else if let Some((slot, ticks)) = self.using
        && let Some(stack) = self.stack_in(slot).cloned()
    {
        self.release_used_item(&stack, ticks, false);
    }
    let (mut yaw, mut pitch, mut managed) = (self.yaw, self.pitch, false);
    self.native_client_rotation_aim(&mut yaw, &mut pitch, &mut managed);
    self.native_client_use_item(hand, yaw, pitch);
}
pub(super) fn native_client_spear_kill_target(&mut self, maximum: f32) -> Option<[f64; 2]> {
    if !self.native_client_slot_owner() {
        return None;
    }
    super::spear_kill_target::find(self, maximum).map(|(id, d)| [f64::from(id), d])
}
pub(super) fn native_client_spear_kill_predict(&self, id: i32, ticks: i32) -> Option<[f64; 3]> {
    if !self.native_client_slot_owner() {
        return None;
    }
    super::spear_kill_forecast::predict(self, id, ticks)
}
pub(super) fn native_client_spear_kill_eye(&self) -> Option<[f64; 3]> {
    if !self.native_client_slot_owner() {
        return None;
    }
    let p = self.player.as_ref()?;
    Some([
        p.position.x,
        p.position.y + f64::from(p.dimensions.eye_height),
        p.position.z,
    ])
}
pub(super) fn native_client_spear_kill_look(&self) -> Option<[f64; 3]> {
    if !self.native_client_slot_owner() {
        return None;
    }
    let v = super::spear_kill_target::look(self.yaw, self.pitch);
    Some([v.x, v.y, v.z])
}
pub(super) fn native_client_spear_kill_velocity(&mut self, value: [f64; 3]) {
    if self.native_client_slot_owner()
        && let Some(p) = self.player.as_mut()
    {
        p.velocity = horizon_core::geometry::Vec3::new(value[0], value[1], value[2]);
    }
}
pub(super) fn native_client_spear_kill_boxes(&self, id: i32) -> String {
    if !self.native_client_slot_owner() {
        return "[]".to_owned();
    }
    serde_json::to_string(&super::spear_kill_target::boxes(self, id))
        .unwrap_or_else(|_| "[]".into())
}
