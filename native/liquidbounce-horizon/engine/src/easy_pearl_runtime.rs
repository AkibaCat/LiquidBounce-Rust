// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientEasyPearlEvent, Host};
use liquidbounce_behaviors::{
    auto_bow_math::V,
    easy_pearl::EasyPearl,
    no_rotate_set::{RotationsSettings, SmoothMode},
    rotations::{DefaultRotationTarget, Rotation},
};
use serde_json::{Value, json};
pub const ROTATION_PROVIDER: u64 = 22;
pub const PROCESSOR_BASE: u64 = 300;
pub const REQUESTER: u64 = 0x4541_5359_5045_4152;
#[derive(Default)]
pub struct EasyPearlRuntime {
    pub behavior: EasyPearl,
    pub rotations: RotationsSettings,
}
impl EasyPearlRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
}
fn query(host: &dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_easy_pearl_query(&v.to_string())).unwrap_or(Value::Null)
}
fn position(v: &Value) -> Option<V> {
    let p = serde_json::from_value::<[f64; 3]>(v.clone()).ok()?;
    Some(V(p[0], p[1], p[2]))
}
fn rotation(v: &Value) -> Option<Rotation> {
    let r = serde_json::from_value::<[f32; 2]>(v.clone()).ok()?;
    Some(Rotation::new(r[0], r[1]))
}
fn solve(host: &dyn Host, target: V) -> Option<Rotation> {
    rotation(&query(host, json!({"op":"solve","target":target.array()})))
}
impl LiquidBounceHooks {
    fn easy_pearl_angle(&self, host: &dyn Host, target: Rotation) -> f32 {
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
        query(host,json!({"op":"angle","rotation":[server.yaw,server.pitch],"target":[target.yaw,target.pitch]})).as_f64().unwrap_or(f64::INFINITY)as f32
    }
    pub(crate) fn easy_pearl(&self, host: &dyn Host, event: &mut ClientEasyPearlEvent<'_>) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let running = {
            let s = self.state();
            s.behaviors.borrow().query_running(&s.registry, "EasyPearl")
        };
        if input["phase"] == "options" {
            let show = self
                .state()
                .behaviors
                .borrow()
                .easy_pearl
                .behavior
                .settings
                .show_distance;
            *event.output = json!({"enabled":running,"showDistance":show}).to_string();
            return;
        }
        if !running || input["phase"] != "interact" {
            return;
        }
        let Some(hit) = position(&input["hit"]) else {
            return;
        };
        let solved = rotation(&input["rotation"]);
        let angle = solved.map_or(0., |r| self.easy_pearl_angle(host, r));
        let result = self
            .state()
            .behaviors
            .borrow_mut()
            .easy_pearl
            .behavior
            .interact(
                input["holding"] == true,
                input["useDown"] == true,
                hit,
                input["miss"] == true,
                solved,
                angle,
            );
        let warning = if result.warning {
            liquidbounce_interop::localization::describe(
                &horizon_client::lang::code(),
                "liquidbounce.module.easyPearl.messages.noInReachWarning",
            )
        } else {
            String::new()
        };
        *event.output = json!({"cancel":result.cancel,"warning":warning}).to_string();
    }
    pub(crate) fn easy_pearl_prepare(&self, host: &mut dyn Host) {
        let target = {
            let s = self.state();
            let b = s.behaviors.borrow();
            if !b.query_running(&s.registry, "EasyPearl") {
                return;
            }
            b.easy_pearl.behavior.target_position
        };
        let Some(rotation) = target.and_then(|p| solve(host, p)) else {
            return;
        };
        let s = self.state();
        let b = s.behaviors.borrow();
        let r = &b.easy_pearl.rotations;
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(
                ROTATION_PROVIDER,
                40,
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
    pub(crate) fn easy_pearl_tick(&self, host: &mut dyn Host) {
        let target = {
            let s = self.state();
            let b = s.behaviors.borrow();
            if !b.query_running(&s.registry, "EasyPearl") {
                return;
            }
            b.easy_pearl.behavior.target_position
        };
        let Some(target) = target else {
            return;
        };
        let solved = solve(host, target);
        let Some(current) = solved else {
            return;
        };
        // isRotationDone performs its own solve after the source assigns currentTargetRotation.
        let check = solve(host, target);
        let angle = check.map_or(0., |r| self.easy_pearl_angle(host, r));
        let done = self
            .state()
            .behaviors
            .borrow()
            .easy_pearl
            .behavior
            .rotation_done(check, angle);
        let slot = query(host, json!({"op":"slot"})).as_i64().map(|i| i as i32);
        let effect = self
            .state()
            .behaviors
            .borrow_mut()
            .easy_pearl
            .behavior
            .tick(Some(current), done, slot);
        let Some((slot, r)) = effect else {
            return;
        };
        if slot >= 0 {
            let _ = host.client_select_slot_silently(REQUESTER, slot, 0);
        }
        host.client_use_item(if slot < 0 { 1 } else { 0 }, r.yaw, r.pitch);
        self.state()
            .behaviors
            .borrow_mut()
            .easy_pearl
            .behavior
            .thrown();
    }
}
