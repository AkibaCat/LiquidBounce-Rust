// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use serde_json::{Value, json};
impl Game<'_> {
    pub(super) fn chest_stealer_context(&self) -> Option<String> {
        let mut v: Value = self
            .native_client_chest_cleaner_context()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}));
        v["abstract"] = json!(matches!(self.screen, Screen::Inventory | Screen::Creative));
        v["inventory"] =
            json!(self.screen == Screen::Inventory && self.client_inventory_player_menu());
        Some(v.to_string())
    }
    pub(super) fn client_chest_stealer_options(&self) -> Value {
        self.client_better_inventory_query(json!({"kind":"chestStealerQuery"}))
    }
    pub(super) fn client_chest_stealer_hidden(&self) -> bool {
        self.client_chest_stealer_options()["hidden"] == true
    }
    pub(super) fn native_client_chest_stealer_screen(&mut self, screen: Screen) -> Screen {
        let previous = self.screen;
        self.screen = screen;
        let mut value = self
            .chest_stealer_context()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({"abstract":false}));
        self.screen = previous;
        value["kind"] = json!("chestStealerScreen");
        self.client_better_inventory_query(value);
        screen
    }
    pub(super) fn client_chest_stealer_sent(&self, p: &horizon_protocol::packets::play::UseItemOn) {
        let mut positions = vec![p.pos];
        if let Some((other, _)) = self.chest_partner((p.pos[0], p.pos[1], p.pos[2])) {
            positions.push([other.0, other.1, other.2]);
        }
        self.client_better_inventory_query(
            json!({"kind":"chestStealerUseOn","positions":positions,"cancelled":false}),
        );
    }
    pub(super) fn chest_stealer_use_on(&mut self, payload: &str) -> bool {
        if !self.native_client_slot_owner() {
            return false;
        }
        let Ok(v) = serde_json::from_str::<Value>(payload) else {
            return false;
        };
        let Ok(pos) = serde_json::from_value::<[i32; 3]>(v["target"]["pos"].clone()) else {
            return false;
        };
        let Ok(rotation) = serde_json::from_value::<[f32; 2]>(v["rotation"].clone()) else {
            return false;
        };
        let Some(range) = v["range"].as_f64() else {
            return false;
        };
        let Some(hit) = self
            .auto_deposit_rotation_hit(pos, rotation, range)
            .filter(|h| h.pos == pos)
        else {
            return false;
        };
        self.client_chest_stealer_result = Some(false);
        self.client_chest_stealer_hide_swing = v["hideSwing"] == true;
        if self.remote.is_some() {
            self.remote_use_item_on_at(horizon_client::first_person::Hand::Main, Some(hit));
        } else {
            let previous_use = self.guest_use;
            self.guest_use = Some(GuestUse::OnBlock(horizon_client::first_person::Hand::Main));
            let _ = self.with_hand(horizon_client::first_person::Hand::Main, |g| {
                g.use_hand(Some(&hit))
            });
            self.guest_use = previous_use;
        }
        self.client_chest_stealer_hide_swing = false;
        self.client_chest_stealer_result.take().unwrap_or(false)
    }
    pub(super) fn client_chest_stealer_animation(
        &self,
    ) -> horizon_client::first_person::SwingAnimation {
        use horizon_client::first_person::SwingAnimation;
        let Some(stack) = self.held() else {
            return SwingAnimation::DEFAULT;
        };
        let view = horizon_client::item_render::stack_view(stack);
        let components = self.pack.item_renderer.models.components_view(&view);
        let Some(v) = components.get("minecraft:interact_animation") else {
            return SwingAnimation::DEFAULT;
        };
        SwingAnimation {
            kind: match v["type"].as_str().unwrap_or("whack") {
                "none" => 0,
                "stab" => 2,
                _ => 1,
            },
            duration: v["duration"].as_i64().unwrap_or(6) as i32,
        }
    }
    pub(super) fn client_chest_stealer_tag(&mut self, batch: &mut HudBatch) {
        let options = self.client_chest_stealer_options();
        if options["hidden"] != true || options["draw"] != true {
            return;
        }
        let Ok(pos) = serde_json::from_value::<[i32; 3]>(options["pos"].clone()) else {
            return;
        };
        let Ok(offset) = serde_json::from_value::<[f64; 3]>(options["offset"].clone()) else {
            return;
        };
        let Some(camera) = self.native_camera() else {
            return;
        };
        let mut center = pos.map(|v| f64::from(v) + 0.5);
        if let Some((other, _)) = self.chest_partner((pos[0], pos[1], pos[2])) {
            for (i, p) in [other.0, other.1, other.2].into_iter().enumerate() {
                center[i] = (center[i] + f64::from(p) + 0.5) * 0.5;
            }
        }
        for i in 0..3 {
            center[i] += offset[i];
        }
        let matrix = glam::Mat4::from_cols_array(&camera.projection)
            * glam::Mat4::from_cols_array(&camera.view);
        let p = matrix
            * glam::Vec4::new(
                (center[0] - camera.position[0]) as f32,
                (center[1] - camera.position[1]) as f32,
                (center[2] - camera.position[2]) as f32,
                1.,
            );
        if !p.is_finite() || p.w <= 0. {
            return;
        }
        let (cx, cy) = (
            (p.x / p.w + 1.) * batch.width * 0.5,
            (-p.y / p.w + 1.) * batch.height * 0.5,
        );
        let slots = self
            .client_inventory_mapping(false)
            .into_iter()
            .filter(|slot| slot.is_none_or(|i| !(i < 36 || (500..=504).contains(&i))))
            .map(|slot| slot.and_then(|i| self.slot_view(i)))
            .collect::<Vec<_>>();
        if slots.is_empty() {
            return;
        }
        let title = if options["title"] == true {
            self.chest_stealer_context()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .and_then(|v| v["title"].as_str().map(str::to_owned))
        } else {
            None
        };
        let title_height = if title.is_some() { 11. } else { 0. };
        let width = (slots.len().min(9) * 18) as f32;
        let height = (slots.len().div_ceil(9) * 18) as f32 + title_height;
        let (left, top) = (cx - width * 0.5, cy - height * 0.5);
        let first = batch.vertices.len();
        if options["texture"] != true {
            let margin = options["margin"].as_f64().unwrap_or(2.) as f32;
            let radius = options["radius"].as_f64().unwrap_or(4.) as f32;
            let color = options["color"].as_i64().unwrap_or(i64::from(i32::MIN)) as u32;
            let outline = options["outline"].as_i64().unwrap_or(0) as u32;
            let glow = options["glow"].as_f64().unwrap_or(0.3) as f32;
            let gradient = options["gradient"].as_f64().unwrap_or(0.3) as f32;
            if glow > 0. {
                let base = if outline >> 24 == 0 { color } else { outline };
                for layer in (1..=3).rev() {
                    let spread = layer as f32;
                    let alpha = ((base >> 24) as f32 * glow / layer as f32) as u32;
                    self.chest_stealer_panel(
                        batch,
                        left - margin - spread,
                        top - margin - spread,
                        width + 2. * (margin + spread),
                        height + 2. * (margin + spread),
                        radius + spread,
                        (base & 0x00ff_ffff) | (alpha << 24),
                        0,
                        0.,
                    );
                }
            }
            self.chest_stealer_panel(
                batch,
                left - margin,
                top - margin,
                width + margin * 2.,
                height + margin * 2.,
                radius,
                color,
                outline,
                gradient,
            );
        }
        if let Some(title) = title {
            let tw = HudBatch::text_width(&self.gui.atlas, &title);
            batch.text(
                &self.gui.atlas,
                &title,
                left + (width - tw) * 0.5,
                top,
                u32::MAX,
                true,
            );
        }
        self.client_better_inventory_origin
            .set((left, top + title_height));
        for (i, stack) in slots.into_iter().enumerate() {
            let (x, y) = (
                left + (i % 9) as f32 * 18.,
                top + title_height + (i / 9) as f32 * 18.,
            );
            if options["texture"] == true {
                batch.sprite_scaled(
                    &self.gui.atlas,
                    "gui/sprites/container/slot",
                    x,
                    y,
                    18.,
                    18.,
                    u32::MAX,
                );
            }
            if let Some(stack) = stack {
                self.draw_item(batch, &stack, x + 1., y + 1.);
            }
        }
        self.client_better_inventory_origin.set((0., 0.));
        let scale = options["scale"].as_f64().unwrap_or(1.5) as f32;
        batch.scale_since(first, cx, cy, scale, scale);
    }
    fn chest_stealer_panel(
        &self,
        batch: &mut HudBatch,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        color: u32,
        outline: u32,
        gradient: f32,
    ) {
        if ![x, y, w, h, r, gradient].iter().all(|v| v.is_finite()) || w <= 0. || h <= 0. {
            return;
        }
        let r = r.max(0.).min(w * 0.5).min(h * 0.5);
        let rgba_color = |c: u32, t: f32| {
            rgba(
                (((c >> 16) & 255) as f32 * (1. - gradient * t)) as u8,
                (((c >> 8) & 255) as f32 * (1. - gradient * t)) as u8,
                ((c & 255) as f32 * (1. - gradient * t)) as u8,
                (c >> 24) as u8,
            )
        };
        let edge = rgba_color(outline, 0.);
        for row in 0..h.ceil().min(4096.) as usize {
            let at = row as f32;
            let thickness = (h - at).min(1.);
            let dy = if at + 0.5 < r {
                r - at - 0.5
            } else if at + 0.5 > h - r {
                at + 0.5 - (h - r)
            } else {
                0.
            };
            let inset = if dy > 0. {
                r - (r * r - dy * dy).max(0.).sqrt()
            } else {
                0.
            };
            let ww = (w - 2. * inset).max(0.);
            if outline >> 24 != 0 {
                batch.fill(&self.gui.atlas, x + inset, y + at, ww, thickness, edge);
            }
            let border = if outline >> 24 != 0 { 1. } else { 0. };
            if row as f32 >= border && (row as f32 + 1.) <= h - border {
                batch.fill(
                    &self.gui.atlas,
                    x + inset + border,
                    y + at,
                    (ww - border * 2.).max(0.),
                    thickness,
                    rgba_color(color, at / h),
                );
            }
        }
    }
}
