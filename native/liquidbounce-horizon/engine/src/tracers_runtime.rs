// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientTracersEvent, Host};
use liquidbounce_behaviors::{
    combine_mobs::{CombineMobs, Key},
    targeting::VisualCamera,
    tracers::Entity,
};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn tracers(&self, _host: &dyn Host, event: &mut ClientTracersEvent) {
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
            .query_running(&state.registry, "Tracers")
        {
            return;
        }
        let perspective = input["perspective"].as_i64().unwrap_or(0) as i32;
        let tick = input["tick"].as_bool() == Some(true);
        let refresh = {
            let s = &state.behaviors.borrow().tracers;
            tick || s.perspective.is_some_and(|p| p != perspective)
        };
        if refresh {
            let free_look = state
                .registry
                .module(state.behaviors.borrow().query_ids["FreeLook"])
                .is_some_and(|m| m.enabled());
            let mut combine = {
                let m = state.behaviors.borrow();
                let mut c = CombineMobs::default();
                c.combine_armor_stands = m.combine_mobs.combine_armor_stands;
                c.combine_minecarts = m.combine_mobs.combine_minecarts;
                c
            };
            let combine_running = state
                .behaviors
                .borrow()
                .query_running(&state.registry, "CombineMobs");
            let mut selected = std::collections::BTreeSet::new();
            for (e, snapshot) in entries.iter().zip(&snapshots) {
                let Some(snapshot) = snapshot else {
                    continue;
                };
                let Some(id) = e["id"].as_i64().map(|v| v as i32) else {
                    continue;
                };
                let info = state.target_info(id, snapshot);
                if !state.global_targets.visual.should_show(
                    &snapshot.entity,
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
            let mut m = state.behaviors.borrow_mut();
            m.tracers.selected = selected;
            m.tracers.perspective = Some(perspective);
            m.combine_mobs.clear_nametags();
        }
        if tick {
            return;
        }
        let mut entities = vec![];
        for (e, snapshot) in entries.iter().zip(&snapshots) {
            let Some(identity) = e["identity"].as_u64() else {
                continue;
            };
            if !state
                .behaviors
                .borrow()
                .tracers
                .selected
                .contains(&identity)
            {
                continue;
            }
            let Some(snapshot) = snapshot else {
                continue;
            };
            let id = e["id"].as_i64().unwrap_or(0) as i32;
            let tag = state.target_info(id, snapshot).resolved_color();
            let friend = snapshot.entity.kind.category
                == Some(liquidbounce_behaviors::targeting::Target::Players)
                && snapshot
                    .player_name
                    .as_deref()
                    .is_some_and(|n| state.friends.contains(n));
            let (Ok(position), Ok(interpolated)) = (
                serde_json::from_value(e["position"].clone()),
                serde_json::from_value(e["interpolated"].clone()),
            ) else {
                continue;
            };
            entities.push(Entity {
                position,
                interpolated,
                height: e["height"].as_f64().unwrap_or(0.) as f32,
                health: e["health"].as_f64().unwrap_or(0.) as f32,
                max_health: e["maxHealth"].as_f64().unwrap_or(20.) as f32,
                friend,
                tag,
            });
        }
        let camera = serde_json::from_value(input["camera"].clone()).unwrap_or([0.; 3]);
        let eye = serde_json::from_value(input["eye"].clone()).unwrap_or([0., 0., 1.]);
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let nanos = START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_nanos() as f64;
        let m = state.behaviors.borrow();
        let s = &m.tracers;
        let lines = s
            .lines(camera, eye, &entities, nanos)
            .into_iter()
            .map(|line| line.map(|p| json!({"position":p.position,"color":p.color})))
            .collect::<Vec<_>>();
        let effect=s.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        *event.geometry =
            json!({"quads":[],"lines":lines,"effect":effect,"line_width":s.line_width}).to_string();
    }
}
