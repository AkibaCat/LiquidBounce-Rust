// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientTntTimerEvent, Host};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn tnt_timer(&self, _: &dyn Host, e: &mut ClientTntTimerEvent) {
        let Ok(v) = serde_json::from_str::<Value>(e.input) else {
            return;
        };
        let state = self.state();
        let b = state.behaviors.borrow();
        if !b.query_running(&state.registry, "TNTTimer") {
            return;
        }
        *e.output = if v["action"] == "glow" {
            if !b.tnt_timer.boolean("ESP") {
                return;
            }
            json!(liquidbounce_behaviors::tnt_timer::TNTTimer::glow(
                v["fuse"].as_i64().unwrap_or(0) as i32
            ))
            .to_string()
        } else {
            b.tnt_timer.frame(&v).to_string()
        };
    }
}
