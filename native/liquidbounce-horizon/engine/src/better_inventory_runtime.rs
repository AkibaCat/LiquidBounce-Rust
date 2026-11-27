// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{better_inventory_draw, runtime::LiquidBounceHooks};
impl LiquidBounceHooks {
    pub(crate) fn better_inventory_render(&self, query: &str, draw: &mut String) {
        let result = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            let running = modules.query_running(&state.registry, "BetterInventory");
            better_inventory_draw::render(
                &modules.better_inventory,
                running,
                modules.inventory.manager.last_clicked_slot,
                query,
            )
        };
        match result {
            Ok(value) => *draw = value.to_string(),
            Err(error) => {
                *draw = "null".into();
                eprintln!("[LiquidBounce] BetterInventory: {error}");
            }
        }
        #[cfg(feature = "webview")]
        if let Ok(input) = serde_json::from_str::<serde_json::Value>(query) {
            let output =
                serde_json::from_str::<serde_json::Value>(draw).unwrap_or(serde_json::Value::Null);
            let kind = input["kind"].as_str().unwrap_or("unknown");
            let name = if kind == "slot" {
                format!("clientBetterInventory:slot:{}", input["index"])
            } else {
                format!("clientBetterInventory:{kind}")
            };
            let mut state = self.state();
            if !output.is_null() {
                state.probe(
                    &format!("clientBetterInventory:{kind}:active"),
                    input.clone(),
                    output.clone(),
                );
            }
            state.probe(&name, input, output);
        }
    }
}
