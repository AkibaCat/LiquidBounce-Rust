// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::potion_spoof::Host as PotionHost;
struct Adapter<'a>(&'a mut dyn Host);
impl PotionHost for Adapter<'_> {
    fn duration(&mut self, id: &str) -> Option<i32> {
        self.0.client_potion_spoof_duration(id)
    }
    fn add_effect(&mut self, id: &str, amp: i32) {
        self.0.client_potion_spoof_apply(id, amp, 0);
    }
    fn add_attributes(&mut self, id: &str, amp: i32) {
        self.0.client_potion_spoof_apply(id, amp, 1);
    }
    fn remove_effect(&mut self, id: &str) {
        self.0.client_potion_spoof_apply(id, 0, 2);
    }
    fn remove_attributes(&mut self, id: &str) {
        self.0.client_potion_spoof_apply(id, 0, 3);
    }
}
pub(crate) fn disabled(
    host: &mut dyn Host,
    module: &liquidbounce_behaviors::potion_spoof::PotionSpoof,
) {
    module.on_disabled(&mut Adapter(host));
}
impl LiquidBounceHooks {
    pub(crate) fn potion_spoof_tick(&self, host: &mut dyn Host) {
        let (module, running) = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            (
                modules.potion_spoof.clone(),
                modules.query_running(&state.registry, "PotionSpoof"),
            )
        };
        module.tick(running, &mut Adapter(host));
    }
}
