// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientBlockEspEvent;
use liquidbounce_behaviors::block_esp::Block;
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn block_esp(&self, event: &mut ClientBlockEspEvent<'_>) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let modules = state.behaviors.borrow();
        if !modules.query_running(&state.registry, "BlockESP") {
            return;
        }
        let settings = &modules.block_esp;
        match input["action"].as_str() {
            Some("policy") => {
                *event.geometry = json!({"running":true,"targets":settings.targets}).to_string()
            }
            Some("frame") => {
                let camera = serde_json::from_value(input["camera"].clone()).unwrap_or([0.; 3]);
                let blocks = input["blocks"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Block::from_json)
                    .collect::<Vec<_>>();
                static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
                let nanos = START
                    .get_or_init(std::time::Instant::now)
                    .elapsed()
                    .as_nanos() as f64;
                *event.geometry = settings.frame(&blocks, camera, nanos).to_string();
            }
            _ => {}
        }
    }
}
