// SPDX-License-Identifier: GPL-3.0-or-later
use super::{
    Game,
    commands::Target,
    indicator::{self, Arrow, Entry, Primitive},
};
use horizon_client::{
    hud::HudBatch,
    text::{InlineObject, Style, Text},
};
use serde_json::{Value, json};
thread_local! {static FRAME:std::cell::RefCell<Vec<Value>>=const{std::cell::RefCell::new(Vec::new())};}
pub(super) fn clear() {
    FRAME.with_borrow_mut(Vec::clear);
}
impl Game<'_> {
    pub(super) fn client_radar_tick(&self) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let input = json!({"tick":true,"perspective":self.client_tracers_perspective(),"entities":self.client_tracers_entities(1.)});
        self.native_client_radar(&input.to_string(), &mut String::new());
    }
    pub(super) fn client_radar_frame(&self, alpha: f64, camera: glam::DVec3) {
        let input = json!({"tick":false,"perspective":self.client_tracers_perspective(),"camera":camera.to_array(),"entities":self.client_tracers_entities(alpha)});
        let mut output = "[]".to_owned();
        self.native_client_radar(&input.to_string(), &mut output);
        FRAME.with_borrow_mut(|frame| *frame = serde_json::from_str(&output).unwrap_or_default());
    }
    fn client_radar_name(&self, id: i32) -> Option<(Text, Option<Text>)> {
        let player_name = if let Some(g) = self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .find(|g| g.placed && g.entity_id == id && g.dimension == self.dim.id())
        {
            Some((g.name.clone(), Some(g.uuid)))
        } else {
            self.mobs
                .iter()
                .find(|m| {
                    !m.removed
                        && m.id as i32 == id
                        && horizon_client::command::args::normalize(&m.kind.id)
                            == "minecraft:player"
                })
                .map(|m| {
                    (
                        self.remote_player_name(m.id)
                            .map_or_else(|| m.command_name().plain(), str::to_owned),
                        self.remote_player_uuid(m.id).or_else(|| {
                            self.persist
                                .uuids
                                .get(&m.id)
                                .map(|u| horizon_client::lan::playerdata::ints_uuid(*u))
                        }),
                    )
                })
        };
        if let Some((name, uuid)) = player_name {
            let shown = self.score.board.players_team(&name).map_or_else(
                || Text::literal(&name),
                |t| t.formatted_name(Text::literal(&name)),
            );
            let mut profile = json!({"name":name});
            if let Some(id) = uuid {
                profile["id"] = json!(horizon_protocol::crypto::uuid_string(id));
            }
            let head = Text::styled(
                "\u{fffc}",
                Style {
                    inline_object: Some(InlineObject::Player {
                        profile,
                        hat: false,
                        fallback: String::new(),
                    }),
                    ..Style::default()
                },
            );
            return Some((shown, Some(head)));
        }
        let target = if self.mobs.iter().any(|m| !m.removed && m.id as i32 == id) {
            Target::Mob(id as u32)
        } else if self.armor_stands.iter().any(|a| a.id as i32 == id) {
            Target::Stand(id as u32)
        } else if let Some(d) = self
            .dragon
            .as_ref()
            .filter(|_| self.client_targeting_dragon_id() == id)
        {
            Target::Dragon(d.id)
        } else {
            return None;
        };
        let name = match target {
            Target::Stand(id) => {
                let a = self.armor_stands.iter().find(|a| a.id == id)?;
                let raw = a
                    .custom_name
                    .clone()
                    .unwrap_or_else(|| Text::tr("entity.minecraft.armor_stand", &[]));
                self.mob_team(id)
                    .map_or_else(|| raw.clone(), |t| t.formatted_name(raw.clone()))
            }
            Target::Dragon(id) => {
                let raw = self.target_name(target);
                self.mob_team(id)
                    .map_or_else(|| raw.clone(), |t| t.formatted_name(raw.clone()))
            }
            _ => self.target_name(target),
        };
        Some((name, None))
    }
    pub(super) fn client_radar_submit(&self, _batch: &mut HudBatch) {
        FRAME.with_borrow(|frame| {
            for value in frame {
                let Some(id) = value["id"].as_i64().map(|v| v as i32) else {
                    continue;
                };
                let Some((name, head)) = self.client_radar_name(id) else {
                    continue;
                };
                let Ok(position) = serde_json::from_value(value["position"].clone()) else {
                    continue;
                };
                let atlas = &self.gui.atlas;
                let head_width = head
                    .as_ref()
                    .map_or(0., |h| atlas.font.width_text(h) + atlas.font.width(" "));
                let width = head_width + atlas.font.width_text(&name);
                let half_height = 4.5;
                let color = value["color"].as_u64().unwrap_or(0xffffffff) as u32;
                let mut tag = vec![];
                let x = -width * 0.5;
                let y: f32 = -half_height;
                if let Some(head) = head {
                    tag.push(Primitive::Text {
                        text: head,
                        origin: [x.trunc(), y.trunc()],
                        scale: 1.,
                        color: u32::MAX,
                        shadow: true,
                    });
                }
                tag.push(Primitive::Text {
                    text: name,
                    origin: [(x + head_width).trunc(), y.trunc()],
                    scale: 1.,
                    color,
                    shadow: true,
                });
                let pointer = value["pointer"].as_str().unwrap_or("Triangle");
                let make_style = || -> Option<Arrow> {
                    if pointer == "Triangle" {
                        Some(Arrow::Triangle {
                            width: value["width"].as_f64()? as f32,
                            height: value["height"].as_f64()? as f32,
                            notch: value["notch"].as_f64()? as f32,
                        })
                    } else {
                        let sprite = if pointer == "Image1" {
                            "liquidbounce/radar_image1"
                        } else {
                            "liquidbounce/radar_image2"
                        };
                        let r = atlas.sprites.get(sprite)?;
                        Some(Arrow::Atlas {
                            size: value["size"].as_f64()? as f32,
                            uv: [[r.u1, r.v1], [r.u0, r.v1], [r.u0, r.v0], [r.u1, r.v0]],
                        })
                    }
                };
                let (Some(style), Some(in_view_style)) = (make_style(), make_style()) else {
                    continue;
                };
                let show = value["show"].as_u64().unwrap_or(3);
                indicator::submit(Entry {
                    position,
                    priority: value["priority"].as_i64().unwrap_or(0) as i32,
                    color,
                    visible: [show & 1 != 0, show & 2 != 0],
                    edge_inset: value["edgeInset"].as_f64().unwrap_or(0.) as f32,
                    style,
                    in_view_style,
                    bounds: [-width * 0.5, -half_height, width * 0.5, half_height],
                    tag,
                });
            }
        });
    }
}
