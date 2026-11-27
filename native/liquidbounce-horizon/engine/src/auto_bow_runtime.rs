// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_bow::{self as model, Candidate, ShootEffect, Shot, Weapon},
    clicking::JavaRandom,
    no_rotate_set::{RotationsSettings, SmoothMode},
    rotations::{DefaultRotationTarget, Rotation},
    target_renderer::{TargetRendererRuntime, TargetRendererSettings},
};
use serde_json::{Value, json};
pub const ROTATION_PROVIDER: u64 = 19;
pub const PROCESSOR_BASE: u64 = 240;
#[derive(Default)]
pub struct AutoBowRuntime {
    pub behavior: model::AutoBow,
    pub rotations: RotationsSettings,
    pub random: JavaRandom,
    pub target_renderer: TargetRendererSettings,
    pub target_render_runtime: TargetRendererRuntime,
}
impl AutoBowRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn clear(&mut self) {
        self.behavior.disabled();
    }
}
fn query(host: &mut dyn Host, value: Value) -> Value {
    serde_json::from_str(&host.client_auto_bow_query(&value.to_string())).unwrap_or(Value::Null)
}
impl LiquidBounceHooks {
    pub(crate) fn auto_bow_use_key(&self, pressed: &mut bool) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        let b = &modules.auto_bow;
        if modules.query_running(&state.registry, "AutoBow")
            && b.behavior.settings.auto_shoot
            && b.behavior.force_uncharged
        {
            *pressed = false;
        }
    }
    pub(crate) fn auto_bow_tick(&self, host: &mut dyn Host) {
        let (running, settings, server, active, freeze) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let mut rotations = b.rotations.lock().expect("RotationManager");
            (
                b.query_running(&state.registry, "AutoBow"),
                b.auto_bow.behavior.settings.clone(),
                rotations.server_rotation(
                    b.criticals.is_lagging(),
                    b.query_running(&state.registry, "Freeze"),
                ),
                rotations
                    .active_target(|id| b.rotation_provider_running(&state.registry, id))
                    .map(|r| r.rotation),
                b.query_running(&state.registry, "Freeze"),
            )
        };
        if !running {
            return;
        }
        let snapshot = query(host, json!({"op":"snapshot"}));
        if snapshot.is_null() {
            self.state().behaviors.borrow_mut().auto_bow.clear();
            return;
        }
        let mut allowed = vec![];
        let mut candidates = vec![];
        for e in snapshot["candidates"].as_array().into_iter().flatten() {
            let Some(id) = e["id"].as_i64().map(|i| i as i32) else {
                continue;
            };
            let Some(target) = crate::targeting::snapshot(&e["target"].to_string()) else {
                continue;
            };
            let valid = {
                let mut state = self.state();
                let info = state.target_info(id, &target);
                state
                    .global_targets
                    .combat
                    .should_be_attacked(&target.entity, info)
            };
            if valid {
                allowed.push(id);
                candidates.push(Candidate {
                    id,
                    type_weight: e["type"].as_i64().unwrap_or(i32::MAX as i64) as i32,
                    health: e["health"].as_f64().unwrap_or(0.) as f32,
                    distance: e["distance"].as_f64().unwrap_or(0.),
                    cosine: e["cosine"].as_f64().unwrap_or(-1.),
                    hurt: e["hurt"].as_i64().unwrap_or(0) as i32,
                    age: e["age"].as_i64().unwrap_or(0) as i32,
                });
            }
        }
        // Source registration order is AutoShoot, BowAimbot, FastCharge. AutoShoot sees the previous target.
        let using = (!snapshot["using"].is_null()).then_some(&snapshot["using"]);
        let item = using.or_else(|| {
            settings
                .precharged
                .then_some(&snapshot["precharged"])
                .filter(|v| !v.is_null() && snapshot["isUsing"] != true)
        });
        let hit = if settings.auto_shoot && settings.requires_hit && using.is_some() {
            query(
                host,
                json!({"op":"hypothetical","rotation":[server.yaw,server.pitch],"allowed":allowed}),
            )
            .as_i64()
            .is_some_and(|id| allowed.contains(&(id as i32)))
        } else {
            false
        };
        let aim_difference = if settings.auto_shoot && !settings.requires_hit && settings.aimbot {
            active.and_then(|target|query(host,json!({"op":"angle","rotation":[server.yaw,server.pitch],"target":[target.yaw,target.pitch]})).as_f64().map(|d|d as f32))
        } else {
            None
        };
        let now = crate::behaviors::current_time_millis();
        let shot = item.and_then(|i| {
            Some(Shot {
                weapon: match i["weapon"].as_str()? {
                    "Bow" => Weapon::Bow,
                    "Crossbow" => Weapon::Crossbow,
                    "Trident" => Weapon::Trident,
                    _ => return None,
                },
                hand: i["hand"].as_u64().unwrap_or(0) as u8,
                charged: i["charged"] == true,
                ticks: i["ticks"].as_i64().unwrap_or(0) as i32,
                pull_time: i["pullTime"].as_i64().unwrap_or(0) as i32,
                now,
                hit,
                active_aim: aim_difference,
            })
        });
        let effect = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let b = &mut b.auto_bow;
            b.behavior.last_shot = b
                .behavior
                .last_shot
                .max(snapshot["lastBowStop"].as_i64().unwrap_or(0));
            let AutoBowRuntime {
                behavior, random, ..
            } = b;
            behavior.shoot(shot, &mut || {
                random.gaussian(horizon_client::clicker_math::fdlibm::log)
            })
        };
        if let ShootEffect::Use(hand) = effect {
            host.client_use_item(i32::from(hand), server.yaw, server.pitch);
        }
        if settings.aimbot {
            let sorted = self
                .state()
                .behaviors
                .borrow_mut()
                .auto_bow
                .behavior
                .select(candidates);
            for candidate in sorted {
                let answer = query(
                    host,
                    json!({"op":"aim","id":candidate.id,"throughWalls":settings.through_walls,"freeze":freeze}),
                );
                let Some(a) = answer.as_array().filter(|a| a.len() == 2) else {
                    continue;
                };
                let (Some(yaw), Some(pitch)) = (a[0].as_f64(), a[1].as_f64()) else {
                    continue;
                };
                let state = self.state();
                let mut b = state.behaviors.borrow_mut();
                b.auto_bow.behavior.target = Some(candidate.id);
                let s = b.auto_bow.rotations.clone();
                b.rotations
                    .lock()
                    .expect("RotationManager")
                    .request_processed(
                        ROTATION_PROVIDER,
                        20,
                        DefaultRotationTarget {
                            rotation: Rotation::new(yaw as f32, pitch as f32),
                            horizontal_turn_speed: 180.,
                            vertical_turn_speed: 180.,
                            ticks_until_reset: s.ticks_until_reset,
                            reset_threshold: s.reset_threshold,
                            consider_inventory: false,
                            movement_correction: s.movement_correction,
                        },
                        PROCESSOR_BASE + s.active.processor_id(),
                        true,
                    );
                break;
            }
        }
        if self
            .state()
            .behaviors
            .borrow()
            .auto_bow
            .behavior
            .fast_charge(
                using.is_some_and(|i| i["weapon"] == "Bow"),
                snapshot["ground"] == true,
                snapshot["moving"] == true,
                snapshot["regeneration"] == true,
            )
        {
            for _ in 0..settings.speed {
                let current = query(host, json!({"op":"using"}));
                if current["isUsing"] != true {
                    continue;
                }
                host.client_old_regen_send(settings.packet);
                let _ = host.client_auto_bow_update_use();
            }
        }
    }
}
