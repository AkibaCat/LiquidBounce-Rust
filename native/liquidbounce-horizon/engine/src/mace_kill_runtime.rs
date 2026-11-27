// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::mace_kill::Host as MaceHost;

struct Adapter<'a>(&'a mut dyn Host);
impl MaceHost for Adapter<'_> {
    fn holding_mace(&mut self) -> bool {
        self.0
            .client_mace_kill_context()
            .is_some_and(|p| p[4] != 0.0)
    }
    fn determine_height(&mut self, maximum: i32) -> i32 {
        self.0.client_mace_kill_height(maximum)
    }
    fn on_ground(&mut self) -> bool {
        self.0
            .client_mace_kill_context()
            .is_some_and(|p| p[3] != 0.0)
    }
    fn position(&mut self) -> [f64; 3] {
        self.0
            .client_mace_kill_context()
            .map_or([0.0; 3], |p| [p[0], p[1], p[2]])
    }
    fn warp(&mut self, position: Option<[f64; 3]>, ground: bool) {
        self.0.client_mace_kill_warp(position, ground);
    }
}
impl LiquidBounceHooks {
    pub(crate) fn mace_kill_attack(&self, host: &mut dyn Host) {
        let (running, settings) = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            (
                modules.query_running(&state.registry, "MaceKill"),
                modules.mace_kill,
            )
        };
        settings.attack(running, &mut Adapter(host));
    }
}
