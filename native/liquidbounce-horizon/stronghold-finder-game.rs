// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{
    chunk::Section,
    hud::{HudBatch, Rect},
    world_primitives::{PrimitiveShape, WorldPrimitives},
};
use horizon_protocol::packets::play::Clientbound as C;
use horizon_world::registry::BlockStateId;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Weak},
};
#[derive(Default)]
pub(super) struct Cache {
    sections: BTreeMap<(i32, i32, i32), (Weak<Section>, Vec<([i32; 3], String)>)>,
    events: Vec<Value>,
    hud: Vec<String>,
}
impl Game<'_> {
    fn client_stronghold_eligible(&self) -> bool {
        self.client_smart_eat_player() && self.dim == super::Dim::Overworld
    }
    fn client_stronghold_enabled(&self) -> bool {
        if !self.client_stronghold_eligible() {
            return false;
        }
        let mut out = String::new();
        self.native_client_stronghold_finder("{\"action\":\"options\"}", &mut out);
        out == "true"
    }
    // Item callbacks may run inside another module's host action. Queue facts
    // locally and dispatch on tick, after that actor lock has been released.
    pub(super) fn client_stronghold_used(&self, eye: bool, consumes: bool) {
        if !eye || !consumes || !self.client_stronghold_eligible() {
            return;
        }
        if let Some(p) = self.player.as_ref() {
            self.client_stronghold_cache.borrow_mut().events.push(json!({"action":"used","position":[p.position.x,p.position.y,p.position.z],"tick":p.health.tick_count}));
        }
    }
    pub(super) fn client_stronghold_packet(&self, packet: &C) {
        if !self.client_stronghold_eligible() {
            return;
        }
        let C::AddEntity(p) = packet else {
            return;
        };
        let eye = self
            .remote
            .as_ref()
            .and_then(|r| r.builtin.get("minecraft:entity_type"))
            .and_then(|r| r.name(p.kind))
            == Some("minecraft:eye_of_ender");
        if !eye {
            return;
        }
        let tick = self.player.as_ref().map_or(0, |p| p.health.tick_count);
        self.client_stronghold_cache
            .borrow_mut()
            .events
            .push(json!({"action":"spawn","id":p.id,"position":p.position,"tick":tick}));
    }
    pub(super) fn client_stronghold_local_spawn(&self, id: u32, position: [f64; 3]) {
        if self.remote.is_none() && self.client_stronghold_eligible() {
            let tick = self.player.as_ref().map_or(0, |p| p.health.tick_count);
            self.client_stronghold_cache
                .borrow_mut()
                .events
                .push(json!({"action":"spawn","id":id,"position":position,"tick":tick}));
        }
    }
    fn client_stronghold_emit(&self, input: Value) {
        self.native_client_stronghold_finder(&input.to_string(), &mut String::new());
    }
    pub(super) fn client_stronghold_tick(&self) {
        let events = std::mem::take(&mut self.client_stronghold_cache.borrow_mut().events);
        if !self.client_stronghold_enabled() {
            self.client_stronghold_cache.borrow_mut().sections.clear();
            return;
        }
        // The integrated server can create the eye before Item.use RETURN.
        // Consume recorded throws before queued spawn delivery in this turn.
        for e in events
            .iter()
            .filter(|e| e["action"] == "used")
            .chain(events.iter().filter(|e| e["action"] != "used"))
        {
            self.client_stronghold_emit(e.clone());
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let eyes = self
            .end
            .eyes_tracked()
            .map(
                |(id, position, velocity)| json!({"id":id,"position":position,"velocity":velocity}),
            )
            .collect::<Vec<_>>();
        self.client_stronghold_emit(
            json!({"action":"tick","tick":p.health.tick_count,"eyes":eyes}),
        );
    }
    fn client_stronghold_portals(&self) {
        let matches = |id: u32| {
            matches!(
                self.res.facts().get(BlockStateId(id)).name.as_ref(),
                "minecraft:end_portal" | "minecraft:end_portal_frame"
            )
        };
        let mut cache = self.client_stronghold_cache.borrow_mut();
        let mut present = BTreeSet::new();
        let mut changed = Vec::new();
        for (&(cx, cz), chunk) in self.world.chunks.iter() {
            for (sy, section) in chunk.sections.iter().enumerate() {
                let key = (cx, chunk.min_y() + sy as i32 * 16, cz);
                present.insert(key);
                let weak = Arc::downgrade(section);
                if cache
                    .sections
                    .get(&key)
                    .is_some_and(|(old, _)| Weak::ptr_eq(old, &weak))
                {
                    continue;
                }
                let mut positions = Vec::new();
                if section.blocks.maybe_has(matches) {
                    for i in 0..4096 {
                        if matches(section.get(i)) {
                            let pos = [
                                cx.wrapping_mul(16).wrapping_add((i & 15) as i32),
                                key.1 + (i >> 8) as i32,
                                cz.wrapping_mul(16).wrapping_add((i >> 4 & 15) as i32),
                            ];
                            positions.push((
                                pos,
                                self.res
                                    .facts()
                                    .get(BlockStateId(section.get(i)))
                                    .name
                                    .to_string(),
                            ));
                        }
                    }
                }
                if let Some((_, old)) = cache.sections.insert(key, (weak, positions.clone())) {
                    for (pos, _) in old {
                        changed.push(json!({"position":pos,"name":"minecraft:air"}));
                    }
                }
                for (pos, name) in positions {
                    changed.push(json!({"position":pos,"name":name}));
                }
            }
        }
        let removed = cache
            .sections
            .keys()
            .filter(|k| !present.contains(k))
            .copied()
            .collect::<Vec<_>>();
        for key in removed {
            if let Some((_, old)) = cache.sections.remove(&key) {
                for (pos, _) in old {
                    changed.push(json!({"position":pos,"name":"minecraft:air"}));
                }
            }
        }
        drop(cache);
        if !changed.is_empty() {
            self.client_stronghold_emit(json!({"action":"blocks","blocks":changed}));
        }
    }
    pub(super) fn client_stronghold_geometry(&self, out: &mut WorldPrimitives, partial: f64) {
        self.client_stronghold_cache.borrow_mut().hud.clear();
        if !self.client_stronghold_enabled() {
            return;
        }
        self.client_stronghold_portals();
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let player = [
            self.previous_position.x + (p.position.x - self.previous_position.x) * partial,
            self.previous_position.y + (p.position.y - self.previous_position.y) * partial,
            self.previous_position.z + (p.position.z - self.previous_position.z) * partial,
        ];
        let mut data = String::new();
        self.native_client_stronghold_finder(
            &json!({"action":"render","player":player}).to_string(),
            &mut data,
        );
        let Ok(v) = serde_json::from_str::<Value>(&data) else {
            return;
        };
        if let Ok(hud) = serde_json::from_value::<Vec<String>>(v["hud"].clone()) {
            self.client_stronghold_cache.borrow_mut().hud = hud;
        }
        for line in v["lines"].as_array().into_iter().flatten() {
            if let (Ok(start), Ok(end), Some(color)) = (
                serde_json::from_value(line["start"].clone()),
                serde_json::from_value(line["end"].clone()),
                line["color"].as_u64(),
            ) {
                out.line(start, end, color as u32, 1.).through_walls();
            }
        }
        for plane in v["planes"].as_array().into_iter().flatten() {
            let (Ok([x, y, z]), Some(size), Some(color)) = (
                serde_json::from_value::<[f64; 3]>(plane["position"].clone()),
                plane["size"].as_f64(),
                plane["color"].as_u64(),
            ) else {
                continue;
            };
            let color = color as u32;
            let dark = (color & 0xff000000)
                | ((color >> 16 & 255) * 7 / 10 << 16)
                | ((color >> 8 & 255) * 7 / 10 << 8)
                | ((color & 255) * 7 / 10);
            let corners = [
                [x, y, z],
                [x + size, y, z],
                [x + size, y, z + size],
                [x, y, z + size],
            ];
            out.push(PrimitiveShape::LiquidBounceQuad {
                corners,
                colors: [color; 4],
            })
            .through_walls();
            for i in 0..4 {
                out.line(corners[i], corners[(i + 1) % 4], dark, 1.)
                    .through_walls();
            }
        }
    }
    pub(super) fn client_stronghold_overlay(&self, batch: &mut HudBatch) {
        if !self.client_stronghold_enabled() {
            return;
        }
        let cache = self.client_stronghold_cache.borrow();
        let (ascent, height) = self.gui.atlas.font.liquidbounce_target_metrics();
        let scale = 9. / height;
        for (index, text) in cache.hud.iter().enumerate() {
            let glyphs = text
                .chars()
                .map(|c| self.gui.atlas.font.liquidbounce_target_glyph(c, false))
                .collect::<Vec<_>>();
            let width = glyphs.iter().map(|g| g.advance).sum::<f32>() * scale;
            let left = batch.width * 0.5 - width * 0.5;
            let baseline = batch.height * 0.5 + 10. + index as f32 * 10. + ascent * scale;
            for shadow in [true, false] {
                let mut x = left + if shadow { scale } else { 0. };
                let y = baseline + if shadow { scale } else { 0. };
                let color = if shadow { 0xff3f3f3f } else { 0xffffffff };
                for g in &glyphs {
                    if let Some(Rect { u0, v0, u1, v1, .. }) = g.rect {
                        let x0 = x + g.left * scale;
                        let x1 = x + g.right * scale;
                        let y0 = y + g.top * scale;
                        let y1 = y + g.bottom * scale;
                        batch.quad(
                            [[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
                            [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                            color,
                        );
                    }
                    x += g.advance * scale;
                }
            }
        }
    }
}
