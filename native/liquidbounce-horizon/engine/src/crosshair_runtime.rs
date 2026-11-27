// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientCrosshairEvent, Host};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn crosshair(&self, _host: &dyn Host, event: &mut ClientCrosshairEvent) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let enabled = state
            .registry
            .module(modules.query_ids["Crosshair"])
            .is_some_and(|m| m.enabled());
        let running = modules.query_running(&state.registry, "Crosshair");
        if v["phase"] == "visibility" {
            *event.geometry = json!({"cancelVanilla":enabled}).to_string();
            return;
        }
        if v["phase"] == "reset" {
            modules.crosshair.reset();
            return;
        }
        if v["phase"] == "tick" {
            modules.crosshair.tick(
                v["now"].as_i64().unwrap_or(0),
                v["target"].as_str().map(str::to_owned),
                v["hurt"].as_i64().unwrap_or(0) as i32,
                running,
            );
            return;
        }
        if running {
            *event.geometry = modules.crosshair.render(&v).to_string();
        } else {
            modules.crosshair.reset();
        }
    }
}
