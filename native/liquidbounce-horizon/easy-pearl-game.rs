// SPDX-License-Identifier: GPL-3.0-or-later
//! Real Player.pick OUTLINE/Fluid.NONE, feet-origin projectile solve and original overlay.
use super::Game;
use horizon_client::{
    hud::{HudBatch, Rect},
    world_primitives::{PrimitiveShape, WorldPrimitives},
};
use horizon_core::geometry::Vec3;
use serde_json::{Value, json};
thread_local! {static OVERLAY:std::cell::RefCell<Option<([f64;3],String,u32)>>=const{std::cell::RefCell::new(None)};}
thread_local! {static LOCAL_PREUSED:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
fn native_color(c: u32) -> u32 {
    u32::from_le_bytes([(c >> 16) as u8, (c >> 8) as u8, c as u8, (c >> 24) as u8])
}
fn color(reachable: bool, alpha: u8) -> u32 {
    (u32::from(alpha) << 24) | if reachable { 0x20c206 } else { 0xd70909 }
}
impl Game<'_> {
    fn client_easy_pearl_holding(&self) -> bool {
        let pearl = |s: Option<&horizon_client::items::ItemStack>| {
            s.is_some_and(|s| s.count > 0 && s.item.as_ref() == "minecraft:ender_pearl")
        };
        pearl(self.stack_in(self.client_hotbar_slot())) || pearl(self.offhand.as_ref())
    }
    fn client_easy_pearl_options(&self) -> Value {
        if !self.native_window() || !self.native_client_slot_owner() || self.player.is_none() {
            return Value::Null;
        }
        let mut output = String::new();
        self.native_client_easy_pearl("{\"phase\":\"options\"}", &mut output);
        serde_json::from_str(&output).unwrap_or(Value::Null)
    }
    fn client_easy_pearl_hit(&self, partial: f32) -> Option<(bool, Vec3)> {
        let player = self.player.as_ref()?;
        let old = self.previous_position;
        let t = f64::from(partial);
        let eye = Vec3::new(
            old.x + (player.position.x - old.x) * t,
            old.y + (player.position.y - old.y) * t + f64::from(player.dimensions.eye_height),
            old.z + (player.position.z - old.z) * t,
        );
        let [yaw, pitch] = if partial == 1. {
            [player.yaw, player.pitch]
        } else {
            [
                self.rope_rot_old.0 + (player.yaw - self.rope_rot_old.0) * partial,
                self.rope_rot_old.1 + (player.pitch - self.rope_rot_old.1) * partial,
            ]
        };
        let direction = Vec3::direction_from_rotation(pitch, yaw);
        super::auto_pearl_ray::clip(
            self,
            eye,
            eye.add_vector(direction.scale(1000.)),
            self.remote.as_ref().map_or(0, |r| r.entity_id),
            true,
        )
    }
    pub(super) fn client_easy_pearl_query(&self, text: &str) -> String {
        if !self.native_window() || !self.native_client_slot_owner() {
            return "null".into();
        }
        let Ok(q) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        match q["op"].as_str() {
            Some("slot") => json!(self.native_client_middle_click_pearl_slot()).to_string(),
            Some("solve") => {
                let Ok(p) = serde_json::from_value::<[f64; 3]>(q["target"].clone()) else {
                    return "null".into();
                };
                json!(self.auto_pearl_rotation(super::auto_bow_math::V(p[0], p[1], p[2]), true))
                    .to_string()
            }
            Some("angle") => self.auto_pearl_query(text),
            _ => "null".into(),
        }
    }
    pub(super) fn client_easy_pearl_interact(&mut self) -> bool {
        if LOCAL_PREUSED.with(std::cell::Cell::get)
            || !self.native_client_slot_owner()
            || !self.client_easy_pearl_holding()
            || !self.client_use_key_down()
        {
            return false;
        }
        if self.client_easy_pearl_options()["enabled"] != true {
            return false;
        }
        let Some((miss, hit)) = self.client_easy_pearl_hit(0.) else {
            return false;
        };
        let rotation = self.auto_pearl_rotation(super::auto_bow_math::V(hit.x, hit.y, hit.z), true);
        let mut result = String::new();
        self.native_client_easy_pearl(&json!({"phase":"interact","holding":true,"useDown":true,"hit":[hit.x,hit.y,hit.z],"miss":miss,"rotation":rotation}).to_string(),&mut result);
        let result = serde_json::from_str::<Value>(&result).unwrap_or(Value::Null);
        if let Some(warning) = result["warning"].as_str().filter(|s| !s.is_empty()) {
            horizon_client::mod_api::Host::chat(self, &format!("§c{warning}"));
        }
        result["cancel"] == true
    }
    /// The useItem HEAD observes the real player view before prediction temporarily
    /// aligns Item.use with its supplied packet angles.
    pub(super) fn client_easy_pearl_local_use(
        &mut self,
        hand: super::Hand,
        yaw: f32,
        pitch: f32,
    ) -> bool {
        if self.client_easy_pearl_interact() {
            self.native_client_item_interacted(true);
            return false;
        }
        let old = LOCAL_PREUSED.with(|s| s.replace(true));
        let result = self.native_with_item_rotation(yaw, pitch, |g| {
            g.with_hand(hand, |g| {
                g.client_use_item_only(true).unwrap_or(false) && !g.use_failed
            })
        });
        LOCAL_PREUSED.with(|s| s.set(old));
        result
    }
    pub(super) fn client_easy_pearl_geometry(&self, out: &mut WorldPrimitives, partial: f64) {
        OVERLAY.with_borrow_mut(|o| *o = None);
        let options = self.client_easy_pearl_options();
        if options["enabled"] != true || !self.client_easy_pearl_holding() {
            return;
        }
        let Some((_, hit)) = self.client_easy_pearl_hit(partial as f32) else {
            return;
        };
        let pos = [hit.x.floor(), hit.y.floor(), hit.z.floor()];
        let state = self
            .world
            .block(pos[0] as i32, pos[1] as i32, pos[2] as i32);
        let reachable = self
            .auto_pearl_rotation(super::auto_bow_math::V(hit.x, hit.y, hit.z), true)
            .is_some();
        let fill = color(reachable, 50);
        let outline = color(reachable, 200);
        let corners: [[f64; 3]; 8] = std::array::from_fn(|i| {
            [
                pos[0] + f64::from(((i & 1) != 0) as u8),
                pos[1] + f64::from(((i & 2) != 0) as u8),
                pos[2] + f64::from(((i & 4) != 0) as u8),
            ]
        });
        if self.res.facts().is_air(state) {
            let bottom = [0, 4, 5, 1];
            out.push(PrimitiveShape::LiquidBounceQuad {
                corners: bottom.map(|i| corners[i]),
                colors: [fill; 4],
            })
            .through_walls();
            for j in 0..4 {
                out.line(
                    corners[bottom[j]],
                    corners[bottom[(j + 1) % 4]],
                    outline,
                    1.,
                )
                .through_walls();
            }
            for (a, b) in [(0, 1), (1, 5), (5, 4), (4, 0)] {
                let mut ta = corners[a];
                ta[1] += 0.7;
                let mut tb = corners[b];
                tb[1] += 0.7;
                out.push(PrimitiveShape::LiquidBounceQuad {
                    corners: [corners[a], corners[b], tb, ta],
                    colors: [fill, fill, color(reachable, 0), color(reachable, 0)],
                })
                .through_walls();
            }
        } else {
            for face in [
                [0, 1, 3, 2],
                [4, 6, 7, 5],
                [0, 4, 5, 1],
                [2, 3, 7, 6],
                [0, 2, 6, 4],
                [1, 5, 7, 3],
            ] {
                out.push(PrimitiveShape::LiquidBounceQuad {
                    corners: face.map(|i| corners[i]),
                    colors: [fill; 4],
                })
                .through_walls();
            }
            for i in 0..8 {
                for axis in [1, 2, 4] {
                    if i & axis == 0 {
                        out.line(corners[i], corners[i | axis], outline, 1.)
                            .through_walls();
                    }
                }
            }
        }
        if options["showDistance"] == true {
            if let Some(p) = self.player.as_ref() {
                let distance = p.position.subtract(hit).length();
                OVERLAY.with_borrow_mut(|o| {
                    *o = Some((
                        [hit.x, hit.y + f64::from(p.dimensions.eye_height), hit.z],
                        format!("{distance:.1}m"),
                        color(reachable, 255),
                    ))
                });
            }
        }
    }
    pub(super) fn client_easy_pearl_overlay(&self, batch: &mut HudBatch) {
        let options = self.client_easy_pearl_options();
        if options["enabled"] != true
            || options["showDistance"] != true
            || !self.client_easy_pearl_holding()
        {
            return;
        }
        let Some(camera) = self.native_camera() else {
            return;
        };
        OVERLAY.with_borrow(|o| {
            let Some((pos, text, color)) = o else {
                return;
            };
            let matrix = glam::Mat4::from_cols_array(&camera.projection)
                * glam::Mat4::from_cols_array(&camera.view);
            let clip = matrix
                * glam::Vec4::new(
                    (pos[0] - camera.position[0]) as f32,
                    (pos[1] - camera.position[1]) as f32,
                    (pos[2] - camera.position[2]) as f32,
                    1.,
                );
            if !clip.is_finite() || clip.w <= 0. {
                return;
            }
            let screen = [
                (clip.x / clip.w + 1.) * batch.width * 0.5,
                (-clip.y / clip.w + 1.) * batch.height * 0.5,
            ];
            let (ascent, height) = self.gui.atlas.font.liquidbounce_target_metrics();
            let scale = 9. / height;
            let glyphs = text
                .chars()
                .map(|c| self.gui.atlas.font.liquidbounce_target_glyph(c, false))
                .collect::<Vec<_>>();
            let width = glyphs.iter().map(|g| g.advance).sum::<f32>() + 2.;
            let left = screen[0] - width * scale * 0.5;
            let baseline = screen[1] - height * scale * 0.5 + ascent * scale;
            for shadow in [true, false] {
                let mut x = left + if shadow { 2. * scale } else { 0. };
                let y = baseline + if shadow { 2. * scale } else { 0. };
                let fill = native_color(if shadow { 0x96000000 } else { *color });
                for g in &glyphs {
                    if let Some(Rect { u0, v0, u1, v1, .. }) = g.rect {
                        let x0 = x + g.left * scale;
                        let x1 = x + g.right * scale;
                        let y0 = y + g.top * scale;
                        let y1 = y + g.bottom * scale;
                        batch.quad(
                            [[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
                            [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                            fill,
                        );
                    }
                    x += g.advance * scale;
                }
            }
        });
    }
}
