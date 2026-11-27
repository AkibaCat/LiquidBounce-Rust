// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientBlockOutlineEvent, Host};
use liquidbounce_behaviors::block_outline::{Bounds, Point, Target};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn block_outline(&self, _host: &dyn Host, event: &mut ClientBlockOutlineEvent) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        event.active = modules.query_running(&state.registry, "BlockOutline");
        if !event.active || event.input.is_empty() {
            return;
        }
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let boxes =
            serde_json::from_value::<Vec<Bounds>>(input["boxes"].clone()).unwrap_or_default();
        let edges =
            serde_json::from_value::<Vec<[Point; 2]>>(input["edges"].clone()).unwrap_or_default();
        let target = if boxes.is_empty() {
            None
        } else {
            Some(Target {
                boxes: &boxes,
                edges: &edges,
                hit: serde_json::from_value(input["hit"].clone()).unwrap_or([0.; 3]),
                side: input["side"].as_u64().unwrap_or(0) as u8,
            })
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |v| v.as_millis() as i64);
        let s = &mut modules.block_outline;
        let mesh = s.render(target, now);
        *event.geometry=json!({"quads":mesh.quads,"lines":mesh.lines,"color":s.color as u32,"outline":s.outline as u32}).to_string();
    }
}
