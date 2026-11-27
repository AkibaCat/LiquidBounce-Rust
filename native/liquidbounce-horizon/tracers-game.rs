// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_core::geometry::Axis;
use serde_json::{Value, json};
fn interpolate(a: [f64; 3], b: [f64; 3], alpha: f64) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * alpha)
}
impl Game<'_> {
    fn client_tracers_health(&self, holder: Option<&str>, native: f32) -> f32 {
        let Some(o) = self.score.board.display_objective("below_name") else {
            return native;
        };
        if !["❤", "HP", "Health", "Здоровья", "Здоровье"]
            .iter()
            .any(|s| o.display_text().plain().contains(s))
        {
            return native;
        }
        holder
            .and_then(|h| self.score.board.score(h, &o.name))
            .map_or(native, |s| s.value as f32)
    }
    fn client_tracers_target(&self, id: i32) -> Value {
        serde_json::from_str(&self.native_client_targeting_entity(id)).unwrap_or(Value::Null)
    }
    fn client_tracers_entities(&self, alpha: f64) -> Vec<Value> {
        let mut out = vec![];
        if let Some(p) = self.player.as_ref() {
            let at = [p.position.x, p.position.y, p.position.z];
            let prev = [
                self.previous_position.x,
                self.previous_position.y,
                self.previous_position.z,
            ];
            let id = self.remote.as_ref().map_or(0, |r| r.entity_id);
            let b = p.bounds();
            out.push(json!({"id":id,"identity":1u64,"kind":"minecraft:player","group":3,"baby":false,"position":at,"interpolated":interpolate(prev,at,alpha),"height":(b.max(Axis::Y)-b.min(Axis::Y))as f32,"health":p.health.health(),"maxHealth":p.clone().attribute("minecraft:max_health").unwrap_or(20.)as f32,"target":self.client_tracers_target(id)}));
        }
        for m in self.mobs.iter().filter(|m| !m.removed) {
            let kind = horizon_client::command::args::normalize(&m.kind.id);
            let player = kind == "minecraft:player";
            let holder = if player {
                self.remote_player_name(m.id).map(str::to_owned)
            } else {
                self.persist.uuids.get(&m.id).map(|u| {
                    horizon_protocol::crypto::uuid_string(
                        horizon_client::lan::playerdata::ints_uuid(*u),
                    )
                })
            };
            let at = [m.position.x, m.position.y, m.position.z];
            let prev = [m.previous.x, m.previous.y, m.previous.z];
            out.push(json!({"id":m.id as i32,"identity":m.join.get()<<3,"kind":kind,"group":if matches!(kind.as_str(),"minecraft:player"|"minecraft:mannequin"){3}else{0},"baby":m.is_baby(),"position":at,"interpolated":interpolate(prev,at,alpha),"height":m.dims().1 as f32,"health":self.client_tracers_health(holder.as_deref(),m.health),"maxHealth":horizon_client::mobs::finalize::max_health(m),"target":self.client_tracers_target(m.id as i32)}));
        }
        for a in self.armor_stands.iter() {
            let at = [a.position.x, a.position.y, a.position.z];
            let prev = [a.previous.x, a.previous.y, a.previous.z];
            let holder = self.persist.uuids.get(&a.id).map(|u| {
                horizon_protocol::crypto::uuid_string(horizon_client::lan::playerdata::ints_uuid(
                    *u,
                ))
            });
            out.push(json!({"id":a.id as i32,"identity":(a.join.get()<<3)|2,"kind":"minecraft:armor_stand","group":1,"baby":a.small,"position":at,"interpolated":interpolate(prev,at,alpha),"height":if a.marker{0.}else{a.probe_dimensions().1 as f32},"health":self.client_tracers_health(holder.as_deref(),a.health),"maxHealth":20.,"target":self.client_tracers_target(a.id as i32)}));
        }
        if let Some(d) = self.dragon.as_ref() {
            let at = [d.position.x, d.position.y, d.position.z];
            let prev = [d.previous.x, d.previous.y, d.previous.z];
            let id = self.client_targeting_dragon_id();
            let holder = self.persist.uuids.get(&d.id).map(|u| {
                horizon_protocol::crypto::uuid_string(horizon_client::lan::playerdata::ints_uuid(
                    *u,
                ))
            });
            out.push(json!({"id":id,"identity":3u64,"kind":"minecraft:ender_dragon","group":0,"baby":false,"position":at,"interpolated":interpolate(prev,at,alpha),"height":8.,"health":self.client_tracers_health(holder.as_deref(),d.health),"maxHealth":200.,"target":self.client_tracers_target(id)}));
        }
        for g in self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .filter(|g| g.placed && g.dimension == self.dim.id())
        {
            let Some(p) = g.motion.as_ref() else {
                continue;
            };
            let b = p.bounds();
            let pos = g.position;
            let within = self.border.geometry().is_within_bounds(
                f64::from(pos[0].floor() as i32),
                f64::from(pos[2].floor() as i32),
                0.,
            );
            let target = json!({"kind":"minecraft:player","alive":!p.health.dead_or_dying(),"invisible":g.effects.get("invisibility").is_some(),"sleeping":p.sleeping,"is_local":false,"has_local_passenger":false,"within_border":within,"player_name":g.name,"abstract_client_player":true,"teams":self.client_teams_facts(g.entity_id)});
            out.push(json!({"id":g.entity_id,"identity":(u64::from(g.conn)<<3)|4,"kind":"minecraft:player","group":3,"baby":false,"position":pos,"interpolated":interpolate(g.previous_position,pos,alpha),"height":(b.max(Axis::Y)-b.min(Axis::Y))as f32,"health":self.client_tracers_health(Some(&g.name),p.health.health()),"maxHealth":p.clone().attribute("minecraft:max_health").unwrap_or(20.)as f32,"target":target}));
        }
        out
    }
    fn client_tracers_perspective(&self) -> i32 {
        match self.camera_type {
            super::perspective::CameraType::FirstPerson => 0,
            super::perspective::CameraType::ThirdPersonBack => 1,
            super::perspective::CameraType::ThirdPersonFront => 2,
        }
    }
    pub(super) fn client_tracers_reset(&self) {
        horizon_client::gfx::tracers::begin_frame("null");
    }
    pub(super) fn client_tracers_tick(&self) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let input=json!({"tick":true,"perspective":self.client_tracers_perspective(),"entities":self.client_tracers_entities(1.)}).to_string();
        self.native_client_tracers(&input, &mut String::new());
    }
    pub(super) fn client_tracers_frame(&self, alpha: f64, camera: glam::DVec3, view: glam::Mat4) {
        let eye = view.inverse().transform_vector3(-glam::Vec3::Z).to_array();
        let input=json!({"tick":false,"perspective":self.client_tracers_perspective(),"camera":camera.to_array(),"eye":eye,"entities":self.client_tracers_entities(alpha)}).to_string();
        let mut geometry = "null".to_owned();
        self.native_client_tracers(&input, &mut geometry);
        horizon_client::gfx::tracers::begin_frame(&geometry);
    }
}
