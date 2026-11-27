// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientBreadcrumbsEvent, Host};
use liquidbounce_behaviors::breadcrumbs::{ColoredPoint, Entity};
use serde_json::{Value, json};
fn point(v: ColoredPoint) -> Value {
    json!({"position":v.position,"color":v.color})
}
impl LiquidBounceHooks {
    pub(crate) fn breadcrumbs(&self, _host: &dyn Host, event: &mut ClientBreadcrumbsEvent) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if !modules.query_running(&state.registry, "Breadcrumbs") {
            return;
        }
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let entities = input["entities"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|e| {
                Some(Entity {
                    identity: e["identity"].as_u64()?,
                    own: e["own"].as_bool()?,
                    alive: e["alive"].as_bool()?,
                    position: serde_json::from_value(e["position"].clone()).ok()?,
                    interpolated: serde_json::from_value(e["interpolated"].clone()).ok()?,
                })
            })
            .collect::<Vec<_>>();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |v| v.as_millis() as i64);
        let s = &mut modules.breadcrumbs;
        if input["tick"].as_bool() == Some(true) {
            s.tick(now, &entities);
            return;
        }
        // System.nanoTime has an arbitrary origin; monotonic elapsed time preserves its period and Float conversion.
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let nanos = START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_nanos() as f64;
        let mesh = s.render(now, nanos, &entities);
        let effect=s.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        *event.geometry=json!({"quads":mesh.quads.into_iter().map(|q|q.map(point)).collect::<Vec<_>>(),"lines":mesh.lines.into_iter().map(|q|q.map(point)).collect::<Vec<_>>(),"effect":effect}).to_string();
    }
}
