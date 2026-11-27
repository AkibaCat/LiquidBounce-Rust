// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientHatsEvent, Host};
use liquidbounce_behaviors::{breadcrumbs::rgba, hats::Entity};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn hats(&self, _host: &dyn Host, event: &mut ClientHatsEvent) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let modules = state.behaviors.borrow();
        if !modules.query_running(&state.registry, "Hats") {
            return;
        }
        let free_look = state
            .registry
            .module(modules.query_ids["FreeLook"])
            .is_some_and(|m| m.enabled());
        let entities = input["entities"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|e| {
                Some(Entity {
                    local: e["local"].as_bool() == Some(true),
                    friend: e["name"]
                        .as_str()
                        .is_some_and(|n| state.friends.contains(n)),
                    position: serde_json::from_value(e["position"].clone()).ok()?,
                    interpolated: serde_json::from_value(e["interpolated"].clone()).ok()?,
                    rotation: serde_json::from_value(e["rotation"].clone()).ok()?,
                    height: e["height"].as_f64()? as f32,
                    eye_height: e["eyeHeight"].as_f64()? as f32,
                    armor: e["armor"].as_bool() == Some(true),
                    hurt: e["hurt"].as_bool() == Some(true),
                })
            })
            .collect::<Vec<_>>();
        let Some(local) = entities.iter().find(|e| e.local).map(|e| e.position) else {
            return;
        };
        let epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis().min(u64::MAX as u128) as u64);
        let s = &modules.hats;
        let mesh = s.mesh(
            &entities,
            local,
            input["firstPerson"].as_bool().unwrap_or(true),
            free_look,
            self.started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            epoch,
        );
        let quads = mesh
            .quads
            .into_iter()
            .map(|q| q.map(|p| json!({"position":p.position,"color":p.color})))
            .collect::<Vec<_>>();
        let effect=s.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        let image = if s.mode == 5 {
            Some(
                json!({"path":s.modes[5].image,"quads":mesh.image_quads,"color":rgba(s.modes[5].color as u32)}),
            )
        } else {
            None
        };
        *event.geometry =
            json!({"quads":quads,"lines":[],"effect":effect,"image":image}).to_string();
    }
}
