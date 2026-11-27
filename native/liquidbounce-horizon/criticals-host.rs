pub(super) fn native_client_criticals_context(&self) -> Option<serde_json::Value> {
    let mut v = self.native_client_crosshair_clicker_critical()?;
    let p = self.player.as_ref()?;
    v["y"] = serde_json::json!(p.position.y);
    v["oldY"] = serde_json::json!(p.old_position.y);
    v["tickCount"] = serde_json::json!(p.health.tick_count);
    Some(v)
}
pub(super) fn native_client_criticals_candidates(
    &self,
    range: f32,
    visible: bool,
) -> Vec<serde_json::Value> {
    if !self.native_client_slot_owner() {
        return vec![];
    }
    super::criticals_target::candidates(self, range, visible)
}
pub(super) fn native_client_criticals_packet(&mut self, kind: usize, offset: f64, ground: bool) {
    use horizon_protocol::packets::play::{
        MovePlayerPos, MovePlayerPosRot, MovePlayerRot, MovePlayerStatus, Serverbound as S,
    };
    if !self.native_client_slot_owner() {
        return;
    }
    let Some(p) = self.player.as_ref() else {
        return;
    };
    let at = p.position;
    let flags = u8::from(ground) | u8::from(p.collision.horizontal) << 1;
    let packet = match kind {
        0 => S::MovePlayerStatus(MovePlayerStatus { flags }),
        1 => S::MovePlayerPos(MovePlayerPos {
            x: at.x,
            y: at.y + offset,
            z: at.z,
            flags,
        }),
        2 => S::MovePlayerRot(MovePlayerRot {
            yaw: self.yaw,
            pitch: self.pitch,
            flags,
        }),
        _ => S::MovePlayerPosRot(MovePlayerPosRot {
            x: at.x,
            y: at.y + offset,
            z: at.z,
            yaw: self.yaw,
            pitch: self.pitch,
            flags,
        }),
    };
    self.send(packet);
}
pub(super) fn native_client_criticals_stop(&mut self) {
    if !self.native_client_slot_owner() {
        return;
    }
    self.native_client_super_knockback_packet(false);
    if let Some(r) = self.remote.as_mut() {
        r.was_sprinting = false;
    }
    if let Some(p) = self.player.as_mut() {
        p.client_super_knockback_was_sprinting = false;
    }
}
pub(super) fn native_client_criticals_visual(&mut self, id: i32, critical: i32, magic: i32) {
    use horizon_client::particles::{ParticleOptions, ParticleType};
    if !self.native_client_slot_owner() {
        return;
    }
    let Some((target, body)) = super::criticals_target::tracking(self, id) else {
        return;
    };
    for (particle, count) in [
        (ParticleType::Crit, critical),
        (ParticleType::EnchantedHit, magic),
    ] {
        for _ in 0..count {
            self.particles
                .tracking_emitter(ParticleOptions::simple(particle), target, body);
        }
    }
}
pub(super) fn native_client_criticals_receive_probe(&mut self) {
    if self.native_window() {
        mod_api::dispatch_client_criticals_receive_probe(self, mod_api::mods());
    }
}
