// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::world_primitives::{PrimitiveShape, WorldPrimitives};
use horizon_protocol::packets::play::Clientbound as C;
use serde_json::{Value, json};
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |v| v.as_millis() as i64)
}
impl Game<'_> {
    fn client_placement_esp_box(&self, position: [i32; 3]) -> Value {
        let state = self.world.block(position[0], position[1], position[2]);
        let f = self.res.facts().get(state);
        let offset = if f.outline_offset {
            self.pack
                .context
                .tables
                .offset(state, position[0], position[2])
                .unwrap_or([0.; 3])
        } else {
            [0.; 3]
        };
        let local = if self.res.facts().is_air(state) {
            vec![[0., 0., 0., 1., 1., 1.]]
        } else if f.name.ends_with("shulker_box") {
            self.context_outline(position, state).into_iter().collect()
        } else {
            f.outline.iter().copied().collect::<Vec<_>>()
        };
        let boxes = local
            .into_iter()
            .map(|b| std::array::from_fn(|i| f64::from(position[i % 3]) + offset[i % 3] + b[i]))
            .collect::<Vec<[f64; 6]>>();
        let mut b = boxes.first().copied().unwrap_or_else(|| {
            std::array::from_fn(|i| f64::from(position[i % 3]) + if i < 3 { 0. } else { 1. })
        });
        for shape in &boxes[1.min(boxes.len())..] {
            for i in 0..3 {
                b[i] = b[i].min(shape[i]);
                b[i + 3] = b[i + 3].max(shape[i + 3]);
            }
        }
        let b: [f64; 6] = std::array::from_fn(|i| b[i] - f64::from(position[i % 3]));
        json!({"position":position,"bounds":b})
    }
    pub(super) fn client_placement_esp_packet(&self, packet: &C) {
        if !self.native_window() {
            return;
        }
        let (source, positions) = match packet {
            C::BlockUpdate(p) => (2, vec![p.pos]),
            C::SectionBlocksUpdate(p) => {
                let [sx, sy, sz] = p.section;
                (
                    4,
                    p.changes
                        .iter()
                        .map(|change| {
                            let local = (change & 0xfff) as i32;
                            [
                                sx.wrapping_mul(16).wrapping_add(local >> 8 & 15),
                                sy.wrapping_mul(16).wrapping_add(local & 15),
                                sz.wrapping_mul(16).wrapping_add(local >> 4 & 15),
                            ]
                        })
                        .collect(),
                )
            }
            _ => return,
        };
        let mut options = String::new();
        self.native_client_placement_esp("{\"action\":\"options\"}", &mut options);
        let Ok(options) = serde_json::from_str::<Value>(&options) else {
            return;
        };
        if options["prophunt"].as_u64().unwrap_or(0) & source == 0 {
            return;
        }
        let blocks = positions
            .into_iter()
            .map(|p| self.client_placement_esp_box(p))
            .collect::<Vec<_>>();
        self.native_client_placement_esp(
            &json!({"action":"prophunt","source":source,"time":now(),"blocks":blocks}).to_string(),
            &mut String::new(),
        );
    }
    pub(super) fn client_placement_esp_tick(&mut self) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let mut options = String::new();
        self.native_client_placement_esp("{\"action\":\"options\",\"tick\":true}", &mut options);
        let Ok(options) = serde_json::from_str::<Value>(&options) else {
            return;
        };
        let blocks = if options["prophunt"].as_u64().unwrap_or(0) & 1 != 0 {
            self.falling
                .iter()
                .map(|f| {
                    self.client_placement_esp_box([
                        f.position.x.floor() as i32,
                        f.position.y.floor() as i32,
                        f.position.z.floor() as i32,
                    ])
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        self.native_client_placement_esp(
            &json!({"action":"prophunt","source":1,"tick":true,"time":now(),"blocks":blocks})
                .to_string(),
            &mut String::new(),
        );
    }
    pub(super) fn client_placement_esp_player_tick(&mut self) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let mut options = String::new();
        self.native_client_placement_esp("{\"action\":\"options\"}", &mut options);
        let Ok(options) = serde_json::from_str::<Value>(&options) else {
            return;
        };
        if !options["void"].is_object() {
            return;
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let player = [
            p.position.x.floor() as i32,
            p.position.y.floor() as i32,
            p.position.z.floor() as i32,
        ];
        let facing = [[0, 1], [-1, 0], [0, -1], [1, 0]]
            [(((f64::from(p.yaw / 90.) + 0.5).floor() as i32) & 3) as usize];
        let side = options["void"]["side"].as_i64().unwrap_or(3) as i32;
        let range = options["void"]["facing"].as_i64().unwrap_or(8) as i32;
        let threshold = options["void"]["threshold"].as_i64().unwrap_or(16) as i32;
        let min_y = self.world.layout().min_y();
        let positions = self.with_level(|b, level, ticks| {
            super::placement_esp_search::search(
                player,
                facing,
                side,
                range,
                threshold,
                min_y,
                |p| {
                    b.client_placement_esp_sturdy(
                        level,
                        ticks,
                        horizon_core::position::BlockPos::new(p[0], p[1], p[2]),
                    )
                },
            )
        });
        let blocks = positions
            .into_iter()
            .map(|p| self.client_placement_esp_box(p))
            .collect::<Vec<_>>();
        self.native_client_placement_esp(
            &json!({"action":"void","time":now(),"blocks":blocks}).to_string(),
            &mut String::new(),
        );
    }
    pub(super) fn client_placement_esp_geometry(&self, out: &mut WorldPrimitives) {
        let mut result = String::new();
        self.native_client_placement_esp(
            &json!({"action":"frame","time":now()}).to_string(),
            &mut result,
        );
        let Ok(Value::Array(draws)) = serde_json::from_str(&result) else {
            return;
        };
        for d in draws {
            let (Ok(p), Ok(b)) = (
                serde_json::from_value::<[i32; 3]>(d["position"].clone()),
                serde_json::from_value::<[f64; 6]>(d["bounds"].clone()),
            ) else {
                continue;
            };
            let b: [f64; 6] = std::array::from_fn(|i| f64::from(p[i % 3]) + b[i]);
            let [x, y, z, X, Y, Z] = b;
            let faces = d["faces"].as_u64().unwrap_or(0) as u32;
            let edges = d["edges"].as_u64().unwrap_or(0) as u32;
            let color = d["color"].as_u64().unwrap_or(0) as u32;
            let outline = d["outline"].as_u64().unwrap_or(0) as u32;
            if color >> 24 != 0 {
                for (i, corners) in [
                    [[x, y, z], [X, y, z], [X, y, Z], [x, y, Z]],
                    [[x, Y, z], [x, Y, Z], [X, Y, Z], [X, Y, z]],
                    [[x, y, z], [x, Y, z], [X, Y, z], [X, y, z]],
                    [[X, y, z], [X, Y, z], [X, Y, Z], [X, y, Z]],
                    [[x, y, Z], [X, y, Z], [X, Y, Z], [x, Y, Z]],
                    [[x, y, z], [x, y, Z], [x, Y, Z], [x, Y, z]],
                ]
                .into_iter()
                .enumerate()
                {
                    if faces & (15 << (i * 4)) != 0 {
                        out.push(PrimitiveShape::LiquidBounceQuad {
                            corners,
                            colors: [color; 4],
                        })
                        .through_walls();
                    }
                }
            }
            if outline >> 24 != 0 {
                for (i, [from, to]) in [
                    [[x, y, z], [X, y, z]],
                    [[X, y, z], [X, y, Z]],
                    [[X, y, Z], [x, y, Z]],
                    [[x, y, Z], [x, y, z]],
                    [[x, y, z], [x, Y, z]],
                    [[X, y, z], [X, Y, z]],
                    [[X, y, Z], [X, Y, Z]],
                    [[x, y, Z], [x, Y, Z]],
                    [[x, Y, z], [X, Y, z]],
                    [[X, Y, z], [X, Y, Z]],
                    [[X, Y, Z], [x, Y, Z]],
                    [[x, Y, Z], [x, Y, z]],
                ]
                .into_iter()
                .enumerate()
                {
                    if edges & (3 << (i * 2)) != 0 {
                        out.line(from, to, outline, 1.).through_walls();
                    }
                }
            }
        }
    }
}
