// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{auto_rod_runtime::NativeSamples, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_bow::Candidate,
    auto_bow_math::{self as math, B, V},
    auto_shoot::AutoShoot,
    clicking::{Clicker, Cooldown, JavaRandom, Math, Settings as ClickSettings},
    crosshair_clicker::Id,
    no_rotate_set::{RotationsSettings, SmoothMode},
    point_tracker::PointTracker,
    rotations::{DefaultRotationTarget, Rotation},
    target_renderer::{TargetRendererRuntime, TargetRendererSettings},
};
use serde_json::{Value, json};
pub const ROTATION_PROVIDER: u64 = 25;
pub const PROCESSOR_BASE: u64 = 360;
pub const REQUESTER: u64 = 0x4155_544f_5348_4f54;
fn click_math() -> Math {
    Math {
        log: horizon_client::clicker_math::log,
        exp: horizon_client::clicker_math::exp,
        strict_log: horizon_client::clicker_math::fdlibm::log,
    }
}
const USE_COOLDOWN: Cooldown = Cooldown {
    real_ticks: 0.,
    reset_ticks: 0.,
    server_tick_speed: 1.,
    attack_delay: 1.,
    has_cooldown: false,
};
pub struct AutoShootRuntime {
    pub behavior: AutoShoot,
    pub clicker: Clicker,
    pub point_tracker: PointTracker,
    pub random: JavaRandom,
    pub rotations: RotationsSettings,
    pub target_renderer: TargetRendererSettings,
    pub target_render_runtime: TargetRendererRuntime,
}
impl Default for AutoShootRuntime {
    fn default() -> Self {
        let mut r = Self {
            behavior: Default::default(),
            clicker: Clicker::new(
                ClickSettings::use_key(),
                false,
                Default::default(),
                click_math(),
            ),
            point_tracker: Default::default(),
            random: Default::default(),
            rotations: Default::default(),
            target_renderer: Default::default(),
            target_render_runtime: Default::default(),
        };
        r.point_tracker
            .initialize(&mut NativeSamples(&mut r.random));
        r
    }
}
impl AutoShootRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
}
fn query(host: &mut dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_auto_shoot_query(&v.to_string())).unwrap_or(Value::Null)
}
impl LiquidBounceHooks {
    fn auto_shoot_active(&self) -> bool {
        let s = self.state();
        let b = s.behaviors.borrow();
        b.query_running(&s.registry, "AutoShoot")
    }
    fn auto_shoot_pause(&self) -> bool {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let aura = b.query_running(&s.registry, "KillAura");
        b.auto_shoot.behavior.pause_for_aura(aura)
    }
    pub(crate) fn auto_shoot_first_tick(&self) {
        if !self.auto_shoot_active() {
            return;
        }
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let r = &mut b.auto_shoot;
        r.clicker.plan.set_math(click_math());
        r.clicker.tick(self.started.elapsed().as_millis() as i64);
    }
    pub(crate) fn auto_shoot_other_click(&self, id: Id) {
        if id != Id::Use || !self.auto_shoot_active() {
            return;
        }
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let leader = b.crosshair_clicker.clicker(id).plan.clone();
        b.auto_shoot
            .clicker
            .plan
            .pressed_by(&leader, self.started.elapsed().as_millis() as i64);
    }
    pub(crate) fn auto_shoot_player_press(&self, use_key: bool, open: bool, pressed: bool) {
        if use_key && !open && pressed && self.auto_shoot_active() {
            self.state()
                .behaviors
                .borrow_mut()
                .auto_shoot
                .clicker
                .plan
                .pressed_by_player();
        }
    }
    fn auto_shoot_slot(&self, host: &mut dyn Host) -> Option<(u8, bool)> {
        let facts = query(host, json!({"op":"facts"}));
        let (slot, projectile, select, reset) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = &b.auto_shoot.behavior;
            let rows = facts["slots"].as_array()?;
            let matching = std::array::from_fn(|i| {
                rows.get(i).is_some_and(|v| {
                    r.accepts(
                        v["item"].as_str().unwrap_or("minecraft:air"),
                        v["count"].as_i64().unwrap_or(0) as i32,
                        v["projectile"] == true,
                    )
                })
            });
            let selected = facts["selected"].as_u64()? as u8;
            let slot = r.choose_slot(matching, selected)?;
            let index = if slot == 40 { 9 } else { usize::from(slot) };
            (
                slot,
                r.projectile(rows[index]["projectile"] == true),
                r.settings.select_slot,
                r.settings.slot_reset,
            )
        };
        if slot != 40 && facts["selected"].as_u64() != Some(u64::from(slot)) {
            if !select {
                return None;
            }
            let _ = host.client_select_slot_silently(REQUESTER, i32::from(slot), reset);
            if query(host, json!({"op":"facts"}))["selected"].as_u64() != Some(u64::from(slot)) {
                return None;
            }
        }
        Some((slot, projectile))
    }
    fn auto_shoot_aim(&self, host: &mut dyn Host, id: i32, projectile: bool) -> Option<Rotation> {
        let q = query(
            host,
            json!({"op":if projectile{"projectile"}else{"point"},"id":id}),
        );
        let a = if projectile {
            serde_json::from_value::<[f32; 2]>(q).ok()?
        } else {
            let eyes = serde_json::from_value::<[f64; 3]>(q["eye"].clone()).ok()?;
            let bounds = serde_json::from_value::<[f64; 6]>(q["bounds"].clone()).ok()?;
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_shoot;
            let eye = V(eyes[0], eyes[1], eyes[2]);
            math::rotation(
                r.point_tracker
                    .find(
                        eye,
                        B::from_array(bounds),
                        &mut NativeSamples(&mut r.random),
                    )
                    .pos
                    .sub(eye),
            )
        };
        Some(Rotation::new(a[0], a[1]))
    }
    pub(crate) fn auto_shoot_prepare(&self, host: &mut dyn Host) {
        if !self.auto_shoot_active() || self.auto_shoot_pause() {
            return;
        }
        let facts = query(host, json!({"op":"facts"}));
        if !facts.is_object() {
            return;
        }
        let mut candidates = vec![];
        let mut visible = vec![];
        for v in facts["candidates"].as_array().into_iter().flatten() {
            let Some(id) = v["id"].as_i64().map(|v| v as i32) else {
                continue;
            };
            let target = if v["target"].is_object() {
                v["target"].to_string()
            } else {
                host.client_targeting_entity(id)
            };
            let Some(t) = crate::targeting::snapshot(&target) else {
                continue;
            };
            if !t.entity.kind.living {
                continue;
            }
            let mut s = self.state();
            let info = s.target_info(id, &t);
            if !s.global_targets.combat.should_be_attacked(&t.entity, info) {
                continue;
            }
            candidates.push(Candidate {
                id,
                type_weight: v["type"].as_i64().unwrap_or(i64::from(i32::MAX)) as i32,
                health: v["health"].as_f64().unwrap_or(0.) as f32,
                distance: v["distance"].as_f64().unwrap_or(f64::INFINITY),
                cosine: v["cosine"].as_f64().unwrap_or(-1.),
                hurt: v["hurt"].as_i64().unwrap_or(0) as i32,
                age: v["age"].as_i64().unwrap_or(0) as i32,
            });
            if v["visible"] == true {
                visible.push(id);
            }
        }
        let id = {
            let s = self.state();
            let combat = s.combat.in_combat();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_shoot.behavior;
            r.select(candidates, |id| visible.contains(&id));
            if r.settings.not_during_combat && combat {
                return;
            }
            r.target
        };
        let Some(id) = id else {
            return;
        };
        let Some((_, projectile)) = self.auto_shoot_slot(host) else {
            return;
        };
        let Some(rotation) = self.auto_shoot_aim(host, id, projectile) else {
            return;
        };
        let s = self.state();
        let b = s.behaviors.borrow();
        let r = &b.auto_shoot;
        let settings = &r.rotations;
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(
                ROTATION_PROVIDER,
                60,
                DefaultRotationTarget {
                    rotation,
                    horizontal_turn_speed: 180.,
                    vertical_turn_speed: 180.,
                    ticks_until_reset: settings.ticks_until_reset,
                    reset_threshold: settings.reset_threshold,
                    consider_inventory: r.behavior.settings.consider_inventory,
                    movement_correction: settings.movement_correction,
                },
                PROCESSOR_BASE + settings.active.processor_id(),
                true,
            );
    }
    pub(crate) fn auto_shoot_tick(&self, host: &mut dyn Host) {
        if !self.auto_shoot_active() || self.auto_shoot_pause() {
            return;
        }
        let (id, threshold, server) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = &b.auto_shoot;
            if r.behavior.settings.not_during_combat && s.combat.in_combat() {
                return;
            }
            let Some(id) = r.behavior.target else {
                return;
            };
            let server = b
                .rotations
                .lock()
                .expect("RotationManager")
                .server_rotation(
                    b.criticals.is_lagging(),
                    b.query_running(&s.registry, "Freeze"),
                );
            (id, r.behavior.settings.aim_threshold, server)
        };
        let Some((slot, projectile)) = self.auto_shoot_slot(host) else {
            return;
        };
        let Some(aim) = self.auto_shoot_aim(host, id, projectile) else {
            return;
        };
        let Some(angle) = query(
            host,
            json!({"op":"angle","rotation":[server.yaw,server.pitch],"target":[aim.yaw,aim.pitch]}),
        )
        .as_f64() else {
            return;
        };
        if angle > f64::from(threshold) {
            return;
        }
        let mut successes = 0i32;
        loop {
            if !self
                .state()
                .behaviors
                .borrow_mut()
                .auto_shoot
                .clicker
                .take_attempt(USE_COOLDOWN)
            {
                break;
            }
            let facts = query(host, json!({"op":"facts"}));
            let (consider, open, hide, rotation) = {
                let s = self.state();
                let b = s.behaviors.borrow();
                let r = &b.auto_shoot;
                let player = host.client_player_rotation();
                let rotation = b
                    .rotations
                    .lock()
                    .expect("RotationManager")
                    .current
                    .unwrap_or(Rotation::new(player[0], player[1]));
                (
                    r.behavior.settings.consider_inventory,
                    b.inventory.manager.inventory_open_server_side
                        || facts["inventoryOpen"] == true,
                    r.behavior.settings.hide_swing,
                    rotation,
                )
            };
            if facts["using"] == true || (consider && open) {
                continue;
            }
            if query(
                host,
                json!({"op":"use","hand":i32::from(slot==40),"rotation":[rotation.yaw,rotation.pitch],"hideSwing":hide}),
            ) != true
            {
                continue;
            }
            let use_running = self.crosshair_running()[2];
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_shoot;
            r.clicker.record_success(None);
            let leader = r.clicker.plan.clone();
            if use_running {
                b.crosshair_clicker
                    .clicker(Id::Use)
                    .plan
                    .pressed_by(&leader, self.started.elapsed().as_millis() as i64);
            }
            successes = successes.wrapping_add(1);
        }
        self.state()
            .behaviors
            .borrow_mut()
            .auto_shoot
            .clicker
            .finish_click(successes);
    }
}
