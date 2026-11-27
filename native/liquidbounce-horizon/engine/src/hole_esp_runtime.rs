// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientHoleEspEvent, Host};
use liquidbounce_behaviors::hole_tracker::Hole;
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn hole_esp(&self, _host: &dyn Host, event: &mut ClientHoleEspEvent) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let active = modules.query_running(&state.registry, "HoleESP");
        let s = &modules.hole_esp;
        if v["action"] == "options" {
            *event.geometry =
                json!({"active":active,"horizontal":s.horizontal,"vertical":s.vertical})
                    .to_string();
            return;
        }
        if !active {
            return;
        }
        let Ok(player) = serde_json::from_value::<[f64; 3]>(v["player"].clone()) else {
            return;
        };
        let holes = v["holes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| {
                Some(Hole {
                    position: serde_json::from_value(v["position"].clone()).ok()?,
                    size: serde_json::from_value(v["size"].clone()).ok()?,
                    kind: v["kind"].as_u64()? as u8,
                })
            })
            .collect::<Vec<_>>();
        *event.geometry = s.geometry(player, &holes).to_string();
    }
}
