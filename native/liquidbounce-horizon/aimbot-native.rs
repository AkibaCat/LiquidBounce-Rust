// SPDX-License-Identifier: GPL-3.0-or-later
//! Aimbot's real zero-tick hitboxes, crystal ordering and OUTLINE/player ray.
use super::{
    Game, Screen,
    auto_bow_math::{self as math, B, V},
};
use horizon_core::geometry::Vec3;
use horizon_world::world_ray::{RayBlockMode, RayFluidMode};
use serde_json::{Value, json};
fn v(p: Vec3) -> V {
    V(p.x, p.y, p.z)
}
fn p(v: V) -> Vec3 {
    Vec3::new(v.0, v.1, v.2)
}
fn bounds(b: horizon_core::geometry::Aabb) -> B {
    B {
        lo: v(b.min_position()),
        hi: v(b.max_position()),
    }
}
fn outline(g: &Game<'_>, from: V, to: V) -> bool {
    super::client_ghost_hand::clip(
        g,
        &mut super::no_web_ray::NoWebRay(g),
        p(from),
        p(to),
        RayBlockMode::Outline,
        RayFluidMode::None,
        4096,
    )
    .is_ok_and(|h| h.miss)
}
impl Game<'_> {
    pub(super) fn aimbot_query(&mut self, text: &str) -> String {
        let Ok(q) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        if q["op"] == "facts" {
            let mut facts =
                serde_json::from_str::<Value>(&self.auto_rod_query("{\"op\":\"facts\"}"))
                    .unwrap_or(Value::Null);
            if !facts.is_object() {
                return "null".into();
            }
            let Some(player) = self.player.as_ref() else {
                return "null".into();
            };
            let Some(eye) = self.eye_position().map(|e| V(e[0], e[1], e[2])) else {
                return "null".into();
            };
            let view = v(Vec3::direction_from_rotation(player.pitch, player.yaw));
            let mut crystals=self.crystals.iter().enumerate().map(|(index,c)| {
                let b=bounds(c.bounds());let at=c.position;
                let section=[(at.x.floor() as i32)>>4,(at.z.floor()as i32)>>4,(at.y.floor()as i32)>>4];
                (section,index,json!({"id":c.id as i32,"distance":b.distance(eye),"cosine":math::max_cos(eye,view,b)}))
            }).collect::<Vec<_>>();
            crystals.sort_by_key(|(section, index, _)| (*section, *index));
            let main = self.stack_in(self.client_hotbar_slot());
            facts["emptyHand"] =
                json!(main.is_none_or(|s| s.count == 0 || s.item.as_ref() == "minecraft:air"));
            facts["screen"] = json!(self.screen != Screen::Playing);
            facts["useKey"] = json!(self.settings.keys.get("key.use"));
            facts["crystals"] = json!(
                crystals
                    .into_iter()
                    .map(|(_, _, value)| value)
                    .collect::<Vec<_>>()
            );
            return facts.to_string();
        }
        let Some(player) = self.player.as_ref() else {
            return "null".into();
        };
        let Some(eye) = self.eye_position().map(|e| V(e[0], e[1], e[2])) else {
            return "null".into();
        };
        if q["op"] == "raytrace" {
            let Some(a) = serde_json::from_value::<[f64; 6]>(q["bounds"].clone()).ok() else {
                return "null".into();
            };
            let Some(point) = serde_json::from_value::<[f64; 3]>(q["point"].clone()).ok() else {
                return "null".into();
            };
            return super::aimbot_ray::raytrace(
                eye,
                B::from_array(a),
                V(point[0], point[1], point[2]),
                q["range"].as_f64().unwrap_or(0.),
                q["walls"].as_f64().unwrap_or(0.),
                |a, b| outline(self, a, b),
            )
            .map_or_else(|| "null".into(), |r| json!(r).to_string());
        }
        let id = q["id"].as_i64().unwrap_or(i64::MIN);
        let entities = super::auto_bow::entities(self);
        let (id, b, pickable) = if let Some(e) = entities.iter().find(|e| i64::from(e.id) == id) {
            let pickable = if let Some(m) = self.mobs.iter().find(|m| m.id as i32 == e.id) {
                !m.removed && !m.extra.spectator && m.alive()
            } else if let Some(a) = self.armor_stands.iter().find(|a| a.id as i32 == e.id) {
                !a.marker
            } else {
                true
            };
            (e.id, bounds(e.bounds), pickable)
        } else if let Some(c) = self.crystals.iter().find(|c| i64::from(c.id as i32) == id) {
            (c.id as i32, bounds(c.bounds()), true)
        } else {
            return "null".into();
        };
        let b = b.inflate(self.client_pick_margin(id, 0.));
        if q["op"] == "point" {
            return json!({"eye":eye.array(),"bounds":[b.lo.0,b.lo.1,b.lo.2,b.hi.0,b.hi.1,b.hi.2]})
                .to_string();
        }
        if q["op"] == "lazy" {
            if !pickable {
                return "false".into();
            }
            let range = q["range"].as_f64().unwrap_or(0.);
            let walls = q["walls"].as_f64().unwrap_or(0.);
            let end =
                eye.add(v(Vec3::direction_from_rotation(player.pitch, player.yaw)).scale(range));
            let hit = if b.contains(eye) {
                Some(eye)
            } else {
                let shared_root = self.riding.is_some_and(|r| {
                    self.client_reach_root_vehicle(r) == self.client_reach_root_vehicle(id as u32)
                });
                if shared_root {
                    None
                } else {
                    b.clip(eye, end)
                        .filter(|hit| hit.sub(eye).len2() < range * range)
                }
            };
            return json!(hit.is_some_and(|hit| {
                let d = hit.sub(eye).len2();
                d <= walls * walls
                    || d <= range * range
                        && super::spear_kill_ray::clip(self, p(eye), p(hit)).is_some_and(|h| h.0)
            }))
            .to_string();
        }
        "null".into()
    }
    /// WorldRender is emitted after camera extraction, before the 3D HUD. The
    /// source snapshots old angles and yBob then writes normalized local look.
    pub(super) fn client_aimbot_frame(&mut self, partial: f32) {
        if !self.native_window() || !self.native_client_slot_owner() {
            return;
        }
        let (mut yaw, mut pitch, mut apply) = (self.yaw, self.pitch, false);
        self.native_client_aimbot_render(
            partial,
            self.screen != Screen::Playing,
            matches!(self.screen, Screen::Inventory | Screen::Creative),
            &mut yaw,
            &mut pitch,
            &mut apply,
        );
        if !apply {
            return;
        }
        let old = (self.yaw, self.pitch);
        self.rope_rot_old = old;
        self.sway.reset_yaw(old.0);
        if yaw.is_finite() {
            self.yaw = yaw
        }
        if pitch.is_finite() {
            self.pitch = horizon_client::packet_bridge::set_x_rot(pitch)
        }
        if let Some(player) = self.player.as_mut() {
            player.yaw = self.yaw;
            player.pitch = self.pitch;
        }
        if let Some(model) = self.player_model.as_mut() {
            model.previous_yaw = old.0;
            model.yaw = self.yaw;
            model.previous_head_pitch = old.1;
            model.head_pitch = self.pitch;
        }
    }
}
