// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_bow_math::V,
    auto_pearl::{self as model, Effect, Spawn},
    no_rotate_set::{RotationsSettings, SmoothMode},
    random,
    rotations::{DefaultRotationTarget, Rotation, modern_mouse_gcd},
};
use serde_json::{Value, json};
pub const ROTATION_PROVIDER: u64 = 21;
pub const PROCESSOR_BASE: u64 = 280;
pub const REQUESTER: u64 = 0x4155_544f_5045_4152;
#[derive(Default)]
pub struct AutoPearlRuntime {
    pub behavior: model::AutoPearl,
    pub rotations: RotationsSettings,
    /// Populated by the actual KillAura runtime once that module is ported.
    pub aura_target: Option<i32>,
    callback: u64,
    waiting: bool,
}
impl AutoPearlRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn clear(&mut self) {
        self.behavior.disabled();
        self.waiting = false;
        self.callback = self.callback.wrapping_add(1);
        self.aura_target = None;
    }
}
fn query(host: &mut dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_auto_pearl_query(&v.to_string())).unwrap_or(Value::Null)
}
fn vec(v: &Value) -> Option<V> {
    let a = v.as_array()?;
    Some(V(
        a.first()?.as_f64()?,
        a.get(1)?.as_f64()?,
        a.get(2)?.as_f64()?,
    ))
}
impl LiquidBounceHooks {
    fn auto_pearl_request(&self, rotation: Rotation) {
        let s = self.state();
        let b = s.behaviors.borrow();
        let r = &b.auto_pearl.rotations;
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(
                ROTATION_PROVIDER,
                35,
                DefaultRotationTarget {
                    rotation,
                    horizontal_turn_speed: 180.,
                    vertical_turn_speed: 180.,
                    ticks_until_reset: r.ticks_until_reset,
                    reset_threshold: r.reset_threshold,
                    consider_inventory: false,
                    movement_correction: r.movement_correction,
                },
                PROCESSOR_BASE + r.active.processor_id(),
                true,
            );
    }
    fn auto_pearl_effect(&self, host: &mut dyn Host, e: Option<Effect>) {
        let Some(Effect::Throw { slot, rotation }) = e else {
            return;
        };
        let [yaw, pitch] = host.client_player_rotation();
        let (current, range) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = b.rotations.lock().expect("RotationManager");
            (r.current, b.auto_pearl.behavior.settings.slot_reset)
        };
        let rotation = match rotation.normalize(
            current.unwrap_or(Rotation::new(yaw, pitch)),
            modern_mouse_gcd(host.client_mouse_sensitivity()),
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[LiquidBounce] AutoPearl normalize: {e:?}");
                return;
            }
        };
        let delay = random::random_i32_inclusive(range[0], range[1]);
        if slot >= 0 {
            let _ = host.client_select_slot_silently(REQUESTER, slot, delay);
        }
        host.client_use_item(if slot < 0 { 1 } else { 0 }, rotation.yaw, rotation.pitch);
    }
    /// RotationUpdateEvent: drain actual packet facts, then service queue.front.
    pub(crate) fn auto_pearl_prepare(&self, host: &mut dyn Host) {
        let facts = query(host, json!({"op":"spawns"}));
        let (running, freeze, aura) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.query_running(&s.registry, "AutoPearl"),
                b.query_running(&s.registry, "Freeze"),
                if b.query_running(&s.registry,"KillAura"){b.kill_aura.behavior.target}else{None},
            )
        };
        if !running {
            return;
        }
        if !facts["slot"].is_null() {
            for spawn in facts["spawns"].as_array().into_iter().flatten() {
                let owner = spawn["owner"].as_i64().map(|i| i as i32);
                let attackable = crate::targeting::snapshot(&spawn["target"].to_string())
                    .is_some_and(|t| {
                        let mut s = self.state();
                        let info = s.target_info(owner.unwrap_or(0), &t);
                        s.global_targets.combat.should_be_attacked(&t.entity, info)
                    });
                let trigger = {
                    let s = self.state();
                    let b = s.behaviors.borrow();
                    b.auto_pearl.behavior.can_trigger(Spawn {
                        owner,
                        owner_self: spawn["self"] == true,
                        owner_attackable: attackable,
                        aura_target: aura,
                        crosshair_cos: spawn["cosine"].as_f64().unwrap_or(-1.),
                    })
                };
                if !trigger {
                    continue;
                }
                let solved = query(
                    host,
                    json!({"op":"solve","owner":owner,"position":spawn["position"],"velocity":spawn["velocity"],"freeze":freeze}),
                );
                let (Some(player), Some(destination), Some(r)) = (
                    vec(&facts["player"]),
                    vec(&solved["destination"]),
                    solved["rotation"].as_array().filter(|r| r.len() == 2),
                ) else {
                    continue;
                };
                let (Some(yaw), Some(pitch)) = (r[0].as_f64(), r[1].as_f64()) else {
                    continue;
                };
                let state = self.state();
                let mut s = state.behaviors.borrow_mut();
                if s.auto_pearl
                    .behavior
                    .destination_allowed(player, destination)
                    && s.auto_pearl
                        .behavior
                        .throw_allowed(destination, vec(&solved["hit"]))
                {
                    s.auto_pearl
                        .behavior
                        .enqueue(Rotation::new(yaw as f32, pitch as f32));
                }
            }
        }
        let pending = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.auto_pearl.behavior.rotation_update().map(|r| {
                (
                    r,
                    b.auto_pearl.behavior.settings.rotate,
                    b.auto_pearl.behavior.settings.combat_pause,
                )
            })
        };
        if let Some((r, rotate, pause)) = pending {
            self.state().combat.pause_combat_for_at_least(pause);
            if rotate {
                self.auto_pearl_request(r);
            }
        }
    }
    pub(crate) fn auto_pearl_tick(&self, host: &mut dyn Host) {
        let running = {
            let s = self.state();
            s.behaviors.borrow().query_running(&s.registry, "AutoPearl")
        };
        if !running {
            return;
        }
        let slot = query(host, json!({"op":"slot"})).as_i64().map(|i| i as i32);
        let effect = {
            self.state()
                .behaviors
                .borrow_mut()
                .auto_pearl
                .behavior
                .begin_tick(slot)
        };
        self.auto_pearl_effect(host, effect);
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        if b.auto_pearl.behavior.pending.is_some() && !b.auto_pearl.waiting {
            b.auto_pearl.waiting = true;
            b.auto_pearl.callback = b.auto_pearl.callback.wrapping_add(1);
            let callback = b.auto_pearl.callback;
            b.coroutine_ticker.register(WaitOwner::AutoPearl, callback);
        }
    }
    pub(crate) fn auto_pearl_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let target = {
            let s = self.state();
            let b = s.behaviors.borrow();
            if !in_game
                || !b.query_running(&s.registry, "AutoPearl")
                || !b.auto_pearl.waiting
                || b.auto_pearl.callback != id
            {
                return false;
            }
            b.auto_pearl.behavior.pending.map(|p| p.rotation)
        };
        let Some(target) = target else {
            return false;
        };
        let slot = query(host, json!({"op":"slot"})).as_i64().map(|i| i as i32);
        // Coroutine requires guard is checked before every callback's predicate.
        if slot.is_none() {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            b.auto_pearl.behavior.pending = None;
            b.auto_pearl.waiting = false;
            return false;
        }
        // Source always requests aim, including the success and final timeout callback.
        self.auto_pearl_request(target);
        let server = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.rotations
                .lock()
                .expect("RotationManager")
                .server_rotation(
                    b.criticals.is_lagging(),
                    b.query_running(&s.registry, "Freeze"),
                )
        };
        let angle=query(host,json!({"op":"angle","rotation":[server.yaw,server.pitch],"target":[target.yaw,target.pitch]})).as_f64().unwrap_or(f64::INFINITY)as f32;
        let (effect, keep) = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let r = b.auto_pearl.behavior.resume(slot, angle);
            b.auto_pearl.waiting = r.1;
            r
        };
        self.auto_pearl_effect(host, effect);
        keep
    }
}
