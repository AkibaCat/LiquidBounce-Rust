// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{
    chunk::Section,
    world_primitives::{PrimitiveShape, WorldPrimitives},
};
use horizon_world::registry::BlockStateId;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Weak},
};
#[derive(Default)]
pub(super) struct Cache {
    blocks: Vec<String>,
    // Weak snapshots observe Arc::make_mut writes without retaining chunk data.
    sections: BTreeMap<(i32, i32, i32), (Weak<Section>, Vec<[i32; 3]>)>,
}
impl Game<'_> {
    pub(super) fn client_protection_zones_geometry(&self, out: &mut WorldPrimitives) {
        let mut options = String::new();
        self.native_client_protection_zones("{\"action\":\"options\"}", &mut options);
        let mut cache = self.client_protection_zones_cache.borrow_mut();
        let Ok(options) = serde_json::from_str::<Value>(&options) else {
            *cache = Cache::default();
            return;
        };
        let Ok(blocks) = serde_json::from_value::<Vec<String>>(options["blocks"].clone()) else {
            *cache = Cache::default();
            return;
        };
        if cache.blocks != blocks {
            cache.sections.clear();
            cache.blocks = blocks;
        }
        let Some(player) = self.player.as_ref() else {
            cache.sections.clear();
            return;
        };
        let selected = cache
            .blocks
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let matches = |id: u32| {
            let id = BlockStateId(id);
            !self.res.facts().is_air(id)
                && selected.contains(self.res.facts().get(id).name.as_ref())
        };
        let mut updates = Vec::new();
        let mut present = BTreeSet::new();
        for (&(cx, cz), chunk) in self.world.chunks.iter() {
            for (sy, section) in chunk.sections.iter().enumerate() {
                let key = (cx, chunk.min_y() + sy as i32 * 16, cz);
                present.insert(key);
                let weak = Arc::downgrade(section);
                if cache
                    .sections
                    .get(&key)
                    .is_some_and(|(previous, _)| Weak::ptr_eq(previous, &weak))
                {
                    continue;
                }
                let mut positions = Vec::new();
                if section.blocks.maybe_has(matches) {
                    for i in 0..4096 {
                        if matches(section.get(i)) {
                            positions.push([
                                cx.wrapping_mul(16).wrapping_add((i & 15) as i32),
                                chunk.min_y() + (sy as i32) * 16 + (i >> 8) as i32,
                                cz.wrapping_mul(16).wrapping_add((i >> 4 & 15) as i32),
                            ]);
                        }
                    }
                }
                updates.push((key, (weak, positions)));
            }
        }
        drop(selected);
        cache.sections.retain(|key, _| present.contains(key));
        cache.sections.extend(updates);
        let centers = cache
            .sections
            .values()
            .flat_map(|(_, p)| p.iter().copied())
            .collect::<Vec<_>>();
        let holding = [self.stack_in(self.selected), self.offhand.as_ref()]
            .into_iter()
            .flatten()
            .any(|s| {
                self.items()
                    .get(&s.item)
                    .and_then(|info| info.block)
                    .is_some_and(|state| {
                        cache
                            .blocks
                            .iter()
                            .any(|name| name == self.res.facts().get(state).name.as_ref())
                    })
            });
        drop(cache);
        let layout = self.world.layout();
        let mut geometry = String::new();
        self.native_client_protection_zones(&json!({"action":"render","centers":centers,"player":[player.position.x,player.position.y,player.position.z],"minY":layout.min_y(),"maxY":layout.max_y()-1,"holding":holding}).to_string(),&mut geometry);
        let Ok(Value::Array(draws)) = serde_json::from_str(&geometry) else {
            return;
        };
        for draw in draws {
            let Ok(b) = serde_json::from_value::<[f64; 6]>(draw["bounds"].clone()) else {
                continue;
            };
            let fill = draw["fill"].as_u64().unwrap_or(0) as u32;
            let outline = draw["outline"].as_u64().unwrap_or(0) as u32;
            let corners: [[f64; 3]; 8] = std::array::from_fn(|i| {
                [
                    b[if i & 1 == 0 { 0 } else { 3 }],
                    b[if i & 2 == 0 { 1 } else { 4 }],
                    b[if i & 4 == 0 { 2 } else { 5 }],
                ]
            });
            if fill >> 24 != 0 {
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
            }
            if outline >> 24 != 0 {
                for i in 0..8 {
                    for axis in [1, 2, 4] {
                        if i & axis == 0 {
                            out.line(corners[i], corners[i | axis], outline, 1.)
                                .through_walls();
                        }
                    }
                }
            }
        }
    }
}
