// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    extinguish::{Action, Context, Extinguish},
    no_rotate_set::{RotationsSettings, SmoothMode},
    rotations::{DefaultRotationTarget, Rotation, movement_yaw},
};
use serde_json::{Value, json};
pub const ROTATION_PROVIDER: u64 = 23;
pub const PROCESSOR_BASE: u64 = 320;
pub const REQUESTER: u64 = 0x4558_5449_4e47_5549;
#[derive(Default)]
pub struct ExtinguishRuntime {
    pub behavior: Extinguish<Value>,
    pub rotations: RotationsSettings,
}
impl ExtinguishRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
}
fn query(host: &mut dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_extinguish_query(&v.to_string())).unwrap_or(Value::Null)
}
impl LiquidBounceHooks {
    pub(crate) fn extinguish_prepare(&self, host: &mut dyn Host) {
        let player_rotation = host.client_player_rotation();
        let (running, in_combat, rotation, yaw) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let mut manager = b.rotations.lock().expect("RotationManager");
            let correction = manager
                .active_target(|p| b.rotation_provider_running(&s.registry, p))
                .map(|t| t.movement_correction);
            (
                b.query_running(&s.registry, "Extinguish"),
                s.combat.in_combat(),
                manager.server_rotation(
                    b.criticals.is_lagging(),
                    b.query_running(&s.registry, "Freeze"),
                ),
                movement_yaw(player_rotation[0], manager.current, correction),
            )
        };
        if !running {
            return;
        }
        let input = horizon_client::reverse_step_bridge::keys();
        let facts = query(host, json!({"op":"facts"}));
        if !facts.is_object() {
            return;
        }
        // The original handler returns before clearing its previous target in
        // an evaporating dimension, and submits no new rotation request.
        if facts["evaporates"] == true {
            return;
        }
        let context = Context {
            now: crate::behaviors::current_time_millis(),
            water_evaporates: facts["evaporates"] == true,
            fire_resistant: facts["fireResistance"] == true,
            in_combat,
            on_fire: facts["onFire"] == true,
        };
        let target = {
            // Native contextual shapes may synchronously query other modules.
            // Release RefCell before crossing the Host boundary.
            let mut behavior =
                std::mem::take(&mut self.state().behaviors.borrow_mut().extinguish.behavior);
            behavior.rotation_update(context,|action|{
                let request=match action{Action::Pickup(pos)=>json!({"op":"plan","pickup":pos,"serverRotation":[rotation.yaw,rotation.pitch]}),Action::Extinguish=>json!({"op":"plan","keys":input,"movementYaw":yaw,"serverRotation":[rotation.yaw,rotation.pitch]})};
                let plan=query(host,request);plan.is_object().then_some(plan)
            });
            let target = behavior.current_target.clone();
            self.state().behaviors.borrow_mut().extinguish.behavior = behavior;
            target
        };
        let Some(target) = target else {
            return;
        };
        let Some(a) = target["rotation"].as_array().filter(|a| a.len() == 2) else {
            return;
        };
        let (Some(y), Some(p)) = (a[0].as_f64(), a[1].as_f64()) else {
            return;
        };
        let s = self.state();
        let b = s.behaviors.borrow();
        let r = &b.extinguish.rotations;
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(
                ROTATION_PROVIDER,
                40,
                DefaultRotationTarget {
                    rotation: Rotation::new(y as f32, p as f32),
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
    pub(crate) fn extinguish_tick(&self, host: &mut dyn Host) {
        let plan = {
            let s = self.state();
            let b = s.behaviors.borrow();
            if !b.query_running(&s.registry, "Extinguish") {
                return;
            }
            b.extinguish.behavior.current_target.clone()
        };
        let Some(plan) = plan else {
            return;
        };
        let [y, p] = host.client_player_rotation();
        let rotation = self
            .state()
            .behaviors
            .borrow()
            .rotations
            .lock()
            .expect("RotationManager")
            .current
            .unwrap_or(Rotation::new(y, p));
        query(
            host,
            json!({"op":"execute","plan":plan,"rotation":[rotation.yaw,rotation.pitch]}),
        );
    }
    pub(crate) fn extinguish_success(&self, position: [i32; 3]) {
        self.state()
            .behaviors
            .borrow_mut()
            .extinguish
            .behavior
            .succeeded(position, crate::behaviors::current_time_millis());
    }
}
