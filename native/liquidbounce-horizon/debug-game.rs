// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game, Screen};
use horizon_client::{
    actions::Action,
    hud::{HudBatch, Rect, rgba},
    world_primitives::{PrimitiveShape, WorldPrimitives},
};
use serde_json::{Value, json};
fn num(v: &Value) -> f32 {
    v.as_f64().unwrap_or(0.) as f32
}
fn argb(v: &Value, fallback: u32) -> u32 {
    v.as_u64()
        .map(|v| v as u32)
        .or_else(|| v.as_i64().map(|v| v as u32))
        .unwrap_or(fallback)
}
fn tint(c: u32, alpha: f32) -> u32 {
    rgba(
        (c >> 16) as u8,
        (c >> 8) as u8,
        c as u8,
        ((c >> 24) as f32 * alpha.clamp(0., 1.)) as u8,
    )
}
impl Game<'_> {
    pub(super) fn client_debug_query(&self, input: Value) -> Value {
        let mut output = String::new();
        self.native_client_debug(&input.to_string(), &mut output);
        serde_json::from_str(&output).unwrap_or(Value::Null)
    }
    pub(super) fn client_debug_tick(&mut self) {
        if !self.native_window() {
            return;
        }
        let policy = self.client_debug_query(json!({"action":"policy"}));
        if policy["running"] != true {
            return;
        }
        let mut predicted = Vec::new();
        if policy["simulation"] == true && self.client_smart_eat_player() {
            let ticks = policy["ticks"].as_i64().unwrap_or(20).clamp(5, 100) as i32;
            let raw = horizon_client::reverse_step_bridge::keys();
            let keys = horizon_client::sprint::Keys {
                forward: raw[0],
                backward: raw[1],
                left: raw[2],
                right: raw[3],
                jump: raw[4],
                shift: raw[5],
                sprint: raw[6],
                analog: None,
            };
            let _ = super::falling_forecast::visit_with_keys(self, ticks, 0., keys, |p, _| {
                predicted.push([p.position.x,p.position.y,p.position.z]);
                false
            });
        }
        let parameters=self.player.as_ref().map(|p|json!({"Tick":p.health.tick_count,"Position":[p.position.x,p.position.y,p.position.z],"Velocity":[p.velocity.x,p.velocity.y,p.velocity.z],"Rotation":[p.yaw,p.pitch],"OnGround":p.ground.on_ground,"Chunks":self.world.chunks.len(),"Entities":self.mobs.len()})).unwrap_or_else(||json!({"Player":false}));
        self.client_debug_query(
            json!({"action":"tick","parameters":parameters,"predicted":predicted}),
        );
    }
    pub(super) fn client_debug_scroll(&self, amount: f64, cursor: [f32; 2]) -> bool {
        self.screen == Screen::Chat
            && self.client_debug_query(
                json!({"action":"scroll","chat":true,"amount":amount,"cursor":cursor}),
            ) == true
    }
    fn client_debug_frame(&self) -> Value {
        let (width, height) = self.gui_size();
        self.client_debug_query(json!({"action":"frame","width":width,"height":height,"tab":self.keys.held(Action::Playerlist)}))
    }
    pub(super) fn client_debug_overlay(&self, batch: &mut HudBatch) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let frame = self.client_debug_frame();
        let bottom = num(&frame["bottom"]);
        for card in frame["cards"].as_array().into_iter().flatten() {
            let x = num(&card["x"]);
            let y = num(&card["y"]);
            let width = num(&card["width"]);
            let height = num(&card["height"]);
            if y + height <= 22. || y >= bottom {
                continue;
            }
            batch.fill(
                &self.gui.atlas,
                x,
                y.max(22.),
                width,
                (y + height).min(bottom) - y.max(22.),
                tint(0x96000000, 1.),
            );
            let inner = width - 4.;
            let max_name = card["rows"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|r| r["type"] == "item")
                .map(|r| {
                    self.client_debug_width(r["name"].as_str().unwrap_or(""))
                        + if r["indent"] == true { 6. } else { 0. }
                })
                .fold(0., f32::max);
            let col = (max_name + 4.).min(inner * 0.55);
            for (i, row) in card["rows"].as_array().into_iter().flatten().enumerate() {
                let top = y + 2. + i as f32 * 8.;
                if top < 22. || top + 8. > bottom {
                    continue;
                }
                match row["type"].as_str() {
                    Some("header") => {
                        batch.fill(&self.gui.atlas, x, top, width, 8., tint(0x18ffffff, 1.));
                        self.client_debug_text(
                            batch,
                            row["text"].as_str().unwrap_or(""),
                            [x + 2., top],
                            inner,
                            0xffffaa00,
                            1.,
                        );
                    }
                    Some("subheader") => self.client_debug_text(
                        batch,
                        row["text"].as_str().unwrap_or(""),
                        [x + 2., top],
                        inner,
                        0xff55ffff,
                        1.,
                    ),
                    _ => {
                        let indent = if row["indent"] == true { 6. } else { 0. };
                        let alpha = num(&row["alpha"]);
                        self.client_debug_text(
                            batch,
                            row["name"].as_str().unwrap_or(""),
                            [x + 2. + indent, top],
                            col - indent - 4.,
                            0xffaaaaaa,
                            alpha,
                        );
                        self.client_debug_text(
                            batch,
                            row["value"].as_str().unwrap_or(""),
                            [x + 2. + col, top],
                            inner - col,
                            argb(&row["color"], u32::MAX),
                            alpha,
                        );
                    }
                }
            }
        }
        if frame["overflow"] == true {
            let width = num(&frame["contentRight"]) - 120.;
            batch.fill(
                &self.gui.atlas,
                120.,
                bottom,
                width,
                8.,
                tint(0x96000000, 1.),
            );
            let text = frame["status"].as_str().unwrap_or("");
            self.client_debug_text(batch, text, [122., bottom], width - 4., 0xff808080, 1.);
        }
        if let Some(graph) = frame.get("graph").filter(|g| !g.is_null()) {
            self.client_debug_text(
                batch,
                frame["graphLabel"].as_str().unwrap_or("Graph"),
                [300., 500.],
                180.,
                u32::MAX,
                1.,
            );
            for p in graph["samples"].as_array().into_iter().flatten() {
                batch.fill(
                    &self.gui.atlas,
                    300. + num(&p[0]),
                    500. - num(&p[1]),
                    1.,
                    1.,
                    tint(0xff00ff00, 1.),
                );
            }
            for p in graph["curve"].as_array().into_iter().flatten() {
                batch.fill(
                    &self.gui.atlas,
                    298. + num(&p[0]),
                    498. - num(&p[1]),
                    4.,
                    4.,
                    u32::MAX,
                );
            }
        }
    }
    fn client_debug_width(&self, text: &str) -> f32 {
        let (_, h) = self.gui.atlas.font.liquidbounce_target_metrics();
        text.chars()
            .map(|c| {
                self.gui
                    .atlas
                    .font
                    .liquidbounce_target_glyph(c, false)
                    .advance
            })
            .sum::<f32>()
            * 7.2
            / h
    }
    fn client_debug_text(
        &self,
        batch: &mut HudBatch,
        text: &str,
        at: [f32; 2],
        max: f32,
        color: u32,
        alpha: f32,
    ) {
        if max <= 0. {
            return;
        }
        let mut text = text.to_string();
        if self.client_debug_width(&text) > max {
            while !text.is_empty() && self.client_debug_width(&(text.clone() + "…")) > max {
                text.pop();
            }
            text.push('…');
            if self.client_debug_width(&text) > max {
                return;
            }
        }
        let (ascent, height) = self.gui.atlas.font.liquidbounce_target_metrics();
        let scale = 7.2 / height;
        let mut x = at[0];
        let y = at[1] + 0.5 + ascent * scale;
        for c in text.chars() {
            let g = self.gui.atlas.font.liquidbounce_target_glyph(c, false);
            if let Some(Rect { u0, v0, u1, v1, .. }) = g.rect {
                let (x0, x1, y0, y1) = (
                    x + g.left * scale,
                    x + g.right * scale,
                    y + g.top * scale,
                    y + g.bottom * scale,
                );
                batch.quad(
                    [[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
                    [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                    tint(color, alpha),
                );
            }
            x += g.advance * scale;
        }
    }
    pub(super) fn client_debug_geometry(&self, out: &mut WorldPrimitives) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let frame = self.client_debug_frame();
        let predicted =
            serde_json::from_value::<Vec<[f64; 3]>>(frame["predicted"].clone()).unwrap_or_default();
        for points in predicted.windows(2) {
            out.line(points[0], points[1], 0xff0000ff, 1.);
        }
        for geometry in frame["geometry"].as_array().into_iter().flatten() {
            Self::client_debug_shape(out, geometry);
        }
    }
    fn client_debug_shape(out: &mut WorldPrimitives, g: &Value) {
        let color = argb(&g["color"], u32::MAX);
        match g["type"].as_str() {
            Some("collection") => {
                for shape in g["geometry"].as_array().into_iter().flatten() {
                    Self::client_debug_shape(out, shape);
                }
            }
            Some("segment") => {
                if let (Ok(a), Ok(b)) = (
                    serde_json::from_value::<[f64; 3]>(g["from"].clone()),
                    serde_json::from_value::<[f64; 3]>(g["to"].clone()),
                ) {
                    out.line(a, b, color, 1.);
                }
            }
            Some("line") => {
                if let (Ok(p), Ok(d)) = (
                    serde_json::from_value::<[f64; 3]>(g["position"].clone()),
                    serde_json::from_value::<[f64; 3]>(g["direction"].clone()),
                ) {
                    let length = d.iter().map(|v| v * v).sum::<f64>().sqrt();
                    if length > 0. {
                        out.line(
                            std::array::from_fn(|i| p[i] - 100. * d[i] / length),
                            std::array::from_fn(|i| p[i] + 100. * d[i] / length),
                            color,
                            1.,
                        );
                    }
                } else if let (Ok(a), Ok(b)) = (
                    serde_json::from_value::<[f64; 3]>(g["from"].clone()),
                    serde_json::from_value::<[f64; 3]>(g["to"].clone()),
                ) {
                    out.line(a, b, color, 1.);
                }
            }
            Some("triangle") => {
                if let Ok(p) = serde_json::from_value::<[[f64; 3]; 3]>(g["points"].clone()) {
                    out.push(PrimitiveShape::LiquidBounceQuad {
                        corners: [p[0], p[1], p[2], p[2]],
                        colors: [color; 4],
                    });
                }
            }
            Some("box") => {
                if let (Ok(a), Ok(b)) = (
                    serde_json::from_value::<[f64; 3]>(g["min"].clone()),
                    serde_json::from_value::<[f64; 3]>(g["max"].clone()),
                ) {
                    out.filled_box(a, b, color);
                }
            }
            Some("point") => {
                if let Ok(p) = serde_json::from_value::<[f64; 3]>(g["position"].clone()) {
                    let half = g["size"].as_f64().unwrap_or(0.2) * 0.5;
                    out.filled_box(p.map(|v| v - half), p.map(|v| v + half), color);
                }
            }
            _ => {}
        }
    }
}
