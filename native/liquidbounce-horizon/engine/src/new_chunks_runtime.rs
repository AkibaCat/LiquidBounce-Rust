// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientNewChunksEvent, Host};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn new_chunks(&self, _host: &dyn Host, event: &mut ClientNewChunksEvent) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        if !m.query_running(&state.registry, "NewChunks") {
            return;
        }
        let s = &mut m.new_chunks;
        match v["action"].as_str() {
            Some("chunk") => {
                if let Ok(pos) = serde_json::from_value(v["pos"].clone()) {
                    s.chunk(pos);
                }
            }
            Some("forget") => {
                if let Ok(pos) = serde_json::from_value(v["pos"].clone()) {
                    s.forget(pos);
                }
            }
            Some("flow") => {
                if let Some(positions) = v["positions"].as_array() {
                    for pos in positions {
                        if let Ok(pos) = serde_json::from_value(pos.clone()) {
                            s.flowing(pos);
                        }
                    }
                }
            }
            Some("render") => {
                let Ok(player) = serde_json::from_value(v["player"].clone()) else {
                    return;
                };
                *event.geometry = json!(
                    s.render(player)
                        .into_iter()
                        .map(|p| json!({"corners":p.corners,"color":p.color,"outline":p.outline}))
                        .collect::<Vec<_>>()
                )
                .to_string();
            }
            _ => {}
        }
    }
}
