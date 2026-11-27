// SPDX-License-Identifier: GPL-3.0-or-later
use super::{
    Game,
    indicator::{self, Arrow, Entry, Primitive},
};
use horizon_client::{hud::HudBatch, mobs::Who, text::Text};
use serde_json::{Value, json};
impl Game<'_> {
    fn client_tnt_timer_owner(&self, t: &super::tnt::PrimedTnt) -> Option<Text> {
        match t.owner? {
            Who::Player => {
                if let Some(conn) = t.guest {
                    return self
                        .lan
                        .as_ref()?
                        .hub
                        .guests
                        .iter()
                        .find(|g| g.conn == conn)
                        .map(|g| Text::literal(&g.name));
                }
                Some(Text::literal(self.acting_player_name()))
            }
            Who::Mob(id) => {
                if let Some(name) = self.remote_player_name(id) {
                    return Some(Text::literal(name));
                }
                self.mobs
                    .iter()
                    .find(|m| m.id == id && !m.removed)
                    .map(|m| m.command_name())
            }
        }
        .map(|t| t.resolve_legacy())
    }
    pub(super) fn client_tnt_timer_glow(&self, fuse: i32) -> Option<u32> {
        if !self.native_window() {
            return None;
        }
        let mut output = String::new();
        self.native_client_tnt_timer(
            &json!({"action":"glow","fuse":fuse}).to_string(),
            &mut output,
        );
        serde_json::from_str::<u32>(&output).ok()
    }
    pub(super) fn client_tnt_timer_submit(&self, batch: &mut HudBatch) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let Some(camera) = self.native_camera() else {
            return;
        };
        let entities=self.tnts.iter().map(|t|{
            let center=t.bounds().center();
            json!({"id":t.id,"center":[center.x,center.y,center.z],"fuse":t.fuse,"owner":self.client_tnt_timer_owner(t).map(|t|t.component())})
        }).collect::<Vec<_>>();
        let mut output = String::new();
        self.native_client_tnt_timer(
            &json!({"action":"frame","camera":camera.position,"entities":entities}).to_string(),
            &mut output,
        );
        let Ok(frame) = serde_json::from_str::<Value>(&output) else {
            return;
        };
        for label in frame["labels"].as_array().into_iter().flatten() {
            let Ok(position) = serde_json::from_value(label["position"].clone()) else {
                continue;
            };
            let mut text = Text::literal(label["text"].as_str().unwrap_or_default());
            if !label["owner"].is_null() {
                text = text
                    .append(" (")
                    .append(horizon_client::scoreboard::comp_text(&label["owner"]))
                    .append(")");
            }
            let width = self.gui.atlas.font.width_text(&text).ceil();
            let scale = label["scale"].as_f64().unwrap_or(1.) as f32;
            let color = label["color"].as_u64().unwrap_or(0) as u32;
            indicator::submit(Entry {
                position,
                priority: label["priority"].as_i64().unwrap_or(0) as i32,
                color,
                visible: serde_json::from_value(label["visible"].clone()).unwrap_or([true, true]),
                edge_inset: 0.,
                style: Arrow::Triangle {
                    width: 8.,
                    height: 10.,
                    notch: 0.,
                },
                in_view_style: Arrow::None,
                bounds: [
                    -width * 0.5 * scale,
                    -4.5 * scale,
                    width * 0.5 * scale,
                    4.5 * scale,
                ],
                tag: vec![Primitive::Text {
                    text,
                    origin: [-(width as i32 / 2) as f32, -4.],
                    scale,
                    color,
                    shadow: true,
                }],
            });
        }
    }
}
