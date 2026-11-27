// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    sprint::movement_direction,
    vehicle_control::{Input, RehookAction},
};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn vehicle_control_enabled(&self, host: &mut dyn Host) {
        host.client_vehicle_control_apply(&json!({"notice":"quit"}).to_string());
    }
    pub(crate) fn vehicle_control_tick(&self, host: &mut dyn Host) {
        let (running, target) = {
            let state = self.state();
            let m = state.behaviors.borrow();
            (
                m.query_running(&state.registry, "VehicleControl"),
                m.vehicle_control.vehicle_id,
            )
        };
        if !running {
            return;
        }
        let Some(v) = host.client_vehicle_control_state(target) else {
            return;
        };
        let controlled = v["controlled"].as_i64();
        let (input, dir, spear, mouse, notice) = {
            let state = self.state();
            let mut m = state.behaviors.borrow_mut();
            let spear = m
                .query_ids
                .get("SpearKill")
                .filter(|id| state.registry.enabled_without_world(**id))
                .and_then(|_| m.spear_kill.current_charge_attack_movement());
            let [forwards, backwards, left, right] = m.vehicle_control.direction;
            let dir = liquidbounce_behaviors::rotations::DirectionalInput {forwards,backwards,left,right};
            let s = &mut m.vehicle_control;
            let notice = s.movement_notice(controlled.is_some(), v["use"] == true);
            (
                Input {
                    moving: dir.is_moving(),
                    sprint: v["sprint"] == true,
                    jump: v["jump"] == true,
                    sneak: v["sneak"] == true,
                    use_down: v["use"] == true,
                    yaw: v["playerYaw"].as_f64().unwrap_or(0.) as f32,
                    vehicle_yaw: v["yaw"].as_f64().unwrap_or(0.) as f32,
                    vehicle_water: v["water"] == true,
                },
                dir,
                spear,
                s.settings.mouse_control,
                notice,
            )
        };
        if controlled.is_some() {
            let yaw = movement_direction(if mouse { input.yaw } else { input.vehicle_yaw }, dir);
            let trig = host.client_java_sin_cos(f64::from(yaw).to_radians());
            let velocity = self
                .state()
                .behaviors
                .borrow()
                .vehicle_control
                .velocity(input, trig, spear);
            host.client_vehicle_control_apply(
                &json!({"velocity":velocity,"mouse":mouse}).to_string(),
            );
        }
        if notice {
            host.client_vehicle_control_apply(&json!({"notice":"quit"}).to_string());
        }
        self.vehicle_control_rehook(host, &v);
    }
    fn vehicle_control_rehook(&self, host: &mut dyn Host, v: &Value) {
        let action = {
            let state = self.state();
            let mut m = state.behaviors.borrow_mut();
            if !m.vehicle_control.settings.rehook {
                m.vehicle_control.cancel();
                return;
            }
            let (action, wait) = m.vehicle_control.start_rehook(
                v["passenger"] == true,
                v["exists"] == true,
                v["reach"] == true,
            );
            if let Some(id) = wait {
                m.coroutine_ticker.register(WaitOwner::VehicleControl, id);
            }
            action
        };
        let request = match action {
            Some(RehookAction::Interact(id)) => json!({"interact":id}),
            Some(RehookAction::ForceMount(id)) => json!({"mount":id}),
            Some(RehookAction::TooFar) => json!({"notice":"far"}),
            Some(RehookAction::Gone) => json!({"notice":"gone"}),
            None => return,
        };
        host.client_vehicle_control_apply(&request.to_string());
    }
    pub(crate) fn vehicle_control_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        id: u64,
    ) -> bool {
        let controlled = host
            .client_vehicle_control_state(-1)
            .and_then(|v| v["controlled"].as_i64())
            .map(|v| v as i32);
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        if !in_game
            || !m.query_running(&state.registry, "VehicleControl")
            || !m.vehicle_control.settings.rehook
        {
            m.vehicle_control.cancel();
            return false;
        }
        let (keep, next) = m.vehicle_control.callback(id, controlled);
        if let Some(next) = next {
            m.coroutine_ticker.register(WaitOwner::VehicleControl, next);
        }
        keep
    }
    pub(crate) fn vehicle_control_input(&self, host: &mut dyn Host, event: &mut horizon_client::mod_api::ClientDirectionalInputEvent) {
        self.state().behaviors.borrow_mut().vehicle_control.direction=[event.forward,event.backward,event.left,event.right];
        let shift=&mut event.shift;
        let running = {
            let state = self.state();
            let m = state.behaviors.borrow();
            m.query_running(&state.registry, "VehicleControl")
        };
        let Some(v) = host.client_vehicle_control_state(-1) else {
            return;
        };
        let detach = {
            let state = self.state();
            let mut m = state.behaviors.borrow_mut();
            // once<MovementInputEvent> survives the ordinary listener's running guard.
            if running {
                let result = m.vehicle_control.input(
                    v["controlled"].is_u64(),
                    v["ground"] == true || v["water"] == true,
                    *shift,
                );
                *shift = result.0;
                result.1
            } else {
                let detach = std::mem::take(&mut m.vehicle_control.detach_on_input);
                if detach {
                    *shift = true;
                }
                detach
            }
        };
        if detach {
            host.client_vehicle_control_apply(&json!({"detach":true}).to_string());
        }
    }
}
