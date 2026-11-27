// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{
    hud::{HudBatch, Rect},
    text::Text,
};
use serde_json::{Value, json};
thread_local! {static OVERLAY:std::cell::RefCell<Vec<Value>>=const{std::cell::RefCell::new(Vec::new())};}
fn at(p: horizon_core::geometry::Vec3) -> [f64; 3] {
    [p.x, p.y, p.z]
}
fn snapshot(
    id: i32,
    position: [f64; 3],
    old: [f64; 3],
    height: f64,
    health: f32,
    max: f32,
    absorption: f32,
    hurt: i32,
    age: i32,
    partial: f64,
) -> Value {
    json!({"id":id,"position":position,"interpolated":std::array::from_fn::<_,3,_>(|i|old[i]+(position[i]-old[i])*partial),"height":height,"health":health,"maxHealth":max,"absorption":absorption,"hurt":hurt,"age":age,"living":true})
}
fn native_color(argb: u32) -> u32 {
    let [a, r, g, b] = argb.to_be_bytes();
    u32::from_le_bytes([r, g, b, a])
}
fn line(
    batch: &mut HudBatch,
    atlas: &horizon_client::hud::HudAtlas,
    a: [f32; 2],
    b: [f32; 2],
    color: u32,
) {
    let d = glam::Vec2::from_array(b) - glam::Vec2::from_array(a);
    let n = glam::Vec2::new(-d.y, d.x).normalize_or_zero() * (0.5 / batch.scale);
    let a = glam::Vec2::from_array(a);
    let b = glam::Vec2::from_array(b);
    batch.fill_quad(
        atlas,
        [
            (a + n).to_array(),
            (b + n).to_array(),
            (b - n).to_array(),
            (a - n).to_array(),
        ],
        native_color(color),
    );
}
impl Game<'_> {
    pub(super) fn client_target_renderer_errors(&mut self) {
        if self.native_window() && self.native_client_slot_owner() {
            if let Some(error) = horizon_client::gfx::target_renderer::take_error() {
                horizon_client::mod_api::Host::chat(self, &error);
            }
        }
    }
    fn client_target_renderer_entity(&self, id: i32, partial: f64) -> Option<Value> {
        if self.remote.as_ref().map_or(0, |r| r.entity_id) == id {
            let p = self.player.as_ref()?;
            return Some(snapshot(
                id,
                at(p.position),
                at(self.previous_position),
                f64::from(p.dimensions.height),
                p.health.health(),
                p.clone().attribute("minecraft:max_health").unwrap_or(20.) as f32,
                p.health.absorption(),
                p.health.hurt.hurt_time,
                p.health.tick_count,
                partial,
            ));
        }
        if let Some(m) = self.mobs.iter().find(|m| !m.removed && m.id as i32 == id) {
            return Some(snapshot(
                id,
                at(m.position),
                at(m.previous),
                f64::from(m.dims().1),
                m.health,
                horizon_client::mobs::finalize::max_health(m),
                m.absorption,
                m.hurt_time,
                m.age as i32,
                partial,
            ));
        }
        if let Some(a) = self.armor_stands.iter().find(|a| a.id as i32 == id) {
            return Some(snapshot(
                id,
                at(a.position),
                at(a.previous),
                if a.marker { 0. } else { a.probe_dimensions().1 },
                a.health,
                20.,
                0.,
                0,
                a.liquidbounce_tick_count,
                partial,
            ));
        }
        if let Some(d) = self
            .dragon
            .as_ref()
            .filter(|_| self.client_targeting_dragon_id() == id)
        {
            return Some(snapshot(
                id,
                at(d.position),
                at(d.previous),
                8.,
                d.health,
                200.,
                0.,
                d.hurt_time,
                d.ticks,
                partial,
            ));
        }
        if let Some(g) = self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .find(|g| g.placed && g.dimension == self.dim.id() && g.entity_id == id)
        {
            let p = g.motion.as_ref()?;
            return Some(snapshot(
                id,
                g.position,
                g.previous_position,
                f64::from(p.dimensions.height),
                p.health.health(),
                p.clone().attribute("minecraft:max_health").unwrap_or(20.) as f32,
                p.health.absorption(),
                p.health.hurt.hurt_time,
                p.health.tick_count,
                partial,
            ));
        }
        None
    }
    pub(super) fn client_target_renderer_frame(&self, partial: f64) {
        OVERLAY.with_borrow_mut(Vec::clear);
        horizon_client::gfx::target_renderer::begin_frame("null");
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let mut ids = String::new();
        self.native_client_target_renderer("{\"phase\":\"targets\"}", &mut ids);
        let Ok(ids) = serde_json::from_str::<Vec<i32>>(&ids) else {
            return;
        };
        if ids.is_empty() {
            return;
        }
        let entities = ids
            .into_iter()
            .filter_map(|id| self.client_target_renderer_entity(id, partial))
            .collect::<Vec<_>>();
        let q = self.camera_orientation();
        let right = q * glam::Vec3::X;
        let up = q * glam::Vec3::Y;
        let forward = q * glam::Vec3::NEG_Z;
        let yaw = (-forward.x).atan2(forward.z).to_degrees();
        let pitch = (-forward.y).clamp(-1., 1.).asin().to_degrees();
        let input=json!({"phase":"frame","partial":partial,"entities":entities,"right":right.to_array(),"up":up.to_array(),"yaw":yaw,"pitch":pitch}).to_string();
        let mut geometry = String::new();
        self.native_client_target_renderer(&input, &mut geometry);
        if let Ok(frame) = serde_json::from_str::<Value>(&geometry) {
            OVERLAY.with_borrow_mut(|out| {
                *out = frame["overlay"].as_array().cloned().unwrap_or_default()
            });
        }
        horizon_client::gfx::target_renderer::begin_frame(&geometry);
    }
    pub(super) fn client_target_renderer_overlay(&self, batch: &mut HudBatch) {
        let Some(camera) = self.native_camera() else {
            return;
        };
        let matrix = glam::Mat4::from_cols_array(&camera.projection)
            * glam::Mat4::from_cols_array(&camera.view);
        OVERLAY.with_borrow(|overlays| {
            for o in overlays {
                let Ok(pos) = serde_json::from_value::<[f64; 3]>(o["position"].clone()) else {
                    continue;
                };
                let clip = matrix
                    * glam::Vec4::new(
                        (pos[0] - camera.position[0]) as f32,
                        (pos[1] - camera.position[1]) as f32,
                        (pos[2] - camera.position[2]) as f32,
                        1.,
                    );
                if !clip.is_finite() || clip.w <= 0. {
                    continue;
                }
                let screen = [
                    (clip.x / clip.w + 1.) * batch.width * 0.5,
                    (-clip.y / clip.w + 1.) * batch.height * 0.5,
                ];
                let scale = o["scale"].as_f64().unwrap_or(1.) as f32;
                let color = o["color"].as_i64().unwrap_or(-1) as u32;
                if o["arrow"] == true {
                    let [a, b, c] = [
                        [screen[0] - 5. * scale, screen[1] - 10. * scale],
                        screen,
                        [screen[0] + 5. * scale, screen[1] - 10. * scale],
                    ];
                    if color >> 24 != 0 {
                        batch.fill_quad(&self.gui.atlas, [a, b, c, c], native_color(color));
                    }
                    let outline = o["outline"].as_i64().unwrap_or(0) as u32;
                    if outline >> 24 != 0 {
                        for (a, b) in [(a, b), (b, c), (c, a)] {
                            line(batch, &self.gui.atlas, a, b, outline);
                        }
                    }
                    continue;
                }
                let (ascent, height) = self.gui.atlas.font.liquidbounce_target_metrics();
                let shadow = o["shadow"] == true;
                for (i, text) in o["text"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .enumerate()
                {
                    let text = Text::literal(text).resolve_legacy();
                    let glyphs = text
                        .spans
                        .iter()
                        .flat_map(|span| span.text.chars().map(move |c| (c, &span.style)))
                        .map(|(c, style)| {
                            (
                                self.gui
                                    .atlas
                                    .font
                                    .liquidbounce_target_glyph(c, style.bold == Some(true)),
                                style,
                            )
                        })
                        .collect::<Vec<_>>();
                    let width = glyphs.iter().map(|(g, _)| g.advance).sum::<f32>()
                        + if shadow { 2. } else { 0. };
                    let left = screen[0] - width * scale * 0.5;
                    let baseline =
                        screen[1] + i as f32 * height - height * scale * 0.5 + ascent * scale;
                    for is_shadow in [true, false] {
                        if is_shadow && !shadow {
                            continue;
                        }
                        let mut x = left + if is_shadow { 2. * scale } else { 0. };
                        let y = baseline + if is_shadow { 2. * scale } else { 0. };
                        for (g, style) in &glyphs {
                            let fill = if is_shadow {
                                0x96000000
                            } else {
                                (color & 0xff000000) | style.color.unwrap_or(color & 0xffffff)
                            };
                            if let Some(Rect { u0, v0, u1, v1, .. }) = g.rect {
                                let shear = if style.italic == Some(true) { 0.2 } else { 0. };
                                let x0 = x + g.left * scale;
                                let x1 = x + g.right * scale;
                                let y0 = y + g.top * scale;
                                let y1 = y + g.bottom * scale;
                                let top = -g.top * scale * shear;
                                let bottom = -g.bottom * scale * shear;
                                batch.quad(
                                    [
                                        [x0 + top, y0],
                                        [x1 + top, y0],
                                        [x1 + bottom, y1],
                                        [x0 + bottom, y1],
                                    ],
                                    [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                                    native_color(fill),
                                );
                            }
                            if style.underlined == Some(true) {
                                self.native_fill(
                                    batch,
                                    [x, y + 2. * scale, g.advance * scale, scale],
                                    fill,
                                );
                            }
                            if style.strikethrough == Some(true) {
                                self.native_fill(
                                    batch,
                                    [x, y - ascent * scale * 0.4, g.advance * scale, scale],
                                    fill,
                                );
                            }
                            x += g.advance * scale;
                        }
                    }
                }
            }
        });
    }
}
