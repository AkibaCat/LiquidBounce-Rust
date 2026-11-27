// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientBetterTabEvent, Host};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn better_tab(&self, _host: &dyn Host, event: &mut ClientBetterTabEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if !modules.query_running(&state.registry, "BetterTab") {
            return;
        }
        let s = &modules.better_tab;
        let Ok(input) = serde_json::from_str::<Value>(event.players) else {
            return;
        };
        let own = input["own"].as_str().unwrap_or("");
        let mut rows = serde_json::Map::new();
        let mut error = None;
        for p in input["players"].as_array().into_iter().flatten() {
            let Some(id) = p["id"].as_str() else {
                continue;
            };
            let name = p["name"].as_str().unwrap_or("");
            let display = p["display"].as_str();
            match s.hidden(name, display).and_then(|hidden| {
                if hidden {
                    Ok((true, None))
                } else {
                    s.color(name, display, own, state.friends.contains(name))
                        .map(|color| (false, color))
                }
            }) {
                Ok((hidden, color)) => {
                    rows.insert(id.into(), json!({"hidden":hidden,"color":color}));
                }
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }
        if s.tab_size < 0 {
            error = Some("Stream.limit requires nonnegative TabSize");
        }
        if let Some(e) = error {
            eprintln!("[LiquidBounce] BetterTab render handler: {e}");
        }
        *event.policy=json!({"sorting":s.sorting as u8,"visibility":s.visibility,"showGameMode":s.show_game_mode,"limit":s.tab_size,"height":s.column_height,"accurate":s.accurate_latency,"suffix":s.append_ms_suffix,"rows":rows,"error":error}).to_string();
    }
}
