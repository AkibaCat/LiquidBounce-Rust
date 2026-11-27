// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientCustomAmbienceEvent, Host};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn custom_ambience_tick(&self) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if modules.query_running(&state.registry, "CustomAmbience") {
            modules.custom_ambience.game_tick();
        }
    }
    pub(crate) fn custom_ambience(&self, _: &dyn Host, e: &mut ClientCustomAmbienceEvent) {
        let Ok(input) = serde_json::from_str::<Value>(e.input) else {
            return;
        };
        let state = self.state();
        if input["action"] == "frame" {
            let mut modules = state.behaviors.borrow_mut();
            if modules.query_running(&state.registry, "CustomAmbience") {
                *e.output = modules.custom_ambience.frame(&input).to_string();
            }
        } else {
            let modules = state.behaviors.borrow();
            if !modules.query_running(&state.registry, "CustomAmbience") {
                return;
            }
            let s = &modules.custom_ambience;
            *e.output=match input["action"].as_str().unwrap_or(""){
                "weather"=>json!(s.weather(serde_json::from_value(input["original"].clone()).unwrap_or([0.;2]))),
                "time"=>json!(s.time(input["original"].as_i64().unwrap_or(0),input["overworld"].as_bool().unwrap_or(false))),
                "precipitation"=>json!({"intensity":s.precipitation(input["intensity"].as_f64().unwrap_or(0.)as f32),"snow":s.text("Weather")=="Snowy"}),
                _=>s.settings()
            }.to_string();
        }
    }
}
