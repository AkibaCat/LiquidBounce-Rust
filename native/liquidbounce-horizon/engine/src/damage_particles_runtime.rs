// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientDamageParticlesEvent, Host};
use liquidbounce_behaviors::damage_particles::Entity;
use serde_json::{Value, json};
fn entity(e: &Value) -> Option<Entity> {
    Some(Entity {
        identity: e["identity"].as_u64().unwrap_or(0),
        own: e["own"].as_bool()?,
        ticked: e["ticked"].as_bool()?,
        dead: e["dead"].as_bool().unwrap_or(false),
        health: e["health"].as_f64().unwrap_or(0.) as f32,
        max_health: e["max"].as_f64().unwrap_or(0.) as f32,
        center: serde_json::from_value(e["center"].clone()).ok()?,
        movement: serde_json::from_value(e["movement"].clone()).ok()?,
    })
}
impl LiquidBounceHooks {
    pub(crate) fn damage_particles(
        &self,
        _host: &dyn Host,
        event: &mut ClientDamageParticlesEvent,
    ) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if !modules.query_running(&state.registry, "DamageParticles") {
            return;
        }
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let s = &mut modules.damage_particles;
        for u in input["updates"].as_array().into_iter().flatten() {
            if let Some(e) = entity(u) {
                s.update(
                    u["now"].as_i64().unwrap_or(0),
                    e,
                    u["old"].as_f64().unwrap_or(0.) as f32,
                    u["new"].as_f64().unwrap_or(0.) as f32,
                    u["max"].as_f64().unwrap_or(0.) as f32,
                );
            }
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |v| v.as_millis() as i64);
        if input["tick"].as_bool() == Some(true) {
            let entities = input["entities"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(entity)
                .collect::<Vec<_>>();
            s.tick(now, &entities);
        } else {
            *event.draws=json!(s.render(now).into_iter().map(|p|json!({"text":p.text,"color":p.color,"position":p.position,"scale":p.scale})).collect::<Vec<_>>()).to_string();
        }
    }
}
