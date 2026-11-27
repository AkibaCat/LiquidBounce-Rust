// SPDX-License-Identifier: GPL-3.0-or-later
use super::{
    Game,
    hole_tracker::{self, AIR, HoleManager, Region},
};
use horizon_client::chunk::Section;
use horizon_world::registry::BlockStateId;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Weak},
};
#[derive(Default)]
pub(super) struct Cache {
    pub manager: HoleManager,
    sections: BTreeMap<(i32, i32, i32), (Weak<Section>, Vec<i8>)>,
    chunks: BTreeSet<(i32, i32)>,
    layout: Option<(i32, i32)>,
}
impl Game<'_> {
    pub(super) fn client_hole_esp_reset(&self) {
        *self.client_hole_esp_cache.borrow_mut() = Cache::default();
    }
    fn client_hole_esp_options(&self) -> Option<(i32, i32)> {
        let mut result = String::new();
        self.native_client_hole_esp("{\"action\":\"options\"}", &mut result);
        let v: Value = serde_json::from_str(&result).ok()?;
        if !v["active"].as_bool().unwrap_or(false) {
            return None;
        }
        Some((
            v["horizontal"].as_i64()? as i32,
            v["vertical"].as_i64()? as i32,
        ))
    }
    /// Real LocalPlayer post-tick: manager movement and ChunkScanner-equivalent diffs.
    pub(super) fn client_hole_esp_tick(&self) {
        let (Some((h, v)), Some(player)) = (self.client_hole_esp_options(), self.player.as_ref())
        else {
            self.client_hole_esp_reset();
            return;
        };
        let layout = self.world.layout();
        let (min_y, max_y) = (layout.min_y(), layout.max_y());
        let mut cache = self.client_hole_esp_cache.borrow_mut();
        if cache.layout != Some((min_y, max_y)) {
            *cache = Cache::default();
            cache.layout = Some((min_y, max_y));
        }
        let position =
            [player.position.x, player.position.y, player.position.z].map(|n| n.floor() as i32);
        let state_class = |id: BlockStateId| {
            hole_tracker::classify(
                self.res.facts().is_air(id),
                self.res.facts().get(id).explosion_resistance,
            )
        };
        let mut read = |p: [i32; 3]| state_class(self.world.block(p[0], p[1], p[2]));
        if !cache.manager.active() {
            cache.manager.subscribe(0, h, v, position, max_y, &mut read);
        } else {
            cache.manager.settings(0, h, v);
            cache.manager.player_post_tick(position, max_y, &mut read);
        }
        let region = cache.manager.region;
        let mut present = BTreeSet::new();
        let mut sections = BTreeMap::new();
        let mut updates = vec![];
        let mut changed = vec![];
        let mut classes = BTreeMap::new();
        for (&(cx, cz), chunk) in self.world.chunks.iter() {
            let bounds = Region::chunk(cx, cz, min_y, max_y);
            if !bounds.intersects(region) {
                continue;
            }
            present.insert((cx, cz));
            let new_chunk = !cache.chunks.contains(&(cx, cz));
            if new_chunk {
                updates.push(bounds);
            }
            for (sy, section) in chunk.sections.iter().enumerate() {
                let key = (cx, chunk.min_y() + sy as i32 * 16, cz);
                let weak = Arc::downgrade(section);
                if cache
                    .sections
                    .get(&key)
                    .is_some_and(|(previous, _)| Weak::ptr_eq(previous, &weak))
                {
                    continue;
                }
                let values = (0..4096)
                    .map(|i| {
                        let id = section.get(i);
                        *classes
                            .entry(id)
                            .or_insert_with(|| state_class(BlockStateId(id)))
                    })
                    .collect::<Vec<_>>();
                if !new_chunk {
                    for (i, &class) in values.iter().enumerate() {
                        let previous = cache
                            .sections
                            .get(&key)
                            .and_then(|(_, states)| states.get(i))
                            .copied()
                            .unwrap_or(AIR);
                        if previous != class {
                            changed.push((
                                [
                                    cx * 16 + (i & 15) as i32,
                                    key.1 + (i >> 8) as i32,
                                    cz * 16 + (i >> 4 & 15) as i32,
                                ],
                                class,
                            ));
                        }
                    }
                }
                sections.insert(key, (weak, values));
            }
        }
        // ChunkScanner.clearChunk invalidates the entire unloaded footprint.
        let unloaded = cache
            .chunks
            .difference(&present)
            .copied()
            .collect::<Vec<_>>();
        for (cx, cz) in unloaded {
            if !self.world.chunks.contains_key(&(cx, cz)) {
                cache
                    .manager
                    .tracker
                    .invalidate(Region::chunk(cx, cz, min_y, max_y));
            }
        }
        cache
            .sections
            .retain(|(x, _, z), _| present.contains(&(*x, *z)));
        cache.sections.extend(sections);
        cache.chunks = present;
        // New chunk updates scan its complete height as the original tracker does.
        for bounds in updates {
            cache
                .manager
                .tracker
                .chunk_update(bounds, region, max_y, &mut read);
        }
        // COW section identity changes observe both integrated and remote world edits.
        for (p, class) in changed {
            cache
                .manager
                .tracker
                .record_block(p, class, max_y, &mut read);
        }
    }
    pub(super) fn client_hole_esp_frame(&self) {
        if self.client_hole_esp_options().is_none() {
            self.client_hole_esp_reset();
            return;
        }
        let Some(player) = self.player.as_ref() else {
            return;
        };
        let cache = self.client_hole_esp_cache.borrow();
        let holes = cache
            .manager
            .tracker
            .holes
            .values()
            .map(|h| json!({"position":h.position,"size":h.size,"kind":h.kind}))
            .collect::<Vec<_>>();
        let input = json!({"action":"render","player":[player.position.x,player.position.y,player.position.z],"holes":holes});
        drop(cache);
        let mut geometry = String::new();
        self.native_client_hole_esp(&input.to_string(), &mut geometry);
        horizon_client::gfx::hole_esp::begin_frame(&geometry);
    }
}
