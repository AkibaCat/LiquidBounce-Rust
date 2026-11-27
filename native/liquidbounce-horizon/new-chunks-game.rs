// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::world_primitives::{PrimitiveShape, WorldPrimitives};
use horizon_protocol::packets::play::Clientbound as C;
use horizon_world::{
    fluid::{FluidKind, FluidState},
    registry::BlockStateId,
};
use serde_json::{Value, json};
impl Game<'_> {
    fn client_new_chunks_flowing(&self, id: i32) -> bool {
        self.pack
            .assets
            .registry
            .state(BlockStateId(id as u32))
            .and_then(|b| FluidState::from_block(b).ok())
            .is_some_and(|f| f.kind() != FluidKind::Empty && !f.is_source())
    }
    pub(super) fn client_new_chunks_packet(&self, packet: &C) {
        if !self.native_window() {
            return;
        }
        let input = match packet {
            C::LevelChunkWithLight(p) => json!({"action":"chunk","pos":[p.x,p.z]}),
            C::ForgetLevelChunk(p) => json!({"action":"forget","pos":p.pos}),
            C::BlockUpdate(p) if self.client_new_chunks_flowing(p.state) => {
                json!({"action":"flow","positions":[p.pos]})
            }
            C::SectionBlocksUpdate(p) => {
                let [sx, sy, sz] = p.section;
                let positions = p
                    .changes
                    .iter()
                    .filter_map(|&change| {
                        if !self.client_new_chunks_flowing((change >> 12) as i32) {
                            return None;
                        }
                        let local = (change & 0xfff) as i32;
                        Some([
                            sx.wrapping_mul(16).wrapping_add(local >> 8 & 15),
                            sy.wrapping_mul(16).wrapping_add(local & 15),
                            sz.wrapping_mul(16).wrapping_add(local >> 4 & 15),
                        ])
                    })
                    .collect::<Vec<_>>();
                if positions.is_empty() {
                    return;
                }
                json!({"action":"flow","positions":positions})
            }
            _ => return,
        };
        self.native_client_new_chunks(&input.to_string(), &mut String::new());
    }
    pub(super) fn client_new_chunks_geometry(&self, out: &mut WorldPrimitives) {
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let input = json!({"action":"render","player":[p.position.x,p.position.y,p.position.z]})
            .to_string();
        let mut geometry = String::new();
        self.native_client_new_chunks(&input, &mut geometry);
        let Ok(Value::Array(planes)) = serde_json::from_str(&geometry) else {
            return;
        };
        for p in planes {
            let Ok(corners) = serde_json::from_value::<[[f64; 3]; 4]>(p["corners"].clone()) else {
                continue;
            };
            let color = p["color"].as_u64().unwrap_or(0) as u32;
            let outline = p["outline"].as_u64().unwrap_or(0) as u32;
            if color >> 24 != 0 {
                out.push(PrimitiveShape::LiquidBounceQuad {
                    corners,
                    colors: [color; 4],
                })
                .through_walls();
            }
            if outline >> 24 != 0 {
                for i in 0..4 {
                    out.line(corners[i], corners[(i + 1) % 4], outline, 1.)
                        .through_walls();
                }
            }
        }
    }
}
