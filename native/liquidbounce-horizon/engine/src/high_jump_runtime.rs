// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::high_jump::TickContext;
impl LiquidBounceHooks {
    /// Called synchronously from Hooks::client_high_jump, with no Game Host
    /// borrowed while MotionWorld owns its player. No packet/effect is delayed.
    pub(crate) fn high_jump_jump(&self, motion: &mut f32) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let running = modules.query_running(&state.registry, "HighJump");
        if let Some(id) = modules.high_jump.jump(running, motion) {
            modules.coroutine_ticker.register(WaitOwner::HighJump, id);
        }
    }
    pub(crate) fn high_jump_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let (keep, y) = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = in_game && modules.query_running(&state.registry, "HighJump");
            modules.high_jump.advance_callback(id, running)
        };
        if let Some(y) = y {
            host.client_high_jump_vertical(y);
            #[cfg(feature = "webview")]
            self.state().probe(
                "clientHighJumpWait",
                serde_json::json!({"id":id,"player":host.client_high_jump_state()}),
                serde_json::json!(y),
            );
        }
        keep
    }
    /// GameTickEvent after all CoroutineTicker callbacks and before entities.
    pub(crate) fn high_jump_tick(&self, host: &mut dyn Host) {
        let Some([ground, fall_distance, count]) = host.client_high_jump_state() else {
            return;
        };
        let y = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "HighJump");
            modules.high_jump.game_tick(
                running,
                TickContext {
                    on_ground: ground != 0.0,
                    fall_distance,
                    tick_count: count as i32,
                },
            )
        };
        if let Some(y) = y {
            host.client_high_jump_vertical(y);
            #[cfg(feature="webview")]
            self.state().probe("clientHighJumpGlide",serde_json::json!({"onGround":ground!=0.0,"fallDistance":fall_distance,"tickCount":count as i32}),serde_json::json!(y));
        }
    }
}
