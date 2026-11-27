// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use serde_json::{Value, json};
#[derive(Default)]
pub(super) struct Rotations {
    guests: std::collections::BTreeMap<u128, [f32; 2]>,
}
fn interpolate(a: [f64; 3], b: [f64; 3], alpha: f64) -> [f64; 3] {
    std::array::from_fn(|i| alpha.mul_add(b[i] - a[i], a[i]))
}
fn rotation(old: [f32; 2], new: [f32; 2], alpha: f64) -> [f32; 2] {
    std::array::from_fn(|i| (alpha as f32).mul_add(new[i] - old[i], old[i]))
}
impl Game<'_> {
    pub(super) fn client_hats_tick(&mut self) {
        if let Some(error) = horizon_client::gfx::hats::take_error() {
            horizon_client::mod_api::Host::chat(self, &error);
        }
        // Snapshot the LAN entities' real rotations at ClientTick.Pre, before their world tick.
        self.client_hats_rotations.get_mut().guests = self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .filter(|g| g.placed && g.dimension == self.dim.id())
            .map(|g| (g.uuid, [g.yaw, g.pitch]))
            .collect();
    }
    fn client_hats_entities(&self, alpha: f64) -> Vec<Value> {
        let mut out = vec![];
        if let Some(p) = self.player.as_ref() {
            let at = [p.position.x, p.position.y, p.position.z];
            let prev = [
                self.previous_position.x,
                self.previous_position.y,
                self.previous_position.z,
            ];
            let old = if self.player_model.as_ref().is_none_or(|m| m.age == 0) {
                [self.yaw, self.pitch]
            } else {
                [self.rope_rot_old.0, self.rope_rot_old.1]
            };
            out.push(json!({"local":true,"position":at,"interpolated":interpolate(prev,at,alpha),"rotation":rotation(old,[self.yaw,self.pitch],alpha),"height":p.dimensions.height,"eyeHeight":p.dimensions.eye_height,"armor":self.armor[0].as_ref().is_some_and(|s|s.count>0),"hurt":p.health.hurt.hurt_time>0}));
        }
        for m in self
            .mobs
            .iter()
            .filter(|m| !m.removed && m.kind.id == "minecraft:player")
        {
            let at = [m.position.x, m.position.y, m.position.z];
            let prev = [m.previous.x, m.previous.y, m.previous.z];
            let old = if m.age == 0 {
                [m.yaw, m.head_pitch]
            } else {
                [m.previous_yaw, m.previous_head_pitch]
            };
            out.push(json!({"local":false,"name":self.remote_player_name(m.id),"position":at,"interpolated":interpolate(prev,at,alpha),"rotation":rotation(old,[m.yaw,m.head_pitch],alpha),"height":m.dims().1,"eyeHeight":m.eye_height(),"armor":m.equipment_stacks()[5].as_ref().is_some_and(|s|s.count>0),"hurt":m.hurt_time>0}));
        }
        let rotations = self.client_hats_rotations.borrow();
        for g in self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .filter(|g| g.placed && g.dimension == self.dim.id())
        {
            let Some(p) = g.motion.as_ref() else { continue };
            let old = rotations
                .guests
                .get(&g.uuid)
                .copied()
                .unwrap_or([g.yaw, g.pitch]);
            out.push(json!({"local":false,"name":g.name,"position":g.position,"interpolated":interpolate(g.previous_position,g.position,alpha),"rotation":rotation(old,[g.yaw,g.pitch],alpha),"height":p.dimensions.height,"eyeHeight":p.dimensions.eye_height,"armor":g.armor[0].as_ref().is_some_and(|s|s.count>0),"hurt":p.health.hurt.hurt_time>0}));
        }
        out
    }
    pub(super) fn client_hats_frame(&self, alpha: f64) {
        let input=json!({"firstPerson":self.camera_type.first_person(),"entities":self.client_hats_entities(alpha)}).to_string();
        let mut geometry = "null".to_owned();
        self.native_client_hats(&input, &mut geometry);
        horizon_client::gfx::hats::begin_frame(&geometry);
    }
    pub(super) fn client_hats_reset(&self) {
        self.client_hats_rotations.borrow_mut().guests.clear();
        horizon_client::gfx::hats::begin_frame("null");
    }
}
