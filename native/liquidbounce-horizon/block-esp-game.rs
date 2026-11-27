// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::chunk::Section;
use horizon_world::registry::BlockStateId;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Weak},
};
#[derive(Default)]
pub(super) struct State {
    targets: BTreeSet<String>,
    sections: BTreeMap<(i32, i32, i32), (Weak<Section>, Vec<[i32; 3]>)>,
}
impl Game<'_> {
    fn client_block_esp_query(&self, input: Value) -> Value {
        let mut output = String::new();
        self.native_client_block_esp(&input.to_string(), &mut output);
        serde_json::from_str(&output).unwrap_or(Value::Null)
    }
    pub(super) fn client_block_esp_frame(&self, camera: glam::DVec3) {
        horizon_client::gfx::block_esp::begin_frame("null");
        if !self.native_window() || self.player.is_none() {
            *self.client_block_esp.borrow_mut() = Default::default();
            return;
        }
        let policy = self.client_block_esp_query(json!({"action":"policy"}));
        if policy["running"] != true {
            *self.client_block_esp.borrow_mut() = Default::default();
            return;
        }
        let targets = serde_json::from_value::<BTreeSet<String>>(policy["targets"].clone())
            .unwrap_or_default();
        let mut cache = self.client_block_esp.borrow_mut();
        if cache.targets != targets {
            cache.targets = targets;
            cache.sections.clear();
        }
        let wanted = |state: BlockStateId| {
            !self.res.facts().is_air(state)
                && cache
                    .targets
                    .contains(self.res.facts().get(state).name.as_ref())
        };
        let mut present = BTreeSet::new();
        let mut changed = Vec::new();
        for (&(cx, cz), chunk) in &self.world.chunks {
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
                if section.blocks.maybe_has(|id| wanted(BlockStateId(id))) {
                    for i in 0..4096 {
                        if wanted(BlockStateId(section.get(i))) {
                            positions.push([
                                cx.wrapping_mul(16).wrapping_add((i & 15) as i32),
                                key.1 + (i >> 8) as i32,
                                cz.wrapping_mul(16).wrapping_add((i >> 4 & 15) as i32),
                            ]);
                        }
                    }
                }
                changed.push((key, weak, positions));
            }
        }
        for (key, weak, positions) in changed {
            cache.sections.insert(key, (weak, positions));
        }
        cache.sections.retain(|key, _| present.contains(key));
        let positions = cache
            .sections
            .values()
            .flat_map(|(_, p)| p.iter().copied())
            .collect::<Vec<_>>();
        drop(cache);
        let blocks=positions.into_iter().filter_map(|position|{let state=self.world.block(position[0],position[1],position[2]);if self.res.facts().is_air(state){return None;}let facts=self.res.facts().get(state);let map=self.native_block_map_color(state).map(|(_,rgb)|rgb).unwrap_or(0);Some(json!({"position":position,"name":facts.name.as_ref(),"mapColor":map,"boxes":self.outline_boxes(position)}))}).collect::<Vec<_>>();
        let frame = self.client_block_esp_query(
            json!({"action":"frame","camera":camera.to_array(),"blocks":blocks}),
        );
        horizon_client::gfx::block_esp::begin_frame(&frame.to_string());
    }
}
