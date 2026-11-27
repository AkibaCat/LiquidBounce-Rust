// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientRadarEvent;
use liquidbounce_behaviors::{
    combine_mobs::{CombineMobs, Key},
    targeting::{Target, VisualCamera},
};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn radar(&self, event: &mut ClientRadarEvent) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let entries = input["entities"].as_array().cloned().unwrap_or_default();
        let snapshots = entries
            .iter()
            .map(|e| crate::targeting::snapshot(&e["target"].to_string()))
            .collect::<Vec<_>>();
        let mut state = self.state();
        if !state
            .behaviors
            .borrow()
            .query_running(&state.registry, "Radar")
        {
            return;
        }
        let perspective = input["perspective"].as_i64().unwrap_or(0) as i32;
        let tick = input["tick"].as_bool() == Some(true);
        let refresh = {
            let b = state.behaviors.borrow();
            tick || (b.radar.perspective.is_some_and(|p| p != perspective)
                && state.global_targets.visual.contains(Target::SelfEntity))
        };
        if refresh {
            let (free_look, combine_running, mut combine) = {
                let b = state.behaviors.borrow();
                let mut c = CombineMobs::default();
                c.combine_armor_stands = b.combine_mobs.combine_armor_stands;
                c.combine_minecarts = b.combine_mobs.combine_minecarts;
                (
                    state
                        .registry
                        .module(b.query_ids["FreeLook"])
                        .is_some_and(|m| m.enabled()),
                    b.query_running(&state.registry, "CombineMobs"),
                    c,
                )
            };
            let mut selected = std::collections::BTreeSet::new();
            for (e, snapshot) in entries.iter().zip(&snapshots) {
                let Some(s) = snapshot else { continue };
                let Some(id) = e["id"].as_i64().map(|v| v as i32) else {
                    continue;
                };
                let info = state.target_info(id, s);
                if !state.global_targets.visual.should_show(
                    &s.entity,
                    info,
                    VisualCamera {
                        raw_first_person: perspective == 0,
                        free_cam: false,
                        free_look,
                    },
                ) {
                    continue;
                }
                let pos = serde_json::from_value(e["position"].clone()).unwrap_or([0.; 3]);
                let key = Key::at(
                    e["kind"].as_str().unwrap_or(""),
                    e["baby"].as_bool().unwrap_or(false),
                    pos,
                );
                if combine.track(
                    combine_running,
                    key,
                    e["group"].as_u64().unwrap_or(3) as u8,
                    true,
                ) {
                    continue;
                }
                if let Some(identity) = e["identity"].as_u64() {
                    selected.insert(identity);
                }
            }
            let mut b = state.behaviors.borrow_mut();
            b.radar.selected = selected;
            b.radar.perspective = Some(perspective);
            b.combine_mobs.clear_nametags();
        }
        if tick {
            return;
        }
        let camera = serde_json::from_value::<[f64; 3]>(input["camera"].clone()).unwrap_or([0.; 3]);
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let nanos = START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_nanos() as f64;
        let b = state.behaviors.borrow();
        let s = &b.radar;
        let mut out = vec![];
        for (e, snapshot) in entries.iter().zip(&snapshots) {
            let Some(identity) = e["identity"].as_u64() else {
                continue;
            };
            if !s.selected.contains(&identity) {
                continue;
            }
            let Some(target) = snapshot else { continue };
            if target.entity.is_local || (s.only_players && e["kind"] != "minecraft:player") {
                continue;
            }
            // ModuleTeams exposes this predicate independently of its enabled state.
            if s.hide_teammates && target.teams.as_ref().is_some_and(|f| b.teams.teammate(f)) {
                continue;
            }
            let (Ok(position), Ok(current)) = (
                serde_json::from_value::<[f64; 3]>(e["interpolated"].clone()),
                serde_json::from_value::<[f64; 3]>(e["position"].clone()),
            ) else {
                continue;
            };
            let distance = |p: [f64; 3]| {
                (0..3)
                    .map(|i| (p[i] - camera[i]) * (p[i] - camera[i]))
                    .sum::<f64>()
                    .sqrt() as f32
            };
            let d = distance(position);
            let Some(color) = s.indicator_color(
                d,
                distance(current),
                e["health"].as_f64().unwrap_or(0.) as f32,
                e["maxHealth"].as_f64().unwrap_or(20.) as f32,
                nanos,
            ) else {
                continue;
            };
            out.push(json!({"id":e["id"],"identity":identity,"position":position,"priority":-(d as i32),"color":color,"show":s.show,"edgeInset":s.edge_inset,"pointer":s.pointer_mode.tag(),"width":s.width,"height":s.height,"notch":s.tail_concave_size,"size":match s.pointer_mode{liquidbounce_behaviors::radar::PointerMode::Image1=>s.image_sizes[0],_=>s.image_sizes[1]}}));
        }
        *event.geometry = json!(out).to_string();
    }
}
