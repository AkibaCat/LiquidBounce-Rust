// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    clicking::{Clicker, JavaRandom, Math, Settings as ClickSettings},
    combat_rotations,
    crosshair_clicker::Id,
    rotations::{DefaultRotationTarget, Rotation},
};
use serde_json::Value;
use std::sync::atomic::{AtomicI64, Ordering};
pub const ROTATION_PROVIDER: u64 = 18;
pub const PROCESSOR_BASE: u64 = 4000;
static AI_NOTIFICATION: AtomicI64 = AtomicI64::new(0);
pub struct ProjectilePuncherRuntime {
    pub range: f32,
    pub swing: u8,
    pub ignore_inventory: bool,
    pub projectiles: u8,
    pub only_air: bool,
    pub fov: f32,
    pub clicker: Clicker,
    pub rotations: combat_rotations::Settings,
    pub target: Option<Value>,
    pub ai_notice_pending: bool,
}
impl Default for ProjectilePuncherRuntime {
    fn default() -> Self {
        Self {
            range: 3.,
            swing: 0,
            ignore_inventory: true,
            projectiles: 3,
            only_air: false,
            fov: 180.,
            clicker: Clicker::new(
                ClickSettings {
                    item_cooldown: None,
                    ..Default::default()
                },
                false,
                JavaRandom::default(),
                math(),
            ),
            rotations: Default::default(),
            target: None,
            ai_notice_pending: false,
        }
    }
}
fn math() -> Math {
    Math {
        log: horizon_client::clicker_math::log,
        exp: horizon_client::clicker_math::exp,
        strict_log: horizon_client::clicker_math::fdlibm::log,
    }
}
impl LiquidBounceHooks {
    fn projectile_puncher_running(&self) -> bool {
        let state = self.state();
        let b = state.behaviors.borrow();
        b.query_running(&state.registry, "ProjectilePuncher")
    }
    pub(crate) fn projectile_puncher_first_tick(&self) {
        if !self.projectile_puncher_running() {
            return;
        }
        let now = self.started.elapsed().as_millis() as i64;
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let p = &mut b.projectile_puncher;
        p.clicker.plan.set_math(math());
        p.clicker.tick(now);
        if let Err(e) = p
            .rotations
            .first_tick(true, &mut liquidbounce_behaviors::random::random_unit_f32)
        {
            eprintln!("[LiquidBounce] ProjectilePuncher Fail: {e}");
        }
    }
    pub(crate) fn projectile_puncher_other_click(&self, id: Id) {
        if !id.attack() || !self.projectile_puncher_running() {
            return;
        }
        let now = self.started.elapsed().as_millis() as i64;
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let leader = b.crosshair_clicker.clicker(id).plan.clone();
        b.projectile_puncher.clicker.plan.pressed_by(&leader, now);
    }
    pub(crate) fn projectile_puncher_player_press(&self, attack: bool, open: bool, pressed: bool) {
        if attack && !open && pressed && self.projectile_puncher_running() {
            self.state()
                .behaviors
                .borrow_mut()
                .projectile_puncher
                .clicker
                .plan
                .pressed_by_player();
        }
    }
    pub(crate) fn projectile_puncher_prepare(&self, host: &mut dyn Host) {
        if !self.projectile_puncher_running() {
            return;
        }
        let (range, projectiles, air, fov, base, rotations) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let p = &b.projectile_puncher;
            let manager = b.rotations.lock().expect("RotationManager");
            let [yaw, pitch] = host.client_player_rotation();
            (
                p.range,
                p.projectiles,
                p.only_air,
                p.fov,
                manager.current.unwrap_or(Rotation::new(yaw, pitch)),
                p.rotations.clone(),
            )
        };
        let Some(plan) =
            host.client_projectile_puncher_plan(range, projectiles, air, fov, base.yaw, base.pitch)
        else {
            return;
        };
        // RotationUpdate's spectator early return deliberately retains the old target.
        if plan["spectator"] == true {
            return;
        }
        self.state()
            .behaviors
            .borrow_mut()
            .projectile_puncher
            .target = plan.get("target").filter(|v| !v.is_null()).cloned();
        let Some(aim) = plan["rotation"].as_array().filter(|a| a.len() == 2) else {
            return;
        };
        let Some(yaw) = aim[0].as_f64() else {
            return;
        };
        let Some(pitch) = aim[1].as_f64() else {
            return;
        };
        let state = self.state();
        let b = state.behaviors.borrow();
        let p = &b.projectile_puncher;
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(
                ROTATION_PROVIDER,
                60,
                DefaultRotationTarget {
                    rotation: Rotation::new(yaw as f32, pitch as f32),
                    horizontal_turn_speed: 180.,
                    vertical_turn_speed: 180.,
                    ticks_until_reset: rotations.basic.ticks_until_reset,
                    reset_threshold: rotations.basic.reset_threshold,
                    consider_inventory: !p.ignore_inventory,
                    movement_correction: rotations.basic.movement_correction,
                },
                PROCESSOR_BASE + rotations.processor_id(),
                true,
            );
    }
    pub(crate) fn projectile_puncher_tick(&self, host: &mut dyn Host) {
        if !self.projectile_puncher_running() {
            return;
        }
        let (target, range, swing, rotation) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let p = &b.projectile_puncher;
            let lag = b.criticals.is_lagging();
            let freeze = b.query_running(&state.registry, "Freeze");
            let rotation = b
                .rotations
                .lock()
                .expect("RotationManager")
                .server_rotation(lag, freeze);
            (p.target.clone(), p.range, p.swing, rotation)
        };
        let Some(target) = target else {
            return;
        };
        let Some(id) = target["id"].as_u64().map(|v| v as u32) else {
            return;
        };
        if !host.client_projectile_puncher_ready(
            &target.to_string(),
            range,
            rotation.yaw,
            rotation.pitch,
        ) {
            return;
        }
        let mut clicks = 0i32;
        loop {
            let Some((cooldown, _)) = self.crosshair_cooldown(host) else {
                break;
            };
            if !self
                .state()
                .behaviors
                .borrow_mut()
                .projectile_puncher
                .clicker
                .take_attempt(cooldown)
            {
                break;
            }
            let Some((cooldown, v)) = self.crosshair_cooldown(host) else {
                break;
            };
            if !self
                .state()
                .behaviors
                .borrow()
                .projectile_puncher
                .clicker
                .passes_gate(cooldown, v["missTime"].as_i64().unwrap_or(0) as i32)
            {
                continue;
            }
            // The original lambda always returns true, including cancelled attacks.
            host.client_projectile_puncher_attack(id, swing);
            let running = self.crosshair_running();
            let now = self.started.elapsed().as_millis() as i64;
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let p = &mut b.projectile_puncher;
            p.clicker.record_success(None);
            let leader = p.clicker.plan.clone();
            for (id, index) in [(Id::Trigger, 0), (Id::Attack, 1)] {
                if running[index] {
                    b.crosshair_clicker
                        .clicker(id)
                        .plan
                        .pressed_by(&leader, now);
                }
            }
            if b.query_running(&state.registry,"KillAura"){b.kill_aura.clicker.plan.pressed_by(&leader,now);}
            clicks = clicks.wrapping_add(1);
        }
        self.state()
            .behaviors
            .borrow_mut()
            .projectile_puncher
            .clicker
            .finish_click(clicks);
    }
    pub(crate) fn projectile_puncher_ai_notification(&self, host: &mut dyn Host) {
        let pending = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            std::mem::take(&mut b.projectile_puncher.ai_notice_pending)
        };
        if !pending {
            return;
        }
        let now = crate::behaviors::current_time_millis();
        let previous = AI_NOTIFICATION.load(Ordering::Relaxed);
        if now.wrapping_sub(previous) < 5000 {
            return;
        }
        AI_NOTIFICATION.store(now, Ordering::Relaxed);
        host.chat(match horizon_client::lang::code().as_str(){"zh_cn"=>"当前平台不支持深度学习旋转；AI 旋转回退至 Interpolation。","zh_tw"=>"目前平台不支援深度學習旋轉；AI 旋轉回退至 Interpolation。",_=>"Deep learning is unsupported on this platform. AI rotations fall back to Interpolation."});
    }
}
