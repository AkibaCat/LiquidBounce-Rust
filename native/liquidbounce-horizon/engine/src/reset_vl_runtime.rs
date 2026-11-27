// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    coroutine_ticker::WaitOwner,
    runtime::{LiquidBounceHooks, raw_options},
};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::reset_vl::{Effect, TIMER_PRIORITY};

/// Separate from ModuleTimer's reserved u64::MAX provider.
pub(crate) const RESET_VL_TIMER_PROVIDER: u64 = u64::MAX - 1;

#[cfg(feature = "webview")]
fn success_message(language: &str) -> &'static str {
    match language.to_ascii_lowercase().replace('-', "_").as_str() {
        "zh_cn" | "zh_sg" | "zh_hans" => "已成功重置违规计数",
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => "已成功重設違規計數",
        _ => "Successfully reset violations",
    }
}

impl LiquidBounceHooks {
    /// Only the short state-machine operations borrow BehaviorModules. Native
    /// effects and synchronous registry callbacks run with that borrow released.
    fn reset_vl_effects(&self, host: &mut dyn Host, mut effect: Option<Effect>) {
        while let Some(current) = effect {
            let mut on_ground = false;
            match current {
                Effect::RequestTimer(speed) => {
                    let timer = {
                        let state = self.state();
                        let modules = state.behaviors.borrow();
                        let running = modules.query_running(&state.registry, "ResetVL");
                        modules
                            .timer
                            .set_provider_running(RESET_VL_TIMER_PROVIDER, running);
                        modules.timer.clone()
                    };
                    timer
                        .state
                        .lock()
                        .expect("Timer poisoned")
                        .timer
                        .request_speed(speed, TIMER_PRIORITY, RESET_VL_TIMER_PROVIDER, 1);
                    on_ground = host.client_reset_vl_ground().unwrap_or(false);
                }
                Effect::Velocity(y) => host.client_reset_vl_velocity(y),
                Effect::Disable => {
                    let options = raw_options(host);
                    let mut state = self.state();
                    let id = state.behaviors.borrow().query_ids["ResetVL"];
                    if let Err(error) = state.with_registry(options, |registry, callbacks| {
                        registry.set_enabled_sync(id, false, callbacks)
                    }) {
                        eprintln!("[LiquidBounce] ResetVL disable: {error:?}");
                    }
                    // CancelSequences must preserve the synchronous remainder:
                    // notification observes the incremented count before reset.
                }
                Effect::NotifySuccess => {
                    #[cfg(feature = "webview")]
                    self.state().events.borrow_mut().push(("notification".into(), serde_json::json!({
                        "title":"ResetVL", "message":success_message(&horizon_client::lang::code()), "severity":"SUCCESS",
                    })));
                }
            }
            effect = {
                let state = self.state();
                let mut modules = state.behaviors.borrow_mut();
                let next = modules.reset_vl.after_effect(current, on_ground);
                for id in modules.reset_vl.take_wait_registrations() {
                    modules.coroutine_ticker.register(WaitOwner::ResetVl, id);
                }
                next
            };
        }
    }
    /// GameTick after all global FIFO callbacks and Timer FIRST_PRIORITY.
    pub(crate) fn reset_vl_tick(&self, host: &mut dyn Host) {
        let effect = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "ResetVL");
            modules
                .timer
                .set_provider_running(RESET_VL_TIMER_PROVIDER, running);
            modules.reset_vl.start_tick_handler(running)
        };
        self.reset_vl_effects(host, effect);
    }
    /// Wait1 continuation executes at its real position in the shared FIFO.
    pub(crate) fn reset_vl_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let (keep, effect) = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = in_game && modules.query_running(&state.registry, "ResetVL");
            modules
                .timer
                .set_provider_running(RESET_VL_TIMER_PROVIDER, running);
            modules.reset_vl.advance_callback(id, running)
        };
        self.reset_vl_effects(host, effect);
        keep
    }
}
