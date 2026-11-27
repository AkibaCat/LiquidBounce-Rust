// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::hud::HudBatch;
use horizon_core::geometry::{Axis, Vec3};
use serde_json::{Value, json};
thread_local! {static OVERLAY:std::cell::RefCell<Vec<Value>>=const{std::cell::RefCell::new(Vec::new())};}
pub(super) fn object(
    id: u32,
    identity: u64,
    kind: &str,
    position: Vec3,
    previous: Vec3,
    width: f64,
    height: f64,
    alpha: f64,
    category: u8,
    own_vehicle: bool,
) -> Value {
    let p = [position.x, position.y, position.z];
    let old = [previous.x, previous.y, previous.z];
    json!({"id":id as i32,"identity":identity,"kind":kind,"position":p,"interpolated":std::array::from_fn::<_,3,_>(|i|old[i]+(p[i]-old[i])*alpha),"width":width,"height":height,"living":false,"object":category,"ownVehicle":own_vehicle})
}
impl Game<'_> {
    fn client_esp_entities(&self, alpha: f64) -> Vec<Value> {
        // Shared live targeting/scoreboard-health provider; add the pose dimensions and
        // hurt state consumed by ESP instead of reconstructing them from rendered meshes.
        let mut entities = self.client_tracers_entities(alpha);
        for e in &mut entities {
            let id = e["id"].as_i64().unwrap_or(0) as i32;
            e["living"] = json!(true);
            let (width, hurt, bounding) = if id == self.remote.as_ref().map_or(0, |r| r.entity_id) {
                self.player.as_ref().map(|p| {
                    (
                        f64::from(p.dimensions.width),
                        p.health.hurt.hurt_time,
                        p.bounds().max(Axis::Y) - p.bounds().min(Axis::Y),
                    )
                })
            } else if let Some(m) = self.mobs.iter().find(|m| !m.removed && m.id as i32 == id) {
                let (w, h, _) = m.dims();
                Some((w, m.hurt_time, h))
            } else if let Some(a) = self.armor_stands.iter().find(|a| a.id as i32 == id) {
                let (w, h, _) = if a.marker {
                    (0., 0., 0.)
                } else {
                    a.probe_dimensions()
                };
                Some((w, 0, h))
            } else if self
                .dragon
                .as_ref()
                .is_some_and(|_| self.client_targeting_dragon_id() == id)
            {
                Some((16., self.dragon.as_ref().unwrap().hurt_time, 8.))
            } else {
                self.lan
                    .iter()
                    .flat_map(|h| h.hub.guests.iter())
                    .find(|g| g.entity_id == id)
                    .and_then(|g| g.motion.as_ref())
                    .map(|p| {
                        (
                            f64::from(p.dimensions.width),
                            p.health.hurt.hurt_time,
                            p.bounds().max(Axis::Y) - p.bounds().min(Axis::Y),
                        )
                    })
            }
            .unwrap_or((0., 0, 0.));
            e["width"] = json!(width);
            e["hurt"] = json!(hurt);
            e["boundingHeight"] = json!(bounding);
        }
        for c in &self.crystals {
            entities.push(object(
                c.id,
                (u64::from(c.id) << 8) | 0x81,
                "minecraft:end_crystal",
                c.position,
                c.position,
                2.,
                2.,
                alpha,
                1,
                false,
            ));
        }
        for p in &self.creatures.projectiles {
            entities.push(object(
                p.id,
                (p.join.get() << 8) | 0x82,
                p.kind.id(),
                p.position,
                p.previous,
                p.kind.size(),
                p.kind.size(),
                alpha,
                2,
                false,
            ));
        }
        for t in &self.tridents {
            entities.push(object(
                t.id,
                (t.join.get() << 8) | 0x83,
                "minecraft:trident",
                t.position,
                t.previous,
                0.5,
                0.5,
                alpha,
                2,
                false,
            ));
        }
        for r in &self.rockets {
            entities.push(object(
                r.id,
                (r.join.get() << 8) | 0x84,
                "minecraft:firework_rocket",
                r.position,
                r.previous,
                0.25,
                0.25,
                alpha,
                2,
                false,
            ));
        }
        for b in &self.boats {
            let bounds = b.bounds();
            let lo = bounds.min_position();
            let hi = bounds.max_position();
            entities.push(object(
                b.id,
                (b.join.get() << 8) | 0x85,
                if b.chest {
                    "minecraft:chest_boat"
                } else {
                    "minecraft:boat"
                },
                b.position,
                b.previous,
                hi.x - lo.x,
                hi.y - lo.y,
                alpha,
                4,
                self.riding == Some(b.id),
            ));
        }
        for c in &self.carts {
            let bounds = c.bounds();
            let lo = bounds.min_position();
            let hi = bounds.max_position();
            entities.push(object(
                c.id,
                (c.join.get() << 8) | 0x86,
                c.kind.id(),
                c.position,
                c.previous,
                hi.x - lo.x,
                hi.y - lo.y,
                alpha,
                4,
                self.riding == Some(c.id),
            ));
        }
        entities.extend(self.client_esp_potions(alpha));
        entities.extend(self.client_esp_hooks(alpha));
        entities
    }
    fn client_esp_perspective(&self) -> i32 {
        match self.camera_type {
            super::perspective::CameraType::FirstPerson => 0,
            super::perspective::CameraType::ThirdPersonBack => 1,
            super::perspective::CameraType::ThirdPersonFront => 2,
        }
    }
    pub(super) fn client_esp_tick(&self) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let camera = self.native_camera().map(|c| c.position).unwrap_or_else(|| {
            let p = self.player.as_ref().unwrap();
            [p.position.x, p.position.y, p.position.z]
        });
        let input=json!({"tick":true,"perspective":self.client_esp_perspective(),"camera":camera,"entities":self.client_esp_entities(1.)}).to_string();
        self.native_client_esp(&input, &mut String::new());
    }
    pub(super) fn client_esp_frame(&self, camera: glam::DVec3, alpha: f64) {
        OVERLAY.with_borrow_mut(Vec::clear);
        horizon_client::gfx::esp_plain::begin_frame("null");
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let input=json!({"tick":false,"perspective":self.client_esp_perspective(),"camera":camera.to_array(),"yaw":self.yaw,"entities":self.client_esp_entities(alpha)}).to_string();
        let mut output = String::new();
        self.native_client_esp(&input, &mut output);
        if let Ok(v) = serde_json::from_str::<Value>(&output) {
            horizon_client::gfx::outline_glow::submit(
                horizon_client::gfx::outline_glow::Owner::Esp,
                &v["glow"].to_string(),
            );
            horizon_client::gfx::esp_plain::begin_frame(&v["plain"].to_string());
            OVERLAY.with_borrow_mut(|o| *o = v["overlay"].as_array().cloned().unwrap_or_default());
        }
    }
    pub(super) fn client_esp_reset(&self) {
        OVERLAY.with_borrow_mut(Vec::clear);
        horizon_client::gfx::esp_plain::begin_frame("null");
        self.native_client_esp("{\"phase\":\"clear\"}", &mut String::new());
    }
    pub(super) fn client_esp_overlay(&self, batch: &mut HudBatch) {
        let Some(camera) = self.native_camera() else {
            return;
        };
        let matrix = glam::Mat4::from_cols_array(&camera.projection)
            * glam::Mat4::from_cols_array(&camera.view);
        let mut entities = vec![];
        OVERLAY.with_borrow(|overlays| {
            for o in overlays {
                let Ok(bounds) = serde_json::from_value::<[f64; 6]>(o["bounds"].clone()) else {
                    continue;
                };
                let mut rect = [
                    f32::INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NEG_INFINITY,
                ];
                for i in 0..8 {
                    let p = std::array::from_fn::<_, 3, _>(|axis| {
                        bounds[axis + if i & (1 << axis) == 0 { 0 } else { 3 }]
                            - camera.position[axis]
                    });
                    let c = matrix * glam::Vec3::from_array(p.map(|v| v as f32)).extend(1.);
                    if !c.is_finite() || c.w <= 0. {
                        continue;
                    }
                    let x = (c.x / c.w + 1.) * batch.width * 0.5;
                    let y = (-c.y / c.w + 1.) * batch.height * 0.5;
                    rect[0] = rect[0].min(x);
                    rect[1] = rect[1].min(y);
                    rect[2] = rect[2].max(x);
                    rect[3] = rect[3].max(y);
                }
                if rect[2] > rect[0] && rect[3] > rect[1] {
                    let mut value = o.clone();
                    value["rect"] = json!(rect);
                    entities.push(value);
                }
            }
        });
        let input =
            json!({"phase":"overlay","guiScale":self.settings.gui_scale,"entities":entities})
                .to_string();
        let mut output = String::new();
        self.native_client_esp(&input, &mut output);
        if let Ok(rects) = serde_json::from_str::<Vec<Value>>(&output) {
            for r in rects {
                let Ok([x1, y1, x2, y2]) = serde_json::from_value::<[f32; 4]>(r["rect"].clone())
                else {
                    continue;
                };
                let argb = r["color"].as_u64().unwrap_or(0) as u32;
                let [a, r, g, b] = argb.to_be_bytes();
                batch.fill_quad(
                    &self.gui.atlas,
                    [[x1, y1], [x2, y1], [x2, y2], [x1, y2]],
                    u32::from_le_bytes([r, g, b, a]),
                );
            }
        }
    }
}
