// SPDX-License-Identifier: GPL-3.0-or-later
//! KillAura's RotationUpdate, click scheduler and native action boundaries.
use crate::{auto_rod_runtime::NativeSamples, runtime::LiquidBounceHooks};
use horizon_client::mod_api::{Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::{
    auto_bow::{self, Candidate},
    auto_bow_math::{B, V},
    clicking::{Clicker, Cooldown, JavaRandom, Math},
    combat_rotations,
    crosshair_clicker::Id,
    kill_aura::{self as model, AutoBlock},
    point_tracker::{PointTracker, Samples},
    random,
    rotations::{DefaultRotationTarget, Rotation, modern_mouse_gcd},
    target_renderer::{Frame, Quad, TargetRendererRuntime, TargetRendererSettings},
};
use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering},
};
pub const ROTATION_PROVIDER: u64 = 24;
pub const PROCESSOR_BASE: u64 = 340;
static AI_NOTIFICATION: AtomicI64 = AtomicI64::new(0);
#[derive(Default)]
pub struct KillAuraBlocking {
    pub state: Mutex<AutoBlock>,
    active: AtomicBool,
    pose: AtomicBool,
    limit: AtomicI32,
}
impl KillAuraBlocking {
    pub fn clear(&self) {
        *self.state.lock().expect("AutoBlocking") = AutoBlock::default();
        self.active.store(false, Ordering::Release);
        self.pose.store(false, Ordering::Release);
    }
    pub fn outgoing(&self, p: &S) {
        let carried = matches!(p, S::SetCarriedItem(_));
        let swap = matches!(p,S::PlayerAction(a) if a.action==7);
        self.state
            .lock()
            .expect("AutoBlocking")
            .outgoing(carried, swap);
    }
    pub fn queues(&self, outgoing: bool, use_item: bool) -> bool {
        if !outgoing {
            return false;
        }
        let mut state = self.state.lock().expect("AutoBlocking");
        let pose = state.wants_pose
            && self.active.load(Ordering::Acquire)
            && self.pose.load(Ordering::Acquire);
        state.queues(pose, use_item, self.limit.load(Ordering::Acquire))
    }
    pub fn snapshot(&self, enforced: &mut i32, pose: &mut bool) {
        let state = self.state.lock().expect("AutoBlocking");
        *enforced = state.enforced.map_or(-1, i32::from);
        *pose = state.wants_pose
            && self.active.load(Ordering::Acquire)
            && self.pose.load(Ordering::Acquire);
    }
}
pub struct KillAuraRuntime {
    pub behavior: model::KillAura,
    pub clicker: Clicker,
    pub rotations: combat_rotations::Settings,
    pub point_tracker: PointTracker,
    pub random: JavaRandom,
    pub target_renderer: TargetRendererSettings,
    pub target_render_runtime: TargetRendererRuntime,
    pub navigation: crate::auto_walk_runtime::AutoWalkSettings,
    pub blocking: Arc<KillAuraBlocking>,
    pub rotation_entity: Option<(bool, f64)>,
    pub facts: Value,
    pub ai_notice_pending: bool,
    pub release_on_disable: bool,
}
impl Default for KillAuraRuntime {
    fn default() -> Self {
        Self::new(Arc::default())
    }
}
impl KillAuraRuntime {
    pub fn new(blocking: Arc<KillAuraBlocking>) -> Self {
        let mut r = Self {
            behavior: Default::default(),
            clicker: Default::default(),
            rotations: Default::default(),
            point_tracker: Default::default(),
            random: Default::default(),
            target_renderer: Default::default(),
            target_render_runtime: Default::default(),
            navigation: Default::default(),
            blocking,
            rotation_entity: None,
            facts: Value::Null,
            ai_notice_pending: false,
            release_on_disable: false,
        };
        r.navigation.render.enabled = false;
        r.point_tracker
            .initialize(&mut NativeSamples(&mut r.random));
        r.behavior.scan_addition = NativeSamples(&mut r.random).float(r.behavior.settings.scan);
        r.behavior.fail_addition =
            NativeSamples(&mut r.random).float(r.behavior.settings.fail.additional);
        r
    }
    pub fn clear(&mut self) {
        self.release_on_disable |= self
            .blocking
            .state
            .lock()
            .expect("AutoBlocking")
            .enforced
            .is_some();
        self.behavior.disabled();
        self.blocking.clear();
        self.rotation_entity = None;
        self.facts = Value::Null;
    }
}
fn query(host: &mut dyn Host, value: Value) -> Value {
    serde_json::from_str(&host.client_kill_aura_query(&value.to_string())).unwrap_or(Value::Null)
}
fn math() -> Math {
    Math {
        log: horizon_client::clicker_math::log,
        exp: horizon_client::clicker_math::exp,
        strict_log: horizon_client::clicker_math::fdlibm::log,
    }
}
fn rotation(v: Value) -> Option<Rotation> {
    serde_json::from_value::<[f32; 2]>(v)
        .ok()
        .map(|a| Rotation::new(a[0], a[1]))
}
impl LiquidBounceHooks {
    fn kill_aura_running(&self) -> bool {
        let s = self.state();
        s.behaviors.borrow().query_running(&s.registry, "KillAura")
    }
    pub(crate) fn kill_aura_first_tick(&self) {
        if !self.kill_aura_running() {
            return;
        }
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let r = &mut b.kill_aura;
        r.clicker.plan.set_math(math());
        r.clicker.tick(self.started.elapsed().as_millis() as i64);
        if let Err(e) = r.rotations.first_tick(true, &mut random::random_unit_f32) {
            eprintln!("[LiquidBounce] KillAura rotations: {e}");
        }
    }
    fn kill_aura_requirements(&self, host: &dyn Host, v: &Value) -> [bool; 6] {
        let click = match v["attackKey"]
            .as_str()
            .and_then(crate::input::configured_bind)
        {
            Some(liquidbounce_core::InputKey::Keyboard(scan)) => {
                host.client_keyboard_scan_down(scan)
            }
            Some(liquidbounce_core::InputKey::Mouse(button)) => self
                .state()
                .behaviors
                .borrow()
                .auto_rod
                .input
                .mouse_click(button, crate::behaviors::current_time_millis()),
            _ => false,
        };
        [
            click,
            self.state()
                .behaviors
                .borrow()
                .auto_rod
                .input
                .mouse_moved(crate::behaviors::current_time_millis()),
            v["weapon"] == true,
            v["emptyHand"] == true,
            v["vanillaName"] == true,
            v["notBreaking"] == true,
        ]
    }
    fn kill_aura_current(&self, host: &dyn Host) -> Rotation {
        let [y, p] = host.client_player_rotation();
        self.state()
            .behaviors
            .borrow()
            .rotations
            .lock()
            .expect("RotationManager")
            .current
            .unwrap_or(Rotation::new(y, p))
    }
    fn kill_aura_find_rotation(
        &self,
        host: &mut dyn Host,
        id: i32,
        range: f32,
        walls: f32,
    ) -> Option<Rotation> {
        let current = self.kill_aura_current(host);
        let settings = self
            .state()
            .behaviors
            .borrow()
            .kill_aura
            .behavior
            .settings
            .clone();
        if settings.lazy
            && query(
                host,
                json!({"op":"hit","id":id,"rotation":[current.yaw,current.pitch],"range":range,"walls":walls}),
            )["hit"]
                == true
        {
            return Some(current);
        }
        let v = query(host, json!({"op":"point","id":id}));
        let eye = serde_json::from_value::<[f64; 3]>(v["eye"].clone()).ok()?;
        let bounds = serde_json::from_value::<[f64; 6]>(v["bounds"].clone()).ok()?;
        let point = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.kill_aura;
            r.point_tracker.find(
                V(eye[0], eye[1], eye[2]),
                B::from_array(bounds),
                &mut NativeSamples(&mut r.random),
            )
        };
        let b = point.bounds;
        let q = json!({"op":"raytrace","bounds":[b.lo.0,b.lo.1,b.lo.2,b.hi.0,b.hi.1,b.hi.2],"point":point.pos.array(),"range":range,"walls":walls});
        let a = rotation(query(host, q.clone()));
        a.or_else(|| {
            if settings.through {
                let mut q = q;
                q["walls"] = json!(range);
                rotation(query(host, q))
            } else {
                None
            }
        })
    }
    fn kill_aura_request(&self, r: Rotation) {
        let s = self.state();
        let b = s.behaviors.borrow();
        if b.query_running(&s.registry,"ElytraTarget")&&b.elytra_target.behavior.settings.ignore_kill_aura{return;}
        let a = &b.kill_aura;
        let t = &a.rotations.basic;
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(
                ROTATION_PROVIDER,
                20,
                DefaultRotationTarget {
                    rotation: r,
                    horizontal_turn_speed: 180.,
                    vertical_turn_speed: 180.,
                    ticks_until_reset: t.ticks_until_reset,
                    reset_threshold: t.reset_threshold,
                    consider_inventory: !a.behavior.settings.ignore_inventory,
                    movement_correction: t.movement_correction,
                },
                PROCESSOR_BASE + a.rotations.processor_id(),
                true,
            );
    }
    pub(crate) fn kill_aura_prepare(&self, host: &mut dyn Host) {
        if !self.kill_aura_running() {
            return;
        }
        let ground = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let fight = &b.kill_aura.behavior.settings.fight;
            fight.enabled && fight.void
        };
        let facts = query(host, json!({"op":"facts","ground":ground}));
        if !facts.is_object() {
            return;
        }
        let actual = self.kill_aura_requirements(host, &facts);
        let (settings, maximum, range, met) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let inventory =
                b.inventory.manager.inventory_open_server_side || facts["inventoryOpen"] == true;
            let a = &mut b.kill_aura;
            a.facts = facts.clone();
            let met = a.behavior.requirements(actual);
            let ok = a.behavior.prepare(
                facts["dead"] == true,
                facts["spectator"] == true,
                inventory,
                met,
            );
            let attribute = facts["attributeRange"].as_f64().unwrap_or(3.) as f32;
            (
                a.behavior.settings.clone(),
                a.behavior.maximum_range(attribute),
                a.behavior.interaction(attribute),
                ok,
            )
        };
        if !met {
            self.kill_aura_navigation(host);
            return;
        }
        let shield_break = self.kill_aura_weapon_special(host).0;
        let mut candidates = vec![];
        for row in facts["candidates"].as_array().into_iter().flatten() {
            let Some(id) = row["id"].as_i64().map(|i| i as i32) else {
                continue;
            };
            let Some(t) = crate::targeting::snapshot(&row["target"].to_string()) else {
                continue;
            };
            let mut s = self.state();
            let info = s.target_info(id, &t);
            if !t.entity.kind.living || !s.global_targets.combat.should_be_attacked(&t.entity, info)
            {
                continue;
            }
            if !settings.ignore_shield
                && row["wouldBlock"] == true
                && facts["axe"] != true
                && !shield_break
            {
                continue;
            }
            candidates.push(Candidate {
                id,
                type_weight: row["type"].as_i64().unwrap_or(i64::from(i32::MAX)) as i32,
                health: row["health"].as_f64().unwrap_or(0.) as f32,
                distance: row["distance"].as_f64().unwrap_or(f64::INFINITY),
                cosine: row["cosine"].as_f64().unwrap_or(-1.),
                hurt: row["hurt"].as_i64().unwrap_or(0) as i32,
                age: row["age"].as_i64().unwrap_or(0) as i32,
            });
        }
        let selection = auto_bow::Settings {
            fov: settings.fov,
            hurt_time: settings.hurt,
            priority: settings.priority.clone(),
            ..Default::default()
        };
        let mut candidates = auto_bow::targets(&selection, candidates);
        if let Some(c) = candidates
            .iter()
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
        {
            self.state()
                .behaviors
                .borrow_mut()
                .kill_aura
                .behavior
                .closest = c.distance;
        }
        let eligible = candidates.clone();
        candidates.retain(|c| c.distance <= f64::from(maximum * maximum));
        candidates.sort_by_key(|c| i32::from(c.distance > f64::from(range * range)));
        let current = self.kill_aura_current(host);
        let last = host.client_player_last_rotation();
        let mut chosen = None;
        for c in candidates {
            let Some(aim) = self.kill_aura_find_rotation(host, c.id, maximum, settings.walls)
            else {
                continue;
            };
            let cooldown = self.crosshair_cooldown(host).map(|x| x.0);
            let (ticks, will) = {
                let s = self.state();
                let mut b = s.behaviors.borrow_mut();
                let a = &mut b.kill_aura;
                let ticks = liquidbounce_behaviors::kill_aura_rotation::ticks(
                    &a.rotations,
                    current,
                    aim,
                    Rotation::new(last[0], last[1]),
                    &mut random::random_unit_f32,
                )
                .unwrap_or(0);
                let will = cooldown.is_some_and(|cool| a.clicker.amount_at(ticks.max(1), cool) > 0);
                (ticks, will)
            };
            let should = self
                .state()
                .behaviors
                .borrow()
                .kill_aura
                .behavior
                .request_rotation(ticks, will);
            let hit = query(
                host,
                json!({"op":"hit","id":c.id,"rotation":[current.yaw,current.pitch],"range":range,"walls":settings.walls}),
            )["hit"]
                == true;
            self.state()
                .behaviors
                .borrow_mut()
                .kill_aura
                .rotation_entity = Some((hit, c.distance.sqrt()));
            if should {
                self.kill_aura_request(aim);
            }
            chosen = Some(c.id);
            break;
        }
        if chosen.is_none() && settings.fight.enabled {
            chosen = eligible
                .into_iter()
                .find(|c| {
                    c.distance <= f64::from(settings.fight.range.powi(2))
                        && facts["candidates"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .find(|r| r["id"].as_i64() == Some(i64::from(c.id)))
                            .is_some_and(|r| {
                                (!settings.fight.visible || r["visible"] == true)
                                    && (!settings.fight.void || r["void"] != true)
                            })
                })
                .map(|c| c.id);
            if let Some(v) = host
                .client_kill_aura_navigation_result()
                .and_then(|v| serde_json::from_str::<Value>(&v).ok())
            {
                if let Some(r) = rotation(v["movementRotation"].clone()) {
                    self.kill_aura_request(r);
                }
            }
        }
        self.state()
            .behaviors
            .borrow_mut()
            .kill_aura
            .behavior
            .target = chosen;
        self.kill_aura_auto_weapon(host, chosen);
        self.kill_aura_navigation(host);
        let danger_ids = self.kill_aura_eligible(&facts);
        let danger = query(
            host,
            json!({"op":"danger","targets":danger_ids,"range":range,"walls":settings.walls,"tolerance":settings.block.tolerance,"force":settings.block.force}),
        );
        let service = self.state().behaviors.borrow().kill_aura.blocking.clone();
        service
            .active
            .store(settings.block.enabled, Ordering::Release);
        service.limit.store(settings.block.blink, Ordering::Release);
        service.pose.store(
            facts["blockAnimation"] == true || {
                let s = self.state();
                s.behaviors
                    .borrow()
                    .query_running(&s.registry, "SwordBlock")
            },
            Ordering::Release,
        );
        service
            .state
            .lock()
            .expect("AutoBlocking")
            .tick(settings.block.danger && danger == true);
    }
    fn kill_aura_weapon_special(&self, host: &dyn Host) -> (bool, bool) {
        use liquidbounce_behaviors::auto_weapon::Snapshot;
        let Some(w) = host
            .client_weapon_context()
            .as_ref()
            .and_then(Snapshot::parse)
        else {
            return (false, false);
        };
        let silent_slot = host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);
        let s = self.state();
        let b = s.behaviors.borrow();
        let c = w.context(
            silent_slot,
            s.registry.enabled_without_world(b.query_ids["MaceKill"]),
        );
        if !b.query_running(&s.registry, "AutoWeapon") {
            return (false, false);
        }
        (
            b.auto_weapon.will_shield_break(&c),
            b.auto_weapon.will_mace_smash(&c),
        )
    }
    fn kill_aura_eligible(&self, facts: &Value) -> Vec<i32> {
        let mut s = self.state();
        facts["candidates"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| {
                let id = r["id"].as_i64()? as i32;
                let t = crate::targeting::snapshot(&r["target"].to_string())?;
                let info = s.target_info(id, &t);
                s.global_targets
                    .combat
                    .should_be_attacked(&t.entity, info)
                    .then_some(id)
            })
            .collect()
    }
    pub(crate) fn kill_aura_ai_notification(&self, host: &mut dyn Host) {
        let pending = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            std::mem::take(&mut b.kill_aura.ai_notice_pending)
        };
        if pending {
            let now = crate::behaviors::current_time_millis();
            if now.wrapping_sub(AI_NOTIFICATION.load(Ordering::Relaxed)) < 5000 {
                return;
            }
            AI_NOTIFICATION.store(now, Ordering::Relaxed);
            host.chat(match horizon_client::lang::code().as_str(){"zh_cn"=>"当前平台不支持深度学习旋转；AI 旋转回退至 Interpolation。","zh_tw"=>"目前平台不支援深度學習旋轉；AI 旋轉回退至 Interpolation。",_=>"Deep learning is unsupported on this platform. AI rotations fall back to Interpolation."});
        }
    }
    pub(crate) fn kill_aura_block_state(&self, enforced: &mut i32, pose: &mut bool) {
        self.kill_aura_blocking.snapshot(enforced, pose);
    }
    fn kill_aura_auto_weapon(&self, host: &mut dyn Host, target: Option<i32>) {
        use liquidbounce_behaviors::auto_weapon::{REQUESTER, Snapshot, Target};
        let Some(id) = target else {
            return;
        };
        let weapon = host
            .client_weapon_context()
            .as_ref()
            .and_then(Snapshot::parse);
        let owns = host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);
        let t = host.client_crosshair_clicker_target(id);
        let switch = {
            let s = self.state();
            let b = s.behaviors.borrow();
            weapon.as_ref().and_then(|w| {
                b.auto_weapon.on_target(
                    b.query_running(&s.registry, "AutoWeapon"),
                    &w.context(
                        owns,
                        s.registry.enabled_without_world(b.query_ids["MaceKill"]),
                    ),
                    Some(Target {
                        living: t[0],
                        would_block_hit: t[1],
                    }),
                )
            })
        };
        if let Some(v) = switch {
            let _ = host.client_select_slot_silently(REQUESTER, v.slot, v.ticks);
        }
    }
    fn kill_aura_cooldown(&self, host: &mut dyn Host, ahead: i32) -> Option<(Cooldown, i32)> {
        let (mut c, v) = self.crosshair_cooldown(host)?;
        let (shield, mace) = self.kill_aura_weapon_special(host);
        let (ignore, target, increase) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.kill_aura.behavior.settings.cooldown_ignore,
                b.kill_aura.behavior.target,
                b.kill_aura.behavior.settings.increase,
            )
        };
        let bypass = ignore[0]
            && shield
            && target.is_some_and(|id| host.client_crosshair_clicker_target(id)[1])
            || ignore[1] && mace
            || ignore[2] && ahead >= 0 && target.is_some_and(|id| {
                query(
                    host,
                    json!({"op":"exiting","id":id,"ticks":1.+f64::from(ahead),"increase":increase}),
                ) == true
            });
        if bypass {
            c.real_ticks = f64::from(i32::MAX);
        }
        Some((c, v["missTime"].as_i64().unwrap_or(0) as i32))
    }
    fn kill_aura_can_attack(&self, host: &mut dyn Host, target: bool) -> bool {
        let facts = query(host, json!({"op":"canAttack"}));
        if facts != true {
            return false;
        }
        let (settings, wait, inventory, may_use) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.kill_aura.behavior.settings.clone(),
                b.kill_aura.behavior.wait,
                b.inventory.manager.inventory_open_server_side,
                b.multi_actions
                    .policy(b.query_running(&s.registry, "MultiActions"))
                    .1,
            )
        };
        if wait > 0 || inventory && !settings.ignore_inventory && !settings.close_inventory {
            return false;
        }
        if !target {
            return true;
        }
        let Some((cool, v)) = self.crosshair_cooldown(host) else {
            return false;
        };
        if v["fallFlying"] == true || settings.criticals == 1 {
            return true;
        }
        let (jump, adjust, blink) = self.state().behaviors.borrow().criticals.selection();
        let allows = v["allowsCritical"] == true;
        let ground = v["onGround"] == true;
        let sprint = v["sprinting"] == true;
        let legacy = v["legacy"] == true;
        let fall = v["fallDistance"].as_f64().unwrap_or(0.);
        let velocity = v["velocityY"].as_f64().unwrap_or(0.);
        let _ = may_use;
        if settings.criticals == 2 {
            return allows && !ground && cool.progress(0) > 0.9 && (!sprint || legacy) && fall > 0.;
        }
        if blink || !allows || ground && !(v["jumping"] == true || adjust) {
            return true;
        }
        let next = ((cool.attack_delay * 0.9 - 0.5 - cool.ticks(0) as f32)
            / cool.server_tick_speed as f32)
            .max(0.);
        if !ground && velocity <= 0. {
            if fall > 0. && cool.progress(0) > 0.9 {
                return true;
            }
            return host
                .client_crosshair_clicker_falling((next + 1.) as i32, 0.)
                .is_ok_and(|h| h.is_some_and(|n| n < next as i32));
        }
        let initial = if ground { f64::from(jump) } else { velocity };
        let until = (initial / 0.08) as f32;
        host.client_crosshair_clicker_falling(
            (next.max(until) + 5.) as i32,
            if ground { initial } else { 0. },
        )
        .is_ok_and(|h| h.is_some_and(|n| n < until as i32))
    }
    fn kill_aura_stop_block(&self, host: &mut dyn Host, pauses: bool) -> bool {
        let (settings, service) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.kill_aura.behavior.settings.block.clone(),
                b.kill_aura.blocking.clone(),
            )
        };
        if !pauses {
            service.state.lock().expect("AutoBlocking").wants_pose = false;
        }
        let (enforced, using) = {
            let b = service.state.lock().expect("AutoBlocking");
            (b.enforced, query(host, json!({"op":"blocking"})) == true)
        };
        if !pauses && query(host, json!({"op":"physicalUse"})) == true
            || enforced.is_none() && !using
        {
            return false;
        }
        let pause = random::random_i32_inclusive(settings.pause[0], settings.pause[1]);
        service.state.lock().expect("AutoBlocking").pause = pause;
        if pauses && settings.unblock == 3 {
            return false;
        }
        query(host, json!({"op":"unblock","mode":settings.unblock}));
        let cleared = settings.unblock != 1 || enforced == Some(0);
        if cleared {
            service.state.lock().expect("AutoBlocking").enforced = None;
        }
        cleared
    }
    fn kill_aura_start_block(&self, host: &mut dyn Host) {
        let (settings, service, paused) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.kill_aura.behavior.settings.block.clone(),
                b.kill_aura.blocking.clone(),
                s.combat.should_pause_blocking(),
            )
        };
        if !settings.enabled || paused {
            return;
        }
        if settings.danger && !service.state.lock().expect("AutoBlocking").danger {
            self.kill_aura_stop_block(host, false);
            return;
        }
        if query(host, json!({"op":"using"})) == true {
            service
                .state
                .lock()
                .expect("AutoBlocking")
                .blocked_since_attack = true;
            return;
        }
        if !NativeSamples(&mut self.state().behaviors.borrow_mut().kill_aura.random)
            .chance(settings.chance)
        {
            return;
        }
        let hand = query(host, json!({"op":"blockHand","assume":settings.assume}))
            .as_u64()
            .map(|h| h as u8);
        let Some(hand) = hand else {
            return;
        };
        let current = self.kill_aura_current(host);
        if settings.mode == 2 {
            service.state.lock().expect("AutoBlocking").wants_pose = true;
            return;
        }
        let (raycast, range, facts) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let a = &b.kill_aura;
            (
                a.behavior.settings.raycast,
                a.behavior
                    .interaction(a.facts["attributeRange"].as_f64().unwrap_or(3.) as f32),
                a.facts.clone(),
            )
        };
        let eligible = self.kill_aura_eligible(&facts);
        let used = query(
            host,
            json!({"op":"block","mode":settings.mode,"vanilla":settings.vanilla,"hand":hand,"rotation":[current.yaw,current.pitch],"raycast":raycast,"range":range,"eligible":eligible}),
        ) == true;
        let mut b = service.state.lock().expect("AutoBlocking");
        b.wants_pose = true;
        if used {
            b.enforced = Some(hand);
            b.blocked_since_attack = true;
            b.reblock = random::random_i32_inclusive(settings.reblock[0], settings.reblock[1]);
        }
    }
    pub(crate) fn kill_aura_tick(&self, host: &mut dyn Host) {
        if !self.kill_aura_running() {
            return;
        }
        let facts = self.state().behaviors.borrow().kill_aura.facts.clone();
        let actual = self.kill_aura_requirements(host, &facts);
        let (settings, target, range, dead, pause, requirements) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let a = &b.kill_aura;
            (
                a.behavior.settings.clone(),
                a.behavior.target,
                a.behavior
                    .interaction(a.facts["attributeRange"].as_f64().unwrap_or(3.) as f32),
                a.facts["dead"] == true || a.facts["spectator"] == true,
                s.combat.should_pause_combat(),
                a.behavior.requirements(actual),
            )
        };
        if dead {
            return;
        }
        if pause {
            self.kill_aura_stop_block(host, false);
            return;
        }
        let Some(mut id) = target else {
            let unblocked = self.kill_aura_stop_block(host, false);
            if settings.fail.enabled && requirements {
                if !self.kill_aura_pause_after_unblock(unblocked) {
                    self.kill_aura_fail_swing(host, None);
                }
            }
            return;
        };
        if !requirements {
            return;
        }
        let mut current = self.kill_aura_current(host);
        if settings.timing == 2 {
            if let Some(r) = self.kill_aura_find_rotation(host, id, range, settings.walls) {
                current = r;
            }
        }
        let base = self.kill_aura_current(host);
        if let Ok(r) = current.normalize(base, modern_mouse_gcd(host.client_mouse_sensitivity())) {
            current = r;
        }
        if settings.raycast != 0 {
            let q = query(
                host,
                json!({"op":"crosshair","rotation":[current.yaw,current.pitch],"range":range}),
            );
            for row in q.as_array().into_iter().flatten() {
                let Some(candidate) = row["id"].as_i64().map(|i| i as i32) else {
                    continue;
                };
                let enemy = crate::targeting::snapshot(&host.client_targeting_entity(candidate))
                    .is_some_and(|t| {
                        let mut s = self.state();
                        let info = s.target_info(candidate, &t);
                        s.global_targets.combat.should_be_attacked(&t.entity, info)
                    });
                if settings.raycast == 1 && !enemy {
                    continue;
                }
                id = candidate;
                if row["living"] == true && enemy {
                    self.state()
                        .behaviors
                        .borrow_mut()
                        .kill_aura
                        .behavior
                        .target = Some(id);
                }
                break;
            }
        }
        let hit = query(
            host,
            json!({"op":"hit","id":id,"rotation":[current.yaw,current.pitch],"range":range,"walls":settings.walls,"increase":settings.increase}),
        );
        let service = self.state().behaviors.borrow().kill_aura.blocking.clone();
        if settings.block.enabled {
            service.state.lock().expect("AutoBlocking").wants_pose = true;
        }
        if hit["hit"] != true || hit["inRange"] != true {
            if settings.block.enabled
                && settings.block.scan
                && hit["distance"].as_f64().is_some_and(|d| {
                    d <= f64::from(
                        (range.max(settings.walls)
                            + self
                                .state()
                                .behaviors
                                .borrow()
                                .kill_aura
                                .behavior
                                .scan_addition)
                            .powi(2),
                    )
                })
            {
                self.kill_aura_maybe_reblock(host);
                return;
            }
            let unblocked = self.kill_aura_stop_block(host, false);
            if !self.kill_aura_pause_after_unblock(unblocked) && settings.fail.enabled {
                self.kill_aura_fail_swing(host, Some(id));
            }
            return;
        }
        let Some((cool, miss)) = self.kill_aura_cooldown(host, 0) else {
            return;
        };
        let hand = query(
            host,
            json!({"op":"blockHand","assume":settings.block.assume}),
        )
        .as_u64()
        .map(|h| h as u8);
        let using = query(host, json!({"op":"using"})) == true;
        let prioritize =
            service
                .state
                .lock()
                .expect("AutoBlocking")
                .prioritize(&settings.block, using, hand);
        let click = self
            .state()
            .behaviors
            .borrow_mut()
            .kill_aura
            .clicker
            .can_execute(cool, miss);
        if !click || !self.kill_aura_can_attack(host, true) || prioritize {
            self.kill_aura_maybe_reblock(host);
            return;
        }
        if !self.kill_aura_before_click(host) {
            return;
        }
        let inventory = self
            .state()
            .behaviors
            .borrow()
            .inventory
            .manager
            .inventory_open_server_side
            && settings.close_inventory;
        if inventory {
            query(host, json!({"op":"inventory","open":false}));
        }
        if settings.timing == 2 {
            query(
                host,
                json!({"op":"rotation","rotation":[current.yaw,current.pitch]}),
            );
        }
        let mut clicks = 0i32;
        loop {
            let Some((cool, miss)) = self.kill_aura_cooldown(host, 0) else {
                break;
            };
            if !self
                .state()
                .behaviors
                .borrow_mut()
                .kill_aura
                .clicker
                .take_attempt(cool)
            {
                break;
            }
            if !self
                .state()
                .behaviors
                .borrow()
                .kill_aura
                .clicker
                .passes_gate(cool, miss)
                || !self.kill_aura_can_attack(host, true)
            {
                continue;
            }
            let keep_sprint = settings.keep_sprint && !self.kill_aura_stop_sprint(host);
            let critical = self.crosshair_cooldown(host).is_some_and(|(cool, v)| {
                v["allowsCritical"] == true
                    && v["onGround"] != true
                    && cool.progress(0) > 0.9
                    && v["fallDistance"].as_f64().unwrap_or(0.) > 0.
            });
            if query(
                host,
                json!({"op":"attack","id":id,"keepSprint":keep_sprint,"critical":critical}),
            ) != true
            {
                continue;
            }
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let a = &mut b.kill_aura;
            a.behavior.scan_addition = NativeSamples(&mut a.random).float(settings.scan);
            a.behavior.failed_count = 0;
            a.blocking
                .state
                .lock()
                .expect("AutoBlocking")
                .blocked_since_attack = false;
            let next = a
                .clicker
                .settings
                .item_cooldown
                .map(|r| r[0] + (r[1] - r[0]) * random::random_unit_f32());
            a.clicker.record_success(next);
            drop(b);
            drop(s);
            self.kill_aura_notify_click();
            clicks = clicks.wrapping_add(1);
        }
        self.state()
            .behaviors
            .borrow_mut()
            .kill_aura
            .clicker
            .finish_click(clicks);
        if settings.timing == 2 {
            let p = host.client_player_rotation();
            query(host, json!({"op":"rotation","rotation":p}));
        }
        if service.state.lock().expect("AutoBlocking").reblock == 0 {
            self.kill_aura_start_block(host);
        }
        if inventory {
            query(host, json!({"op":"inventory","open":true}));
        }
    }
    fn kill_aura_pause_after_unblock(&self, unblocked: bool) -> bool {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let a = &mut b.kill_aura;
        let pause = a.blocking.state.lock().expect("AutoBlocking").pause;
        if unblocked && pause > 0 {
            a.behavior.wait = pause;
            true
        } else {
            false
        }
    }
    fn kill_aura_maybe_reblock(&self, host: &mut dyn Host) {
        let reblock = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let a = &b.kill_aura;
            let ticks = a.blocking.state.lock().expect("AutoBlocking").reblock;
            a.clicker.ticks_since_last_click >= ticks
        };
        if reblock {
            self.kill_aura_start_block(host);
        }
    }
    fn kill_aura_before_click(&self, host: &mut dyn Host) -> bool {
        let (s, owns, may) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.kill_aura.behavior.settings.block.clone(),
                b.kill_aura
                    .blocking
                    .state
                    .lock()
                    .expect("AutoBlocking")
                    .enforced
                    .is_some(),
                b.multi_actions
                    .policy(b.query_running(&s.registry, "MultiActions"))
                    .1,
            )
        };
        if query(host, json!({"op":"blocking"})) == true || owns {
            if !s.enabled && !may {
                return false;
            }
            if s.enabled && s.unblock != 3 {
                let stopped = self.kill_aura_stop_block(host, true);
                if self.kill_aura_pause_after_unblock(stopped) {
                    return false;
                }
            }
        } else if query(host, json!({"op":"using"})) == true && !may {
            return false;
        }
        true
    }
    fn kill_aura_fail_swing(&self, host: &mut dyn Host, target: Option<i32>) {
        if !self.kill_aura_can_attack(host, false) {
            return;
        }
        let (settings, range) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let a = &b.kill_aura;
            (
                a.behavior.settings.clone(),
                a.behavior
                    .interaction(a.facts["attributeRange"].as_f64().unwrap_or(3.) as f32)
                    + a.behavior.fail_addition,
            )
        };
        let facts = self.state().behaviors.borrow().kill_aura.facts.clone();
        let eligible = self.kill_aura_eligible(&facts);
        let hit = query(
            host,
            json!({"op":"failEligible","id":target,"range":range,"eligible":eligible}),
        );
        let Some(id) = hit["id"].as_i64() else {
            return;
        };
        let Some((cool, miss)) = self.kill_aura_cooldown(host, 0) else {
            return;
        };
        if !self
            .state()
            .behaviors
            .borrow_mut()
            .kill_aura
            .clicker
            .can_execute(cool, miss)
            || !self.kill_aura_before_click(host)
        {
            return;
        }
        let inventory = self
            .state()
            .behaviors
            .borrow()
            .inventory
            .manager
            .inventory_open_server_side
            && settings.close_inventory;
        if inventory {
            query(host, json!({"op":"inventory","open":false}));
        }
        let current = self.kill_aura_current(host);
        let mut clicks = 0i32;
        loop {
            let Some((cool, miss)) = self.kill_aura_cooldown(host, 0) else {
                break;
            };
            if !self
                .state()
                .behaviors
                .borrow_mut()
                .kill_aura
                .clicker
                .take_attempt(cool)
            {
                break;
            }
            if !self
                .state()
                .behaviors
                .borrow()
                .kill_aura
                .clicker
                .passes_gate(cool, miss)
            {
                continue;
            }
            let q = query(
                host,
                json!({"op":"failSwing","id":id,"rotation":[current.yaw,current.pitch],"sound":settings.fail.mode==2,"volume":settings.fail.volume/100.,"pitch":settings.fail.pitch}),
            );
            if let (Ok(eye), Ok(center), Ok(dir)) = (
                serde_json::from_value::<[f64; 3]>(q["eye"].clone()),
                serde_json::from_value::<[f64; 3]>(q["center"].clone()),
                serde_json::from_value::<[f64; 3]>(q["direction"].clone()),
            ) {
                let s = self.state();
                let mut b = s.behaviors.borrow_mut();
                let a = &mut b.kill_aura;
                a.behavior.failed_hit(
                    V(eye[0], eye[1], eye[2]),
                    V(center[0], center[1], center[2]),
                    current,
                    V(dir[0], dir[1], dir[2]),
                );
                let next = a
                    .clicker
                    .settings
                    .item_cooldown
                    .map(|r| r[0] + (r[1] - r[0]) * random::random_unit_f32());
                a.clicker.record_success(next);
            }
            self.kill_aura_notify_click();
            clicks = clicks.wrapping_add(1);
        }
        self.state()
            .behaviors
            .borrow_mut()
            .kill_aura
            .clicker
            .finish_click(clicks);
        let service = self.state().behaviors.borrow().kill_aura.blocking.clone();
        if service.state.lock().expect("AutoBlocking").reblock == 0 {
            self.kill_aura_start_block(host);
        }
        if inventory {
            query(host, json!({"op":"inventory","open":true}));
        }
    }
    fn kill_aura_notify_click(&self) {
        let now = self.started.elapsed().as_millis() as i64;
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let leader = b.kill_aura.clicker.plan.clone();
        for (id, index) in [(Id::Trigger, 0), (Id::Attack, 1)] {
            let running = if index == 0 {
                b.query_running(&s.registry, "TriggerBot")
            } else {
                b.query_running(&s.registry, "AutoClicker")
            };
            if running {
                b.crosshair_clicker
                    .clicker(id)
                    .plan
                    .pressed_by(&leader, now);
            }
        }
        if b.query_running(&s.registry, "ProjectilePuncher") {
            b.projectile_puncher.clicker.plan.pressed_by(&leader, now);
        }
    }
    pub(crate) fn kill_aura_other_click(&self, id: Id) {
        if !id.attack() || !self.kill_aura_running() {
            return;
        }
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let leader = b.crosshair_clicker.clicker(id).plan.clone();
        b.kill_aura
            .clicker
            .plan
            .pressed_by(&leader, self.started.elapsed().as_millis() as i64);
    }
    pub(crate) fn kill_aura_player_press(&self, attack: bool, open: bool, pressed: bool) {
        if attack && !open && pressed && self.kill_aura_running() {
            self.state()
                .behaviors
                .borrow_mut()
                .kill_aura
                .clicker
                .plan
                .pressed_by_player();
        }
    }
    pub(crate) fn kill_aura_attack_event(&self) {
        if self.kill_aura_running() {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let a = &mut b.kill_aura;
            a.behavior.fail_addition =
                NativeSamples(&mut a.random).float(a.behavior.settings.fail.additional);
        }
    }
    pub(crate) fn kill_aura_attack_key(&self, pressed: &mut bool) {
        if self.kill_aura_running() {
            *pressed = self
                .state()
                .behaviors
                .borrow()
                .kill_aura
                .clicker
                .attack_key_down(*pressed);
        }
    }
    pub(crate) fn kill_aura_stop_sprint(&self, host: &dyn Host) -> bool {
        if !self.kill_aura_running() {
            return false;
        }
        let Some((cool, v)) = self.crosshair_cooldown(host) else {
            return false;
        };
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        if b.query_running(&s.registry, "ElytraTarget") {
            return false;
        }
        let a = &mut b.kill_aura;
        a.behavior.target.is_some()
            && a.behavior.settings.criticals != 1
            && v["legacy"] != true
            && v["onGround"] != true
            && a.clicker.amount_at(1, cool) > 0
    }
    fn kill_aura_navigation(&self, host: &mut dyn Host) {
        let current = self.kill_aura_current(host);
        let input_yaw = host.client_player_rotation()[0];
        let cooldown = self.kill_aura_cooldown(host, 0).map(|v| v.0);
        let will = cooldown.is_some_and(|c| {
            self.state()
                .behaviors
                .borrow_mut()
                .kill_aura
                .clicker
                .amount_at(0, c)
                > 0
        });
        let v = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let a = &b.kill_aura;
            let f = &a.behavior.settings.fight;
            let n = &a.navigation;
            json!({"running":b.query_running(&s.registry,"KillAura")&&f.enabled,"enabled":s.registry.enabled_without_world(b.query_ids["KillAura"])&&f.enabled,"targetId":a.behavior.target,"range":a.behavior.interaction(a.facts["attributeRange"].as_f64().unwrap_or(3.)as f32),"opponentRange":f.opponent,"dangerousYaw":f.yaw,"runawayOnCooldown":f.runaway,"willClick":will,"leader":f.leader,"username":f.username,"leaderRadius":f.radius,"inputYaw":input_yaw,"movementYaw":current.yaw,"sent":[current.yaw,current.pitch],"current":[current.yaw,current.pitch],"externalRotation":false,"isFlying":b.query_running(&s.registry,"Fly"),"recastsElytra":b.query_running(&s.registry,"ElytraRecast"),"scaffold":b.query_ids.get("Scaffold").is_some_and(|id|s.registry.enabled_without_world(*id)),"options":{"allowBreak":n.allow_break,"allowPlace":n.allow_place,"allowParkour":n.allow_parkour,"riskyJumps":n.risky_jumps,"maxFall":n.max_fall,"pathRange":n.path_range,"allowFly":n.allow_fly,"allowElytra":n.allow_elytra,"elytraRockets":n.elytra_rockets,"allowBoat":n.allow_boat,"allowEat":n.allow_eat}})
        };
        host.client_kill_aura_navigation_config(&v.to_string());
    }
    pub(crate) fn kill_aura_disabled(&self, host: &mut dyn Host) {
        let release = std::mem::take(
            &mut self
                .state()
                .behaviors
                .borrow_mut()
                .kill_aura
                .release_on_disable,
        );
        if query(host, json!({"op":"physicalUse"})) != true
            && (release || query(host, json!({"op":"blocking"})) == true)
        {
            query(host, json!({"op":"unblock","mode":0}));
        }
        host.client_kill_aura_navigation_config("{\"running\":false,\"enabled\":false}");
    }
}
fn rgba(c: u32) -> [f32; 4] {
    [
        (c >> 16 & 255) as f32 / 255.,
        (c >> 8 & 255) as f32 / 255.,
        (c & 255) as f32 / 255.,
        (c >> 24) as f32 / 255.,
    ]
}
fn quad(frame: &mut Frame, corners: [[f64; 3]; 4], colors: [[f32; 4]; 4], depth: bool) {
    frame.quads.push(Quad {
        corners,
        colors,
        uv: [[0.; 2]; 4],
        shape: 0,
        inner: 0.,
        inner_color: [0.; 4],
        texture: String::new(),
        depth,
    });
}
impl LiquidBounceHooks {
    pub(crate) fn kill_aura_render(&self, input: &Value, frame: &mut Frame) {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        if !b.query_running(&s.registry, "KillAura") {
            return;
        }
        let inventory = b.inventory.manager.inventory_open_server_side;
        let a = &mut b.kill_aura;
        let settings = a.behavior.settings.clone();
        for (v, mut c) in a.behavior.failed_render() {
            if settings.fail.rainbow {
                let hue = (self.started.elapsed().as_secs_f32() / 2.).fract();
                let x = hue * 6.;
                let z = 1. - (x % 2. - 1.).abs();
                let [r, g, b] = match x as i32 {
                    0 => [1., z, 0.],
                    1 => [z, 1., 0.],
                    2 => [0., 1., z],
                    3 => [0., z, 1.],
                    4 => [z, 0., 1.],
                    _ => [1., 0., z],
                };
                c = (c & 0xff000000)
                    | ((r * 255.) as u32) << 16
                    | ((g * 255.) as u32) << 8
                    | (b * 255.) as u32;
            }
            let p = v.array();
            let z = 0.05;
            for axis in 0..3 {
                for side in [0., z] {
                    let mut q = [p; 4];
                    let j = (axis + 1) % 3;
                    let k = (axis + 2) % 3;
                    for (i, (x, y)) in [(0., 0.), (z, 0.), (z, z), (0., z)].into_iter().enumerate()
                    {
                        q[i][axis] += side;
                        q[i][j] += x;
                        q[i][k] += y;
                    }
                    quad(frame, q, [rgba(c); 4], true);
                }
            }
        }
        let i = &settings.indicator;
        if !i.enabled {
            return;
        }
        let f = a.facts.clone();
        if i.hide_dead && f["dead"] == true
            || i.hide_spectator && f["spectator"] == true
            || i.hide_vehicle && f["vehicle"] == true
            || i.inventory
                && !settings.ignore_inventory
                && (f["inventoryOpen"] == true || inventory)
        {
            return;
        }
        let a = &mut b.kill_aura;
        let Ok(mut at) = serde_json::from_value::<[f64; 3]>(input["playerPosition"].clone()) else {
            return;
        };
        at[1] += 0.001;
        let targeted = a.behavior.target.is_some();
        let target = if targeted { 1. } else { 0. };
        a.behavior.color_factor = if i.fade {
            a.behavior.color_factor + i.fade_speed * (target - a.behavior.color_factor)
        } else {
            target
        };
        let range = f["itemMaxRange"]
            .as_f64()
            .unwrap_or(if f["creative"] == true { 5. } else { 3. }) as f32
            + settings.increase;
        let pulse = if i.pulse {
            (self.started.elapsed().as_secs_f32() * i.pulse_speed * std::f32::consts::TAU).sin()
                * i.pulse_intensity
                * range
        } else {
            0.
        };
        let blend = |left: u32, right: u32, factor: f32| {
            let l = rgba(left);
            let r = rgba(right);
            std::array::from_fn(|i| l[i] + (r[i] - l[i]) * factor)
        };
        let color = match i.mode {
            1 => {
                let h = (self.started.elapsed().as_secs_f32() / 2.).fract();
                let x = h * 6.;
                let z = 1. - (x % 2. - 1.).abs();
                let [r, g, b] = match x as i32 {
                    0 => [1., z, 0.],
                    1 => [z, 1., 0.],
                    2 => [0., 1., z],
                    3 => [0., z, 1.],
                    4 => [z, 0., 1.],
                    _ => [1., 0., z],
                };
                [r, g, b, 0.5]
            }
            2 => {
                let distance = a.behavior.target.and_then(|id| {
                    f["candidates"]
                        .as_array()?
                        .iter()
                        .find(|r| r["id"].as_i64() == Some(i64::from(id)))?["distance"]
                        .as_f64()
                });
                distance.map_or(rgba(i.idle), |d| {
                    blend(i.active, i.idle, (d.sqrt() as f32 / range).clamp(0., 1.))
                })
            }
            _ => blend(i.idle, i.active, a.behavior.color_factor),
        };
        let mut circles = vec![(
            range + pulse,
            color,
            255u8,
            f["itemMinRange"].as_f64().unwrap_or(0.) as f32,
        )];
        if i.walls >> 24 > 0 && settings.walls < range {
            let mut c = rgba(i.walls);
            if targeted {
                c[3] /= 1.5;
            }
            circles.push((settings.walls + pulse * 0.5, c, 80, 0.));
        }
        if i.scan >> 24 > 0 {
            circles.push((
                a.behavior
                    .scan_range(f["attributeRange"].as_f64().unwrap_or(3.) as f32),
                rgba(i.scan),
                60,
                0.,
            ));
        }
        if i.opponent >> 24 > 0 && targeted {
            circles.push((3., rgba(i.opponent), 100, 0.));
        }
        for (r, color, alpha, inner) in circles {
            let r = f64::from(r.max(0.));
            if r <= 0. {
                continue;
            }
            let inner = f64::from(inner).clamp(0., r);
            for n in 0..128 {
                let t = f64::from(n) / 128. * std::f64::consts::TAU;
                let u = f64::from(n + 1) / 128. * std::f64::consts::TAU;
                let p =
                    |r: f64, t: f64, y: f64| [at[0] + r * t.cos(), at[1] + y, at[2] + r * t.sin()];
                let mut transparent = color;
                transparent[3] = 0.;
                quad(
                    frame,
                    [p(inner, t, 0.), p(r, t, 0.), p(r, u, 0.), p(inner, u, 0.)],
                    [transparent, color, color, transparent],
                    i.covered,
                );
                if i.outline {
                    let mut c = rgba(i.outline_color);
                    c[3] *= f32::from(alpha) / 255.;
                    for r in [r, inner].into_iter().filter(|r| *r > 0.) {
                        quad(
                            frame,
                            [
                                p(r - 0.005, t, 0.001),
                                p(r + 0.005, t, 0.001),
                                p(r + 0.005, u, 0.001),
                                p(r - 0.005, u, 0.001),
                            ],
                            [c; 4],
                            i.covered,
                        );
                    }
                }
            }
        }
    }
}
