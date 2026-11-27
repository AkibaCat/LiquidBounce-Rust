// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientDirectionalInputEvent, Host};
use liquidbounce_behaviors::{
    auto_wind_charge::{self as model, Context},
    camera_clip::ModifierKey,
    no_rotate_set::{RotationsSettings, SmoothMode},
    random,
    rotations::{DefaultRotationTarget, Rotation, modern_mouse_gcd, movement_yaw},
};
pub const ROTATION_PROVIDER: u64 = 16;
pub const PROCESSOR_BASE: u64 = 200;
pub const REQUESTER: u64 = 0x5749_4e44_4348_4152;
#[derive(Default)]
pub struct AutoWindChargeRuntime {
    pub behavior: model::AutoWindCharge,
    pub rotations: RotationsSettings,
    pub input: [bool; 7],
}
impl AutoWindChargeRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn clear(&mut self) {
        self.behavior.disabled();
        self.input = [false; 7];
    }
    pub fn record_input(&mut self, event: &ClientDirectionalInputEvent) {
        self.input = [
            event.forward,
            event.backward,
            event.left,
            event.right,
            event.jump,
            event.shift,
            event.sprint,
        ];
    }
}
impl LiquidBounceHooks {
    pub(crate) fn auto_wind_charge_tick(&self, host: &mut dyn Host) {
        let (running, key, input) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            (
                b.query_running(&state.registry, "AutoWindCharge"),
                b.auto_wind_charge.behavior.settings.key.clone(),
                b.auto_wind_charge.input,
            )
        };
        if !running {
            return;
        }
        let Some(v) = host.client_auto_wind_charge_snapshot() else {
            return;
        };
        let [yaw, pitch] = host.client_player_rotation();
        let [last_yaw, last_pitch] = host.client_player_last_rotation();
        let player = Rotation::new(yaw, pitch);
        let player_last = Rotation::new(last_yaw, last_pitch);
        let boost =
            ModifierKey::from_name(&key).is_some_and(|k| host.client_keyboard_scan_down(k.value));
        let c = Context {
            fall_flying: v["fallFlying"] == true,
            swimming: v["swimming"] == true,
            in_liquid: v["liquid"] == true,
            jump_key: v["jump"] == true,
            slot: v["slot"].as_i64().map(|i| i as i32),
            cooldown: v["cooldown"] == true,
            on_ground: v["ground"] == true,
            boost_key: boost,
            yaw,
            keys: [input[0], input[1], input[2], input[3]],
        };
        let rotation = match self
            .state()
            .behaviors
            .borrow_mut()
            .auto_wind_charge
            .behavior
            .begin(&c, &mut random::random_unit_f32)
        {
            Ok(Some(r)) => r,
            Ok(None) => return,
            Err(e) => {
                eprintln!("[LiquidBounce] AutoWindCharge pitch: {e}");
                return;
            }
        };
        let (settings, rotations, server, current, last, actual, motion_yaw) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let mut r = b.rotations.lock().expect("RotationManager");
            let correction = r
                .active_target(|provider| b.rotation_provider_running(&state.registry, provider))
                .map(|t| t.movement_correction);
            (
                b.auto_wind_charge.behavior.settings.clone(),
                b.auto_wind_charge.rotations.clone(),
                r.server_rotation(
                    b.criticals.is_lagging(),
                    b.query_running(&state.registry, "Freeze"),
                ),
                r.current,
                r.last,
                r.actual_server,
                movement_yaw(yaw, r.current, correction),
            )
        };
        let ticks = if !c.on_ground && settings.rotate {
            match model::calculate_ticks(
                &rotations,
                actual,
                rotation,
                last,
                player_last,
                &mut random::random_unit_f32,
            ) {
                Ok(v) => v.wrapping_add(1),
                Err(e) => {
                    eprintln!("[LiquidBounce] AutoWindCharge landing aim: {e}");
                    return;
                }
            }
        } else {
            0
        };
        let query=serde_json::json!({"server":[server.yaw,server.pitch],"target":[rotation.yaw,rotation.pitch],"ticks":ticks,"movementYaw":motion_yaw,"keys":input}).to_string();
        let Ok(v) =
            serde_json::from_str::<serde_json::Value>(&host.client_auto_wind_charge_query(&query))
        else {
            return;
        };
        let Some(angle) = v["angle"].as_f64() else {
            return;
        };
        let effects = self
            .state()
            .behaviors
            .borrow_mut()
            .auto_wind_charge
            .behavior
            .finish(&c, rotation, !v["landing"].is_null(), angle as f32);
        if let Some(rotation) = effects.aim {
            self.state()
                .combat
                .pause_combat_for_at_least(settings.combat_pause);
            self.state()
                .behaviors
                .borrow()
                .rotations
                .lock()
                .expect("RotationManager")
                .request_processed(
                    ROTATION_PROVIDER,
                    35,
                    DefaultRotationTarget {
                        rotation,
                        horizontal_turn_speed: 180.,
                        vertical_turn_speed: 180.,
                        ticks_until_reset: rotations.ticks_until_reset,
                        reset_threshold: rotations.reset_threshold,
                        consider_inventory: false,
                        movement_correction: rotations.movement_correction,
                    },
                    PROCESSOR_BASE + rotations.active.processor_id(),
                    true,
                );
        }
        if let Some(slot) = effects.throw {
            let throw = if settings.rotate {
                server
            } else {
                match rotation.normalize(
                    current.unwrap_or(player),
                    modern_mouse_gcd(host.client_mouse_sensitivity()),
                ) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("[LiquidBounce] AutoWindCharge normalize: {e:?}");
                        return;
                    }
                }
            };
            let delay =
                random::random_i32_inclusive(settings.slot_reset[0], settings.slot_reset[1]);
            if slot >= 0 {
                let _ = host.client_select_slot_silently(REQUESTER, slot, delay);
            }
            host.client_use_item(if slot < 0 { 1 } else { 0 }, throw.yaw, throw.pitch);
        }
    }
}
