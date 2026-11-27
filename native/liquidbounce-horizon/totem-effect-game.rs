// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_protocol::packets::play::Clientbound;
use serde_json::{Value, json};
impl Game<'_> {
    pub(super) fn client_totem_effect_packet(&self, packet: &Clientbound) {
        let Clientbound::EntityEvent(p) = packet else {
            return;
        };
        if p.event != 35 || !self.native_window() {
            return;
        }
        let Some(handle) = u32::try_from(p.id)
            .ok()
            .and_then(|id| self.native_entity(id))
        else {
            return;
        };
        let (Some(position), Some([yaw, pitch]), Some([_, height])) = (
            self.native_entity_position(handle),
            self.native_entity_rotation(handle),
            self.native_entity_dimensions(handle),
        ) else {
            return;
        };
        let target: Value =
            serde_json::from_str(&self.native_client_targeting_entity(p.id)).unwrap_or(Value::Null);
        let input=json!({"action":"pop","id":p.id,"position":position,"xRot":pitch,"yRot":yaw,"height":height as f32,"target":target,"firstPerson":self.effective_camera_type().first_person()}).to_string();
        self.native_client_totem_effect(&input, &mut String::new());
    }
    pub(super) fn client_totem_effect_frame(&self, partial: f64) {
        let mut geometry = "null".to_owned();
        if self.native_window() {
            self.native_client_totem_effect(
                &json!({"action":"frame","partial":partial as f32}).to_string(),
                &mut geometry,
            );
        }
        horizon_client::gfx::totem_effect::begin_frame(&geometry);
    }
}
