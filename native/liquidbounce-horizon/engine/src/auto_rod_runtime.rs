// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_bow::{self, Candidate},
    auto_bow_math::{self as math, B, V},
    auto_rod::{self as model, Context, Hook, Phase},
    clicking::JavaRandom,
    no_rotate_set::{RotationsSettings, SmoothMode},
    point_tracker::{PointTracker, Samples},
    random,
    rotations::{DefaultRotationTarget, Rotation},
    target_renderer::{TargetRendererRuntime, TargetRendererSettings},
};
use liquidbounce_core::InputKey;
use serde_json::{Value, json};
pub const ROTATION_PROVIDER: u64 = 20;
pub const PROCESSOR_BASE: u64 = 260;
pub const REQUESTER: u64 = 0x4155_544f_524f_4400;
pub struct NativeSamples<'a>(pub &'a mut JavaRandom);
impl Samples for NativeSamples<'_> {
    fn float(&mut self, r: [f32; 2]) -> f32 {
        if r[0] >= r[1] {
            r[0]
        } else {
            random::with_thread_random(|random| random.next_f32_between(r[0], r[1].next_up()))
        }
    }
    fn int(&mut self, r: [i32; 2]) -> i32 {
        random::random_i32_inclusive(r[0], r[1])
    }
    fn gaussian(&mut self) -> f64 {
        self.0.gaussian(horizon_client::clicker_math::fdlibm::log)
    }
    fn chance(&mut self, p: f32) -> bool {
        if p == 0. {
            false
        } else if p == 100. {
            true
        } else {
            let unit = (self.0.double() * 16_777_216.).floor() as f32 / 16_777_216.;
            (unit * 100.).min(100f32.next_down()) < p
        }
    }
}
pub struct AutoRodRuntime {
    pub behavior: model::AutoRod,
    pub point_tracker: PointTracker,
    pub rotations: RotationsSettings,
    pub random: JavaRandom,
    pub input: model::InputTracker,
    pub target_renderer: TargetRendererSettings,
    pub target_render_runtime: TargetRendererRuntime,
    callback: u64,
    waiting: bool,
    target_position: [f64; 3],
}
impl Default for AutoRodRuntime {
    fn default() -> Self {
        let mut r = Self {
            behavior: Default::default(),
            point_tracker: Default::default(),
            rotations: Default::default(),
            random: Default::default(),
            input: Default::default(),
            target_renderer: Default::default(),
            target_render_runtime: Default::default(),
            callback: 0,
            waiting: false,
            target_position: [0.; 3],
        };
        r.behavior.initialize(&mut NativeSamples(&mut r.random));
        r.point_tracker
            .initialize(&mut NativeSamples(&mut r.random));
        r
    }
}
impl AutoRodRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn clear(&mut self) -> bool {
        self.callback = self.callback.wrapping_add(1);
        self.waiting = false;
        self.behavior.clear()
    }
}
fn query(host: &mut dyn Host, value: Value) -> Value {
    serde_json::from_str(&host.client_auto_rod_query(&value.to_string())).unwrap_or(Value::Null)
}
fn hook(v: &Value) -> Option<Hook> {
    Some(Hook {
        id: v["id"].as_i64()? as i32,
        hooked: v["hooked"] == true,
        stationary: v["stationary"] == true,
    })
}
struct Facts {
    context: Context,
    candidates: Vec<Candidate>,
    rows: Vec<Value>,
}
impl LiquidBounceHooks {
    fn auto_rod_facts(&self, host: &mut dyn Host) -> Option<Facts> {
        let value = query(host, json!({"op":"facts"}));
        if !value.is_object() {
            return None;
        }
        let now = crate::behaviors::current_time_millis();
        let click = match value["attackKey"]
            .as_str()
            .and_then(crate::input::configured_bind)
        {
            Some(InputKey::Keyboard(scan)) => host.client_keyboard_scan_down(scan),
            Some(InputKey::Mouse(button)) => self
                .state()
                .behaviors
                .borrow()
                .auto_rod
                .input
                .mouse_click(button, now),
            _ => false,
        };
        let (forbidden, inventory, moved) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.query_running(&s.registry, "Blink")
                    || b.query_running(&s.registry, "Scaffold")
                    || b.query_running(&s.registry, "Freeze"),
                b.inventory.manager.inventory_open_server_side,
                b.auto_rod.input.mouse_moved(now),
            )
        };
        let mut context = Context {
            selected: value["selected"].as_u64().unwrap_or(0) as u8,
            hook: hook(&value["hook"]),
            health: value["health"].as_f64().unwrap_or(0.) as f32,
            requires: [
                click,
                moved,
                value["weapon"] == true,
                value["vanillaName"] == true,
                value["notBreaking"] == true,
            ],
            ignore: [
                inventory || value["inventoryOpen"] == true,
                value["using"] == true,
                value["consumable"] == true,
            ],
            main_item: value["mainItem"].as_str().unwrap_or("minecraft:air").into(),
            forbidden,
            ..Default::default()
        };
        for (i, v) in value["rods"]
            .as_array()
            .into_iter()
            .flatten()
            .take(10)
            .enumerate()
        {
            context.rods[i] = v == true;
        }
        let mut rows = vec![];
        let mut candidates = vec![];
        for v in value["candidates"].as_array().into_iter().flatten() {
            let Some(id) = v["id"].as_i64().map(|i| i as i32) else {
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
            rows.push(v.clone());
        }
        let settings = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let s = &b.auto_rod.behavior.settings;
            auto_bow::Settings {
                fov: s.fov,
                hurt_time: s.hurt_time,
                priority: s.priority.clone(),
                ..Default::default()
            }
        };
        let candidates = auto_bow::targets(&settings, candidates);
        context.enemies = candidates.len() as i32;
        Some(Facts {
            context,
            candidates,
            rows,
        })
    }
    pub(crate) fn auto_rod_first_tick(&self, host: &mut dyn Host) {
        let running = {
            let s = self.state();
            s.behaviors.borrow().query_running(&s.registry, "AutoRod")
        };
        if !running {
            return;
        }
        if let Some(f) = self.auto_rod_facts(host) {
            self.state()
                .behaviors
                .borrow_mut()
                .auto_rod
                .behavior
                .first_tick(&f.context);
        }
    }
    fn auto_rod_aim(&self, host: &mut dyn Host, id: i32) -> Option<Rotation> {
        let projectile = self
            .state()
            .behaviors
            .borrow()
            .auto_rod
            .behavior
            .settings
            .projectile;
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
            let r = &mut b.auto_rod;
            let eye = V(eyes[0], eyes[1], eyes[2]);
            let point = r.point_tracker.find(
                eye,
                B::from_array(bounds),
                &mut NativeSamples(&mut r.random),
            );
            math::rotation(point.pos.sub(eye))
        };
        Some(Rotation::new(a[0], a[1]))
    }
    pub(crate) fn auto_rod_prepare(&self, host: &mut dyn Host) {
        let running = {
            let s = self.state();
            s.behaviors.borrow().query_running(&s.registry, "AutoRod")
        };
        if !running {
            return;
        }
        let Some(facts) = self.auto_rod_facts(host) else {
            return;
        };
        let id = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_rod;
            if !r.behavior.requirements_met(&facts.context) {
                r.behavior.target = None;
                return;
            }
            let (min, max) = r.behavior.ranges();
            let health = r.behavior.settings.min_target_health;
            r.behavior.target = facts
                .candidates
                .iter()
                .find(|c| {
                    c.health > health
                        && facts
                            .rows
                            .iter()
                            .find(|row| row["id"].as_i64() == Some(i64::from(c.id)))
                            .is_some_and(|row| {
                                row["visible"] == true
                                    && row["rangeSquared"]
                                        .as_f64()
                                        .is_some_and(|d| (min..=max).contains(&d))
                            })
                })
                .map(|c| c.id);
            r.behavior.target
        };
        let Some(id) = id else {
            return;
        };
        let Some(rotation) = self.auto_rod_aim(host, id) else {
            return;
        };
        let state = self.state();
        let b = state.behaviors.borrow();
        let s = &b.auto_rod.rotations;
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(
                ROTATION_PROVIDER,
                20,
                DefaultRotationTarget {
                    rotation,
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
    }
    fn auto_rod_use(&self, host: &mut dyn Host, slot: u8, reset: i32) -> bool {
        if slot != 40 {
            let _ = host.client_select_slot_silently(REQUESTER, i32::from(slot), reset);
        }
        let player = host.client_player_rotation();
        let (rotation, hide) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.rotations
                    .lock()
                    .expect("RotationManager")
                    .current
                    .unwrap_or(Rotation::new(player[0], player[1])),
                b.auto_rod.behavior.settings.hide_swing,
            )
        };
        query(
            host,
            json!({"op":"use","hand":i32::from(slot==40),"rotation":[rotation.yaw,rotation.pitch],"hideSwing":hide}),
        ) == true
    }
    fn auto_rod_register(&self) {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let r = &mut b.auto_rod;
        if r.waiting || r.behavior.run.is_none() {
            return;
        }
        r.callback = r.callback.wrapping_add(1);
        r.waiting = true;
        let id = r.callback;
        b.coroutine_ticker.register(WaitOwner::AutoRod, id);
    }
    pub(crate) fn auto_rod_tick(&self, host: &mut dyn Host) {
        let ready = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.query_running(&s.registry, "AutoRod") && b.auto_rod.behavior.run.is_none()
        };
        if !ready {
            return;
        }
        let Some(f) = self.auto_rod_facts(host) else {
            return;
        };
        let (slot, target, cached, reset, threshold, server) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = &b.auto_rod;
            if !r.behavior.requirements_met(&f.context) {
                return;
            }
            let (Some(slot), Some(target)) = (r.behavior.available_slot, r.behavior.target) else {
                return;
            };
            (
                slot,
                target,
                r.behavior.cached_hook,
                r.behavior.settings.slot_reset[1].wrapping_add(r.behavior.settings.hit_timeout),
                r.behavior.settings.aim_threshold,
                b.rotations
                    .lock()
                    .expect("RotationManager")
                    .server_rotation(
                        b.criticals.is_lagging(),
                        b.query_running(&s.registry, "Freeze"),
                    ),
            )
        };
        let Some(rotation) = self.auto_rod_aim(host, target) else {
            return;
        };
        let Some(angle)=query(host,json!({"op":"angle","rotation":[server.yaw,server.pitch],"target":[rotation.yaw,rotation.pitch]})).as_f64()else{return;};
        if angle > f64::from(threshold) {
            return;
        }
        if cached.is_none() {
            if !self.auto_rod_use(host, slot, reset) {
                return;
            }
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_rod;
            r.behavior.scan_changed(&mut NativeSamples(&mut r.random));
        }
        {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_rod;
            if let Some(row) = f
                .rows
                .iter()
                .find(|row| row["id"].as_i64() == Some(i64::from(target)))
            {
                r.target_position =
                    serde_json::from_value(row["position"].clone()).unwrap_or([0.; 3]);
            }
            r.behavior.begin(slot, target);
        }
        // tickConditional(0) resumes immediately; negative imported values still
        // register a callback like the JVM helper. UI values normally start at 5.
        if self
            .state()
            .behaviors
            .borrow()
            .auto_rod
            .behavior
            .run
            .is_some_and(|run| run.hit_timeout == 0)
        {
            self.auto_rod_pull(host, slot);
        } else {
            self.auto_rod_register();
        }
    }
    fn auto_rod_pull(&self, host: &mut dyn Host, slot: u8) {
        let reset = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_rod;
            NativeSamples(&mut r.random).int(r.behavior.settings.slot_reset)
        };
        self.auto_rod_use(host, slot, reset);
        {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_rod;
            let ticks = NativeSamples(&mut r.random).int(r.behavior.settings.cooldown);
            r.behavior.cooldown(ticks);
            r.waiting = false;
        }
        self.auto_rod_register();
    }
    pub(crate) fn auto_rod_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let run = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = &b.auto_rod;
            if !in_game
                || !b.query_running(&s.registry, "AutoRod")
                || !r.waiting
                || r.callback != id
            {
                return false;
            }
            r.behavior.run
        };
        let Some(run) = run else {
            return false;
        };
        let slot_ok = query(host, json!({"op":"slot","slot":run.slot})) == true;
        if !slot_ok {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            b.auto_rod.behavior.run = None;
            b.auto_rod.waiting = false;
            return false;
        }
        if matches!(run.phase, Phase::Cooldown { .. }) {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_rod;
            let keep = r.behavior.advance_cooldown(true);
            r.waiting = keep;
            return keep;
        }
        let current_hook = hook(&query(host, json!({"op":"hook"})));
        let position = self.state().behaviors.borrow().auto_rod.target_position;
        let distance = query(
            host,
            json!({"op":"distance","id":run.target,"position":position}),
        );
        let pull = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.auto_rod;
            // computedOn retains an entity reference between FIRST_PRIORITY updates;
            // refresh that same object's fields, without adopting a new hook early.
            if current_hook
                .is_some_and(|h| r.behavior.cached_hook.is_some_and(|old| h.id == old.id))
            {
                r.behavior.cached_hook = current_hook;
            }
            if let Ok(p) = serde_json::from_value::<[f64; 3]>(distance["position"].clone()) {
                r.target_position = p;
            }
            r.behavior
                .advance_hit(true, distance["distance"].as_f64().unwrap_or(f64::NAN))
        };
        if pull {
            self.auto_rod_pull(host, run.slot);
            false
        } else {
            true
        }
    }
    pub(crate) fn auto_rod_disabled(&self, host: &mut dyn Host, release: bool) {
        if release {
            let _ = query(host, json!({"op":"release"}));
        }
    }
}
