// SPDX-License-Identifier: GPL-3.0-or-later
//! Coroutine continuations and native effects never hold each other's locks.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    anti_afk::{Effect, Snapshot},
    random::with_thread_random,
    rotations::Rotation,
};

fn register_waits(modules: &mut crate::behaviors::BehaviorModules) {
    for callback in modules.anti_afk.take_wait_registrations() {
        modules
            .coroutine_ticker
            .register(crate::coroutine_ticker::WaitOwner::AntiAfk, callback);
    }
}

impl LiquidBounceHooks {
    fn anti_afk_context(&self, host: &dyn Host) -> Option<(Snapshot, Rotation)> {
        let snapshot = Snapshot::parse(&host.client_anti_afk_state()?)?;
        let state = self.state();
        let modules = state.behaviors.borrow();
        let server = modules
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .server_rotation(false, false);
        Some((snapshot, server))
    }

    fn anti_afk_effects(&self, host: &mut dyn Host, mut next: Option<Effect>) {
        while let Some(effect) = next {
            match effect {
                Effect::ForwardKey => {
                    host.client_anti_afk_forward();
                }
                Effect::SwingHand => {
                    host.client_anti_afk_swing();
                }
                Effect::SelectedSlot(slot) => {
                    host.client_anti_afk_select_slot(slot);
                }
                Effect::Yaw(yaw) => {
                    host.client_anti_afk_rotation(Some(yaw), None);
                }
                Effect::Pitch(pitch) => {
                    host.client_anti_afk_rotation(None, Some(pitch));
                }
                Effect::Rotation(request) => {
                    let state = self.state();
                    let modules = state.behaviors.borrow();
                    modules
                        .rotations
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .request_processed(3, 20, request.target, request.processor_id(), true);
                }
            }
            #[cfg(feature = "webview")]
            self.state().probe(
                "clientAntiAFKEffect",
                serde_json::json!({"effect":format!("{effect:?}")}),
                serde_json::json!(true),
            );
            let Some((snapshot, server)) = self.anti_afk_context(host) else {
                return;
            };
            let result = {
                let state = self.state();
                let mut modules = state.behaviors.borrow_mut();
                let result = with_thread_random(|random| {
                    modules
                        .anti_afk
                        .after_effect(effect, snapshot, server, random)
                });
                register_waits(&mut modules);
                result
            };
            next = match result {
                Ok(next) => next,
                Err(error) => {
                    eprintln!("[LiquidBounce] AntiAFK continuation: {error}");
                    return;
                }
            };
        }
    }

    /// Dispatch only the callback at the current global FIFO position. Host
    /// effects and later snapshots run after releasing every behavior lock.
    pub(crate) fn anti_afk_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        callback: u64,
    ) -> bool {
        let context = if in_game {
            self.anti_afk_context(host)
        } else {
            None
        };
        let result = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = context.is_some() && modules.query_running(&state.registry, "AntiAFK");
            let (snapshot, server) = context.unwrap_or_default();
            let result = with_thread_random(|random| {
                modules
                    .anti_afk
                    .advance_callback(callback, running, snapshot, server, random)
            });
            register_waits(&mut modules);
            result
        };
        match result {
            Ok((keep, effect)) => {
                self.anti_afk_effects(host, effect);
                keep
            }
            Err(error) => {
                eprintln!("[LiquidBounce] AntiAFK wait: {error}");
                false
            }
        }
    }

    pub(crate) fn anti_afk_tick(&self, host: &mut dyn Host) {
        let Some((snapshot, _)) = self.anti_afk_context(host) else {
            return;
        };
        let swing = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "AntiAFK");
            let effect = modules.anti_afk.start_swing_handler(running, snapshot);
            register_waits(&mut modules);
            effect
        };
        self.anti_afk_effects(host, swing);
        let Some((snapshot, server)) = self.anti_afk_context(host) else {
            return;
        };
        let result = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "AntiAFK");
            let result = with_thread_random(|random| {
                modules
                    .anti_afk
                    .start_main_handler(running, snapshot, server, random)
            });
            register_waits(&mut modules);
            result
        };
        match result {
            Ok(effect) => self.anti_afk_effects(host, effect),
            Err(error) => eprintln!("[LiquidBounce] AntiAFK handler: {error}"),
        }
    }
}
