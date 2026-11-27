// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete SkinDerp coroutine and real options effects, with lock-free Host calls.
use crate::{behaviors::BehaviorModules, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;

fn register_waits(modules: &mut BehaviorModules) {
    for callback in modules.skin_derp.take_wait_registrations() {
        modules
            .coroutine_ticker
            .register(crate::coroutine_ticker::WaitOwner::SkinDerp, callback);
    }
}
impl LiquidBounceHooks {
    fn skin_derp_apply(&self, host: &mut dyn Host, before: u8, next: Option<u8>) {
        if let Some(parts) = next {
            let accepted = host.client_skin_derp_set_parts(i32::from(parts));
            #[cfg(feature = "webview")]
            self.state().probe(
                "clientSkinDerpSetParts",
                serde_json::json!({"before":before,"parts":parts}),
                serde_json::json!(accepted),
            );
            #[cfg(not(feature = "webview"))]
            let _ = (before, accepted);
        }
    }
    pub(crate) fn skin_derp_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        callback: u64,
    ) -> bool {
        let actual = host.client_skin_derp_parts() as u8;
        let (keep, next) = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = in_game && modules.query_running(&state.registry, "SkinDerp");
            let result = liquidbounce_behaviors::random::with_thread_random(|rng| {
                modules
                    .skin_derp
                    .advance_callback(callback, running, actual, &mut || rng.next_bool())
            });
            register_waits(&mut modules);
            result
        };
        self.skin_derp_apply(host, actual, next);
        keep
    }
    pub(crate) fn skin_derp_tick(&self, host: &mut dyn Host) {
        let actual = host.client_skin_derp_parts() as u8;
        let next = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "SkinDerp");
            let result = liquidbounce_behaviors::random::with_thread_random(|rng| {
                modules
                    .skin_derp
                    .start_tick_handler(running, actual, &mut || rng.next_bool())
            });
            register_waits(&mut modules);
            result
        };
        self.skin_derp_apply(host, actual, next);
    }
}
