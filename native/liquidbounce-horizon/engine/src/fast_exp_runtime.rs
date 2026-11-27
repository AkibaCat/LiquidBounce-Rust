// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    fast_exp::{self as model, Context, Effects, RepairItem},
    no_rotate_set::{RotationsSettings, SmoothMode},
    random,
    rotations::{DefaultRotationTarget, Rotation},
};
pub const ROTATION_PROVIDER: u64 = 12;
pub const PROCESSOR_BASE: u64 = 140;
pub const REQUESTER: u64 = 0x4641_5354_4558_50;
#[derive(Default)]
pub struct FastExpRuntime {
    pub behavior: model::FastExp,
    pub rotations: RotationsSettings,
}
impl FastExpRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
}
fn sample(r: [f32; 2]) -> f32 {
    if r[0] >= r[1] {
        r[0]
    } else {
        random::with_thread_random(|v| v.next_f32_between(r[0], r[1].next_up()))
    }
}
impl LiquidBounceHooks {
    fn fast_exp_snapshot(&self, host: &dyn Host) -> Option<Context> {
        let v = host.client_fast_exp_snapshot()?;
        let state = self.state();
        let m = state.behaviors.borrow();
        let freeze = m.query_running(&state.registry, "Freeze");
        let lag=m.criticals.is_lagging(); // Backtrack adds its independent incoming queue when integrated.
        let pitch = m
            .rotations
            .lock()
            .expect("RotationManager")
            .server_rotation(lag, freeze)
            .pitch;
        Some(Context {
            valid: v["valid"] == true,
            handled: v["handled"] == true,
            slot: v["slot"].as_i64().map(|s| s as i32),
            count: v["count"].as_i64().unwrap_or(0) as i32,
            orbs: v["orbs"] == true,
            items: v["items"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|i| RepairItem {
                    damage: i["damage"].as_i64().unwrap_or(0) as i32,
                    max_damage: i["max"].as_i64().unwrap_or(0) as i32,
                })
                .collect(),
            pitch,
        })
    }
    fn fast_exp_effects(&self, host: &mut dyn Host, c: &Context, e: Effects) {
        let (settings, rotations) = {
            let state = self.state();
            let m = state.behaviors.borrow();
            (m.fast_exp.behavior.settings, m.fast_exp.rotations.clone())
        };
        if e.pause {
            self.state()
                .combat
                .pause_combat_for_at_least(settings.combat_pause);
        }
        if e.rotation {
            let [yaw, _] = host.client_player_rotation();
            let target = DefaultRotationTarget {
                rotation: Rotation::new(yaw, 90.),
                horizontal_turn_speed: 180.,
                vertical_turn_speed: 180.,
                ticks_until_reset: rotations.ticks_until_reset,
                reset_threshold: rotations.reset_threshold,
                consider_inventory: false,
                movement_correction: rotations.movement_correction,
            };
            self.state()
                .behaviors
                .borrow()
                .rotations
                .lock()
                .expect("RotationManager")
                .request_processed(
                    ROTATION_PROVIDER,
                    35,
                    target,
                    PROCESSOR_BASE + rotations.active.processor_id(),
                    true,
                );
        }
        let Some(_) = c.slot else {
            return;
        };
        for _ in 0..e.throws {
            let Some(current) = self
                .fast_exp_snapshot(host)
                .filter(|c| c.valid && c.slot.is_some())
            else {
                self.state()
                    .behaviors
                    .borrow_mut()
                    .fast_exp
                    .behavior
                    .cancel();
                return;
            };
            let slot = current.slot.unwrap();
            let delay =
                random::random_i32_inclusive(settings.slot_reset[0], settings.slot_reset[1]);
            if slot >= 0 {
                let _ = host.client_select_slot_silently(REQUESTER, slot, delay);
            }
            let [yaw, _] = host.client_player_rotation();
            let yaw = {
                let state = self.state();
                let m = state.behaviors.borrow();
                m.rotations
                    .lock()
                    .expect("RotationManager")
                    .current
                    .map_or(yaw, |r| r.yaw)
            };
            host.client_use_item(if slot < 0 { 1 } else { 0 }, yaw, e.pitch);
        }
        if let Some(id) = e.register {
            let state = self.state();
            let m = state.behaviors.borrow();
            if id == m.fast_exp.behavior.callback {
                m.coroutine_ticker.register(WaitOwner::FastExp, id);
            }
        }
    }
    pub(crate) fn fast_exp_tick(&self, host: &mut dyn Host) {
        let running = {
            let state = self.state();
            let m = state.behaviors.borrow();
            m.query_running(&state.registry, "FastExp")
        };
        if !running {
            return;
        }
        let Some(c) = self.fast_exp_snapshot(host) else {
            return;
        };
        let e = self
            .state()
            .behaviors
            .borrow_mut()
            .fast_exp
            .behavior
            .start(&c, &mut sample);
        self.fast_exp_effects(host, &c, e);
    }
    pub(crate) fn fast_exp_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let running = {
            let state = self.state();
            let m = state.behaviors.borrow();
            m.query_running(&state.registry, "FastExp")
        };
        let c = self.fast_exp_snapshot(host).unwrap_or_default();
        if !in_game || !running {
            self.state()
                .behaviors
                .borrow_mut()
                .fast_exp
                .behavior
                .cancel();
            return false;
        }
        let (keep, e) = self
            .state()
            .behaviors
            .borrow_mut()
            .fast_exp
            .behavior
            .resume(id, &c, &mut sample);
        self.fast_exp_effects(host, &c, e);
        keep
    }
}
