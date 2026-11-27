// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{damage_particles_capture as capture, hud::HudBatch};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap};
thread_local! {static LIVES:RefCell<(u64,BTreeMap<(u8,i32),(i64,u64)>)>=RefCell::new((0,BTreeMap::new()));}
fn lifetime(role: u8, id: i32, tick: i64) -> u64 {
    LIVES.with_borrow_mut(|s| {
        if s.1.get(&(role, id)).is_none_or(|v| tick < v.0) {
            s.0 += 1;
            let next = s.0;
            s.1.insert((role, id), (tick, next));
        }
        let e = s.1.get_mut(&(role, id)).unwrap();
        e.0 = tick;
        (e.1 << 3) | u64::from(role)
    })
}
fn updates() -> Vec<Value> {
    capture::drain().into_iter().map(|u|json!({"now":u.now,"own":u.context.own,"ticked":u.context.ticked,"center":u.context.center,"movement":u.context.movement,"old":u.old,"new":u.new,"max":u.max})).collect()
}
impl Game<'_> {
    pub(super) fn client_damage_particles_reset(&self) {
        capture::clear();
        LIVES.with_borrow_mut(|s| s.1.clear());
    }
    pub(super) fn client_damage_particles_tick(&self) {
        if !self.native_window() {
            return;
        }
        capture::mark_client();
        let mut entities = vec![];
        for m in self.mobs.iter().filter(|m| !m.removed) {
            let c = m.bounds().center();
            let v = m.velocity;
            entities.push(json!({"identity":m.join.get()<<3,"own":false,"ticked":m.age!=0,"dead":m.health<=0.||m.death_time>0,"health":m.health,"max":horizon_client::mobs::finalize::max_health(m),"center":[c.x,c.y,c.z],"movement":[v.x,v.y,v.z]}));
        }
        for s in self.armor_stands.iter() {
            let c = s.bounds().center();
            let v = s.velocity;
            entities.push(json!({"identity":(s.join.get()<<3)|1,"own":false,"ticked":!s.first_tick,"dead":s.health<=0.,"health":s.health,"max":20.,"center":[c.x,c.y,c.z],"movement":[v.x,v.y,v.z]}));
        }
        if let Some(d) = self.dragon.as_ref() {
            entities.push(d.client_damage_particles_entity(lifetime(
                2,
                d.id as i32,
                d.client_damage_particles_ticks(),
            )));
        }
        for g in self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .filter(|g| g.placed)
        {
            let Some(mut p) = g.motion.clone() else {
                continue;
            };
            let c = p.bounds().center();
            let v = g.known_movement;
            let max = p.attribute("minecraft:max_health").unwrap_or(20.) as f32;
            entities.push(json!({"identity":lifetime(3,g.entity_id,p.health.tick_count as i64),"own":false,"ticked":p.health.tick_count!=0,"dead":p.health.dead_or_dying(),"health":p.health.health(),"max":max,"center":[c.x,c.y,c.z],"movement":v}));
        }
        let input = json!({"tick":true,"entities":entities,"updates":updates()}).to_string();
        self.native_client_damage_particles(&input, &mut String::new());
    }
    pub(super) fn client_damage_particles_overlay(&self, batch: &mut HudBatch) {
        if !self.native_window() {
            return;
        }
        let Some(c) = self.native_camera() else {
            return;
        };
        let input = json!({"tick":false,"updates":updates()}).to_string();
        let mut draws = "[]".to_owned();
        self.native_client_damage_particles(&input, &mut draws);
        let Ok(Value::Array(draws)) = serde_json::from_str::<Value>(&draws) else {
            return;
        };
        let matrix =
            glam::Mat4::from_cols_array(&c.projection) * glam::Mat4::from_cols_array(&c.view);
        for p in draws {
            let Some(text) = p["text"].as_str() else {
                continue;
            };
            let Ok(pos) = serde_json::from_value::<[f64; 3]>(p["position"].clone()) else {
                continue;
            };
            let v = matrix
                * glam::Vec4::new(
                    (pos[0] - c.position[0]) as f32,
                    (pos[1] - c.position[1]) as f32,
                    (pos[2] - c.position[2]) as f32,
                    1.,
                );
            if !v.is_finite() || v.w <= 0. {
                continue;
            }
            let at = [
                (v.x / v.w + 1.) * batch.width * 0.5,
                (-v.y / v.w + 1.) * batch.height * 0.5,
            ];
            let scale = p["scale"].as_f64().unwrap_or(0.) as f32;
            let color = p["color"].as_i64().unwrap_or(-1) as u32;
            let start = batch.vertices.len();
            let width = HudBatch::text_width(&self.gui.atlas, text).ceil() as i32;
            batch.text(&self.gui.atlas, text, -(width / 2) as f32, 0., color, true);
            batch.scale_since(start, 0., 0., scale, scale);
            for vertex in &mut batch.vertices[start..] {
                vertex.pos[0] += at[0];
                vertex.pos[1] += at[1];
            }
        }
    }
}
