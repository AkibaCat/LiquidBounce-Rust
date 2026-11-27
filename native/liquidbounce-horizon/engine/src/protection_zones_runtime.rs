// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientProtectionZonesEvent, Host};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn protection_zones(
        &self,
        _host: &dyn Host,
        event: &mut ClientProtectionZonesEvent,
    ) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let modules = state.behaviors.borrow();
        if !modules.query_running(&state.registry, "ProtectionZones") {
            return;
        }
        let s = &modules.protection_zones;
        match v["action"].as_str() {
            Some("options") => *event.geometry = json!({"blocks":s.protection_blocks}).to_string(),
            Some("render") => {
                let (Ok(centers), Ok(player)) = (
                    serde_json::from_value::<Vec<[i32; 3]>>(v["centers"].clone()),
                    serde_json::from_value(v["player"].clone()),
                ) else {
                    return;
                };
                let (Some(min), Some(max)) = (v["minY"].as_i64(), v["maxY"].as_i64()) else {
                    return;
                };
                *event.geometry=json!(s.render(&centers,player,min as i32,max as i32,v["holding"].as_bool().unwrap_or(false)).into_iter().map(|d|json!({"bounds":d.bounds,"fill":d.fill as u32,"outline":d.outline as u32})).collect::<Vec<_>>()).to_string();
            }
            _ => {}
        }
    }
}
