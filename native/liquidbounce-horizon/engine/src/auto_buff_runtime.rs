// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, inventory::InventoryActor, runtime::LiquidBounceHooks};
use horizon_client::mod_api::{ClientDirectionalInputEvent, Host};
use liquidbounce_behaviors::{
    auto_buff::{self as model, BowlMode, Context, Effect, Item},
    inventory_manager::{
        Action, Context as InventoryContext, Input, Outcome, OwnerState, Request, Slot,
    },
    no_rotate_set::{RotationsSettings, SmoothMode},
    random,
    rotations::{DefaultRotationTarget, Rotation, modern_mouse_gcd, movement_yaw, wrap_degrees},
};
use serde_json::{Value, json};
use std::collections::VecDeque;
pub const ROTATION_PROVIDER: u64 = 17;
pub const PROCESSOR_BASE: u64 = 220;
pub const REQUESTER: u64 = 0x4155_544f_4255_4646;
pub const REFILL_OWNER: u64 = 0x4255_4646_5245_464c;
pub const STACK_OWNER: u64 = 0x4255_4646_5354_414b;
#[derive(Default)]
pub struct AutoBuffRuntime {
    pub behavior: model::AutoBuff,
    pub rotations: RotationsSettings,
    pub input: [bool; 7],
    callback: u64,
    waiting: bool,
    pub refill_pending: Option<u64>,
    pub stack_pending: Option<u64>,
    queue: VecDeque<u8>,
}
impl AutoBuffRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn record_input(&mut self, e: &ClientDirectionalInputEvent) {
        self.input = [
            e.forward, e.backward, e.left, e.right, e.jump, e.shift, e.sprint,
        ];
    }
    pub fn clear(&mut self) {
        self.behavior.clear();
        self.input = [false; 7];
        self.callback = self.callback.wrapping_add(1);
        self.waiting = false;
        self.refill_pending = None;
        self.stack_pending = None;
        self.queue.clear();
    }
    pub fn owners(&self, running: bool) -> [(u64, OwnerState); 2] {
        let s = &self.behavior.settings;
        [
            (
                STACK_OWNER,
                OwnerState {
                    running: running && s.enabled[0] && s.bowl_mode == BowlMode::Stack,
                    guards_pass: true,
                    constraints: s.stack_constraints.clone(),
                },
            ),
            (
                REFILL_OWNER,
                OwnerState {
                    running: running && s.refill,
                    guards_pass: true,
                    constraints: s.refill_constraints.clone(),
                },
            ),
        ]
    }
    fn refill_drive(
        &mut self,
        inventory: &mut InventoryActor,
        c: &InventoryContext,
        facts: &Context,
    ) {
        let Some(id) = self.refill_pending else {
            return;
        };
        while let Some(slot) = self.queue.pop_front() {
            let Slot::Player(slot) = inventory.manager.slot_of(id, Slot::Player(slot)) else {
                continue;
            };
            if model::AutoBuff::hotbar_can_receive(facts, slot) {
                inventory.manager.submit(Request::new(
                    id,
                    REFILL_OWNER,
                    20,
                    vec![Action::Click {
                        screen: None,
                        slot: Slot::Player(slot),
                        button: 0,
                        input: Input::QuickMove,
                    }],
                    c,
                    &inventory.manager,
                ));
                return;
            }
        }
        self.refill_pending = None;
        inventory.manager.end_plan(id);
    }
}
fn sample(r: [i32; 2]) -> i32 {
    random::random_i32_inclusive(r[0], r[1])
}
fn number(v: &Value, k: &str) -> f32 {
    v[k].as_f64().unwrap_or(0.) as f32
}
impl LiquidBounceHooks {
    fn auto_buff_snapshot(&self, host: &mut dyn Host, guards: bool) -> Option<Context> {
        let player_rotation = host.client_player_rotation();
        let (input, yaw, in_combat, inventory_open) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let mut r = b.rotations.lock().expect("RotationManager");
            let correction = r
                .active_target(|provider| b.rotation_provider_running(&s.registry, provider))
                .map(|t| t.movement_correction);
            (
                b.auto_buff.input,
                movement_yaw(player_rotation[0], r.current, correction),
                s.combat.in_combat(),
                b.inventory.manager.inventory_open_server_side,
            )
        };
        let v: Value = serde_json::from_str(&host.client_auto_buff_query(
            &json!({"op":"facts","guards":guards,"movementYaw":yaw,"keys":input}).to_string(),
        ))
        .ok()?;
        if !v.is_object() {
            return None;
        }
        let pitch = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.rotations
                .lock()
                .expect("RotationManager")
                .current
                .map_or(player_rotation[1], |r| r.pitch)
        };
        let mut c = Context {
            now: crate::behaviors::current_time_millis(),
            health: number(&v, "health"),
            max_health: number(&v, "maxHealth"),
            absorption: number(&v, "absorption"),
            dead: v["dead"] == true,
            creative: v["creative"] == true,
            spectator: v["spectator"] == true,
            tick_count: v["tickCount"].as_i64().unwrap_or(0) as i32,
            in_combat,
            inventory_open: inventory_open || v["inventoryOpen"] == true,
            selected: v["selected"].as_u64().unwrap_or(0) as u8,
            server_selected: v["serverSelected"].as_u64().unwrap_or(0) as u8,
            current_pitch: pitch,
            y: v["y"].as_f64().unwrap_or(0.),
            landing_y: v["landingY"].as_i64().map(|i| i as i32),
            lingering_near: v["lingeringNear"] == true,
            splash_near: v["splashNear"] == true,
            ..Default::default()
        };
        c.effects = v["effects"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|s| s.trim_start_matches("minecraft:").to_owned())
            .collect();
        c.items = v["items"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| {
                Some(Item {
                    slot: v["slot"].as_u64()? as u8,
                    id: v["id"].as_str()?.into(),
                    components: v["components"].as_str()?.into(),
                    count: v["count"].as_i64()? as i32,
                    maximum: v["maximum"].as_i64()? as i32,
                    stackable: v["stackable"] == true,
                    effects: v["effects"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect(),
                })
            })
            .collect();
        for e in v["candidates"].as_array().into_iter().flatten() {
            let Some(id) = e["id"].as_i64().map(|i| i as i32) else {
                continue;
            };
            let target_json = if e["target"].is_object() {
                e["target"].to_string()
            } else {
                host.client_targeting_entity(id)
            };
            let Some(t) = crate::targeting::snapshot(&target_json) else {
                continue;
            };
            if !t.entity.kind.living {
                continue;
            }
            let mut s = self.state();
            let info = s.target_info(id, &t);
            if s.global_targets.combat.should_be_attacked(&t.entity, info) {
                c.enemy_near = true;
                break;
            }
        }
        Some(c)
    }
    fn auto_buff_use(&self, host: &mut dyn Host, slot: u8, reset: i32, rotation: Rotation) {
        if slot != 40 {
            let _ = host.client_select_slot_silently(REQUESTER, i32::from(slot), reset);
        }
        host.client_use_item(i32::from(slot == 40), rotation.yaw, rotation.pitch);
    }
    fn auto_buff_current(&self, host: &dyn Host) -> Rotation {
        let p = host.client_player_rotation();
        self.state()
            .behaviors
            .borrow()
            .rotations
            .lock()
            .expect("RotationManager")
            .current
            .unwrap_or(Rotation::new(p[0], p[1]))
    }
    fn auto_buff_effects(&self, host: &mut dyn Host, c: &Context, effects: Vec<Effect>) {
        for e in effects {
            match e {
                Effect::Pause(ticks) => self.state().combat.pause_combat_for_at_least(ticks),
                Effect::Select { slot, ticks } => {
                    let _ = host.client_select_slot_silently(REQUESTER, i32::from(slot), ticks);
                }
                Effect::ReleaseSelection { slot, ticks } => {
                    if host.client_silent_slot_modified_by(REQUESTER) {
                        let _ = host.client_select_slot_silently(REQUESTER, i32::from(slot), ticks);
                    }
                }
                Effect::Use { slot, reset } => {
                    let rotation = self.auto_buff_current(host);
                    self.auto_buff_use(host, slot, reset, rotation);
                }
                Effect::DropBowl => host.client_auto_buff_drop(),
                Effect::StackBowl(_) => {}
                Effect::SoupImmediate => {
                    if let Some(current) = self.auto_buff_snapshot(host, false) {
                        let next = self
                            .state()
                            .behaviors
                            .borrow_mut()
                            .auto_buff
                            .behavior
                            .drive(&current, true, &mut sample);
                        self.auto_buff_effects(host, &current, next);
                    }
                }
                Effect::PreparePotion => {
                    let freeze = {
                        let s = self.state();
                        s.behaviors.borrow().query_running(&s.registry, "Freeze")
                    };
                    let pitch = random::with_thread_random(|r| r.next_f32_between(85., 90.));
                    let Ok(v) = serde_json::from_str::<Value>(&host.client_auto_buff_query(
                        &json!({"op":"trajectory","pitch":pitch,"freeze":freeze}).to_string(),
                    )) else {
                        continue;
                    };
                    let (Some(yaw), Some(pitch)) = (v[0].as_f64(), v[1].as_f64()) else {
                        continue;
                    };
                    let next = self
                        .state()
                        .behaviors
                        .borrow_mut()
                        .auto_buff
                        .behavior
                        .potion_prepared(c, Rotation::new(yaw as f32, pitch as f32), &mut sample);
                    self.auto_buff_effects(host, c, next);
                }
                Effect::Rotate(rotation) => {
                    let s = self.state();
                    let b = s.behaviors.borrow();
                    let r = &b.auto_buff.rotations;
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
                Effect::UsePotion {
                    slot,
                    rotation,
                    pre_packet,
                    post_packet,
                } => {
                    let Ok(rotation) = rotation.normalize(
                        self.auto_buff_current(host),
                        modern_mouse_gcd(host.client_mouse_sensitivity()),
                    ) else {
                        continue;
                    };
                    if pre_packet {
                        host.client_auto_buff_packet(rotation.yaw, rotation.pitch);
                    }
                    self.auto_buff_use(host, slot, 1, rotation);
                    if post_packet {
                        let player = host.client_player_rotation();
                        let r = self.auto_buff_current(host);
                        host.client_auto_buff_packet(
                            r.yaw + wrap_degrees(player[0] - r.yaw),
                            r.pitch,
                        );
                    }
                }
            }
        }
    }
    fn auto_buff_register(&self) {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        if b.auto_buff.behavior.needs_wait() && !b.auto_buff.waiting {
            b.auto_buff.waiting = true;
            b.auto_buff.callback = b.auto_buff.callback.wrapping_add(1);
            b.coroutine_ticker
                .register(WaitOwner::AutoBuff, b.auto_buff.callback);
        }
    }
    pub(crate) fn auto_buff_first_tick(&self) {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        if b.query_running(&s.registry, "AutoBuff") {
            b.auto_buff.behavior.first_tick();
        }
    }
    pub(crate) fn auto_buff_tick(&self, host: &mut dyn Host) {
        let running = {
            let s = self.state();
            s.behaviors.borrow().query_running(&s.registry, "AutoBuff")
        };
        if !running {
            return;
        }
        let Some(c) = self.auto_buff_snapshot(host, true) else {
            return;
        };
        let e = self
            .state()
            .behaviors
            .borrow_mut()
            .auto_buff
            .behavior
            .start(&c, &mut sample);
        self.auto_buff_effects(host, &c, e);
        self.auto_buff_register();
    }
    pub(crate) fn auto_buff_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let valid = {
            let s = self.state();
            let b = s.behaviors.borrow();
            in_game
                && b.query_running(&s.registry, "AutoBuff")
                && b.auto_buff.waiting
                && b.auto_buff.callback == id
        };
        if !valid {
            return false;
        }
        let Some(c) = self.auto_buff_snapshot(host, true) else {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            b.auto_buff.waiting = false;
            b.auto_buff.behavior.run = None;
            b.auto_buff.behavior.force_use = None;
            return false;
        };
        let (before, e) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let before = b
                .auto_buff
                .behavior
                .run
                .map(|r| std::mem::discriminant(&r.phase));
            let e = b.auto_buff.behavior.drive(&c, true, &mut sample);
            (before, e)
        };
        self.auto_buff_effects(host, &c, e);
        let keep = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let after = b
                .auto_buff
                .behavior
                .run
                .map(|r| std::mem::discriminant(&r.phase));
            let keep = b.auto_buff.behavior.needs_wait() && before == after;
            b.auto_buff.waiting = keep;
            keep
        };
        if !keep {
            self.auto_buff_register();
        }
        keep
    }
    pub(crate) fn auto_buff_use_key(&self, pressed: &mut bool) {
        let s = self.state();
        let b = s.behaviors.borrow();
        b.auto_buff
            .behavior
            .force_use_key(b.query_running(&s.registry, "AutoBuff"), pressed);
    }
    pub(crate) fn auto_buff_plan(&self, host: &mut dyn Host, c: &InventoryContext) {
        let running = {
            let s = self.state();
            s.behaviors.borrow().query_running(&s.registry, "AutoBuff")
        };
        if !running {
            return;
        }
        let Some(facts) = self.auto_buff_snapshot(host, false) else {
            return;
        };
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let b = &mut *b;
        if b.auto_buff.stack_pending.is_none()
            && b.auto_buff.behavior.settings.enabled[0]
            && b.auto_buff.behavior.settings.bowl_mode == BowlMode::Stack
        {
            if let Some(slot) = b.auto_buff.behavior.stack_bowl_valid(&facts) {
                let id = b.inventory.id();
                b.auto_buff.stack_pending = Some(id);
                b.inventory.manager.submit(Request::new(
                    id,
                    STACK_OWNER,
                    30,
                    vec![Action::Click {
                        screen: None,
                        slot: Slot::Player(slot),
                        button: 0,
                        input: Input::QuickMove,
                    }],
                    c,
                    &b.inventory.manager,
                ));
            } else {
                b.auto_buff.behavior.bowl_slot = None;
            }
        }
        if b.auto_buff.refill_pending.is_none() && b.auto_buff.behavior.settings.refill {
            let slots = b.auto_buff.behavior.refill_slots(&facts);
            if !slots.is_empty() {
                b.auto_buff.refill_pending = Some(b.inventory.id());
                b.auto_buff.queue = slots.into();
                b.auto_buff.refill_drive(&mut b.inventory, c, &facts);
            }
        }
    }
    pub(crate) fn auto_buff_outcome(
        &self,
        host: &mut dyn Host,
        outcome: &Outcome,
        c: &InventoryContext,
    ) {
        let (id, completed) = match *outcome {
            Outcome::Completed { request } => (request, true),
            Outcome::Cancelled { request } => (request, false),
            _ => return,
        };
        let pending = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.auto_buff.refill_pending == Some(id) || b.auto_buff.stack_pending == Some(id)
        };
        if !pending {
            return;
        }
        let facts = if completed {
            self.auto_buff_snapshot(host, false)
        } else {
            None
        };
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let b = &mut *b;
        if b.auto_buff.stack_pending == Some(id) {
            b.auto_buff.stack_pending = None;
            b.auto_buff.behavior.bowl_slot = None;
            b.inventory.manager.end_plan(id);
        }
        if b.auto_buff.refill_pending == Some(id) {
            if let Some(facts) = facts {
                b.auto_buff.refill_drive(&mut b.inventory, c, &facts);
            } else {
                b.auto_buff.refill_pending = None;
                b.auto_buff.queue.clear();
                b.inventory.manager.end_plan(id);
            }
        }
    }
}
