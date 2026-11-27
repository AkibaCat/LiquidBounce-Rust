// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
impl LiquidBounceHooks {
    pub(crate) fn reverse_step_tick(&self, host: &mut dyn Host) {
        let value = {
            let state = self.state();
            let m = state.behaviors.borrow();
            let s = m.reverse_step;
            serde_json::json!({"running":m.query_running(&state.registry,"ReverseStep"),"mode":s.mode.tag(),"distance":s.maximum_fall_distance,"ticks":s.ticks,"simulate":s.simulate_falling,"motion":s.motion,"factor":s.factor}).to_string()
        };
        host.client_reverse_step_tick(&value);
    }
}
