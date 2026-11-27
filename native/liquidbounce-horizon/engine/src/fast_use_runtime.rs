// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::fast_use::{Context, Step};
pub(crate) const TIMER_PROVIDER: u64 = u64::MAX - 3;
fn context(host: &dyn Host) -> Context {
    host.client_fast_use_context()
        .map_or(Context::default(), |c| Context {
            ground: c[0] != 0.0,
            moving: c[1] != 0.0,
            regeneration: c[2] != 0.0,
            using: c[3] != 0.0,
            consumable: c[4] != 0.0,
            ticks_using: c[5] as i32,
            crossbow: c[6] != 0.0,
            player_ticks: c[7] as i32,
        })
}
impl LiquidBounceHooks {
    fn fast_use_step(&self, host: &mut dyn Host, step: Step) {
        if let Some((speed, ticks)) = step.timer {
            self.state()
                .behaviors
                .borrow()
                .timer
                .state
                .lock()
                .expect("Timer poisoned")
                .timer
                .request_speed(speed, 20, TIMER_PROVIDER, ticks);
        }
        if let Some(id) = step.wait {
            self.state()
                .behaviors
                .borrow()
                .coroutine_ticker
                .register(WaitOwner::FastUse, id);
        }
        for _ in 0..step.send {
            let kind = self
                .state()
                .behaviors
                .borrow()
                .fast_use
                .settings
                .packet_type;
            host.client_old_regen_send(kind);
        }
        if step.release {
            host.client_fast_use_release();
        }
        if step.crossbow {
            host.client_fast_use_crossbow();
        }
    }
    pub(crate) fn fast_use_tick(&self, host: &mut dyn Host) {
        let c = context(host);
        let step = {
            let state = self.state();
            let mut m = state.behaviors.borrow_mut();
            let running = m.query_running(&state.registry, "FastUse");
            m.fast_use.game_tick(running, c)
        };
        self.fast_use_step(host, step);
    }
    pub(crate) fn fast_use_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let c = context(host);
        let (keep, step) = {
            let state = self.state();
            let mut m = state.behaviors.borrow_mut();
            let running = in_game && m.query_running(&state.registry, "FastUse");
            m.fast_use.callback(id, running, c)
        };
        self.fast_use_step(host, step);
        keep
    }
}
