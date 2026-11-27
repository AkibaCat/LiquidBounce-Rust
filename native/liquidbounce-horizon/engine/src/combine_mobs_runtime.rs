// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientCombineMobsEvent, ClientCombineMobsFrameEvent, Host};
use liquidbounce_behaviors::combine_mobs::Key;
impl LiquidBounceHooks {
    pub(crate) fn combine_mobs_frame(
        &self,
        _host: &dyn Host,
        _event: &mut ClientCombineMobsFrameEvent,
    ) {
        self.state()
            .behaviors
            .borrow_mut()
            .combine_mobs
            .clear_render();
    }
    pub(crate) fn combine_mobs(&self, _host: &dyn Host, event: &mut ClientCombineMobsEvent) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let running = modules.query_running(&state.registry, "CombineMobs");
        let key = Key::at(event.kind, event.baby, [event.x, event.y, event.z]);
        if event.action == 2 {
            event.count = modules.combine_mobs.count(running, &key);
        } else if event.action == 3 {
            modules.combine_mobs.clear_nametags();
        } else {
            event.skip = modules
                .combine_mobs
                .track(running, key, event.group, event.action == 1);
        }
    }
    pub(crate) fn combine_mobs_disabled(&self) {
        self.state().behaviors.borrow_mut().combine_mobs.disabled();
    }
}
