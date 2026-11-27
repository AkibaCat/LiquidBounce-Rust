// SPDX-License-Identifier: GPL-3.0-or-later
//! GameTick and onDisabled effects execute after dropping actor state locks.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
impl LiquidBounceHooks {
    pub(crate) fn fast_break_tick(&self, host: &mut dyn Host) {
        let (running, settings) = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            let running = modules.query_running(&state.registry, "FastBreak");
            modules.fast_break.set_running(running);
            let settings = *modules
                .fast_break
                .behavior
                .lock()
                .expect("FastBreak poisoned");
            (running, settings)
        };
        host.client_fast_break_update(running, settings.break_damage, settings.only_tool);
    }
}
