// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientWingsEvent, Host};
use liquidbounce_behaviors::wings::Entity;
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn wings(&self, _host: &dyn Host, event: &mut ClientWingsEvent) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let modules = state.behaviors.borrow();
        if !modules.query_running(&state.registry, "Wings") {
            return;
        }
        let free = state
            .registry
            .module(modules.query_ids["FreeLook"])
            .is_some_and(|m| m.enabled());
        let entities = v["entities"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|e| {
                Some(Entity {
                    local: e["local"].as_bool() == Some(true),
                    friend: e["name"]
                        .as_str()
                        .is_some_and(|s| state.friends.contains(s)),
                    position: serde_json::from_value(e["position"].clone()).ok()?,
                    interpolated: serde_json::from_value(e["interpolated"].clone()).ok()?,
                    body_yaw: e["bodyYaw"].as_f64()? as f32,
                    scale: e["scale"].as_f64()? as f32,
                    height: e["height"].as_f64()? as f32,
                    chest: e["chest"].as_bool() == Some(true),
                    hurt: e["hurt"].as_bool() == Some(true),
                })
            })
            .collect::<Vec<_>>();
        let Some(local) = entities.iter().find(|e| e.local).map(|e| e.position) else {
            return;
        };
        let s = &modules.wings;
        let quads = s
            .mesh(
                &entities,
                local,
                v["firstPerson"].as_bool().unwrap_or(true),
                free,
                v["shift"].as_bool() == Some(true),
            )
            .into_iter()
            .map(|q| q.map(|p| json!({"position":p.position,"color":p.color})))
            .collect::<Vec<_>>();
        let effect=s.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        *event.geometry = json!({"quads":quads,"lines":[],"effect":effect}).to_string();
    }
}
