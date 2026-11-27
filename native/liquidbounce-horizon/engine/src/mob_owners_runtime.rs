// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientMobOwnersEvent, Host};
impl LiquidBounceHooks {
    pub(crate) fn mob_owners(&self, _host: &dyn Host, event: &mut ClientMobOwnersEvent) {
        let state = self.state();
        let m = state.behaviors.borrow();
        event.mask = m
            .mob_owners
            .policy(m.query_running(&state.registry, "MobOwners"));
    }
}
