// SPDX-License-Identifier: GPL-3.0-or-later
//! Real snapshots are obtained outside runtime and rotation-manager locks.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::derp::{ROTATION_PRIORITY, ROTATION_PROCESSOR, Snapshot};

fn register_waits(modules: &mut crate::behaviors::BehaviorModules) {
    for callback in modules.derp.take_wait_registrations() {
        modules
            .coroutine_ticker
            .register(crate::coroutine_ticker::WaitOwner::Derp, callback);
    }
}

impl LiquidBounceHooks {
    pub(crate) fn derp_callback(&self, host: &dyn Host, in_game: bool, callback: u64) -> bool {
        let snapshot = if in_game {
            host.client_derp_state().as_ref().and_then(Snapshot::parse)
        } else {
            None
        };
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let running = snapshot.is_some() && modules.query_running(&state.registry, "Derp");
        let keep = modules
            .derp
            .advance_callback(callback, running, snapshot.unwrap_or_default());
        register_waits(&mut modules);
        keep
    }

    pub(crate) fn derp_tick(&self, host: &dyn Host) {
        let Some(snapshot) = host.client_derp_state().as_ref().and_then(Snapshot::parse) else {
            return;
        };
        let mut state = self.state();
        #[cfg(not(feature = "webview"))]
        let _ = &mut state;
        let mut modules = state.behaviors.borrow_mut();
        let running = modules.query_running(&state.registry, "Derp");
        modules.derp.start_yaw_handler(running, snapshot);
        register_waits(&mut modules);
        let result = modules.derp.rotation_target(
            running,
            snapshot,
            &mut liquidbounce_behaviors::random::random_unit_f32,
        );
        match result {
            Ok(Some(target)) => {
                modules
                    .rotations
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .request_processed(4, ROTATION_PRIORITY, target, ROTATION_PROCESSOR, true);
            }
            Ok(None) => {}
            Err(error) => eprintln!("[LiquidBounce] Derp: {error}"),
        }
        drop(modules);
        #[cfg(feature = "webview")]
        if running {
            let output = match result {
                Ok(Some(target)) => serde_json::json!([target.rotation.yaw, target.rotation.pitch]),
                _ => serde_json::Value::Null,
            };
            state.probe("clientDerpTick", serde_json::json!({"yaw":snapshot.yaw,"pitch":snapshot.pitch,"sprintKey":snapshot.sprint_key,"sprinting":snapshot.sprinting}), output);
        }
    }
}
