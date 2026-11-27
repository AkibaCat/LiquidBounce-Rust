// SPDX-License-Identifier: GPL-3.0-or-later
//! Crosshair clickers use genuine queued KeyMapping clicks and the shared clock.
use crate::{
    coroutine_ticker::WaitOwner,
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::{Host, ServerboundPlayPacket as Serverbound};
use liquidbounce_behaviors::{
    clicking::{Cooldown, Math},
    crosshair_clicker::{self as clicker, Context, CriticalState, Criticals, Hit, Id, State},
};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

#[derive(Default)]
pub struct BreakObserver {
    running: AtomicBool,
    last_finish: AtomicI64,
}
impl BreakObserver {
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
    }
}
impl PacketListener for BreakObserver {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if self.running.load(Ordering::Acquire)
            && matches!(event.packet, Serverbound::PlayerAction(p) if p.action == 2)
        {
            self.last_finish
                .store(crate::behaviors::current_time_millis(), Ordering::Release);
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
fn owner(id: Id) -> WaitOwner {
    match id {
        Id::Trigger => WaitOwner::TriggerBot,
        Id::Attack => WaitOwner::AutoClickerAttack,
        Id::Use => WaitOwner::AutoClickerUse,
    }
}
fn hit(v: &serde_json::Value) -> Hit {
    v["entity"]
        .as_i64()
        .map(|id| Hit::Entity(id as i32))
        .unwrap_or_else(|| {
            if v["block"].as_bool().unwrap_or(false) {
                Hit::Block
            } else {
                Hit::Miss
            }
        })
}
impl LiquidBounceHooks {
    pub(crate) fn crosshair_running(&self) -> [bool; 3] {
        let state = self.state();
        let b = state.behaviors.borrow();
        let auto = b.query_running(&state.registry, "AutoClicker");
        [
            b.query_running(&state.registry, "TriggerBot"),
            auto && b.crosshair_clicker.auto.attack_enabled,
            auto && b.crosshair_clicker.auto.use_enabled,
        ]
    }
    fn crosshair_context(&self, host: &dyn Host) -> Option<Context> {
        let v = host.client_crosshair_clicker_context()?;
        let hit = hit(&v);
        let target = if let Hit::Entity(id) = hit {
            crate::targeting::snapshot(&host.client_targeting_entity(id))
        } else {
            None
        };
        let enemy = target.as_ref().is_some_and(|target| {
            let mut state = self.state();
            let Hit::Entity(id) = hit else { return false };
            let info = state.target_info(id, target);
            state
                .global_targets
                .combat
                .should_be_attacked(&target.entity, info)
        });
        let physical = |name: &str, keyboard: &str| {
            let Some(binding) = v[name].as_str().and_then(crate::input::configured_bind) else {
                return false;
            };
            match binding {
                liquidbounce_core::InputKey::Keyboard(_) => v[keyboard].as_bool().unwrap_or(false),
                liquidbounce_core::InputKey::Mouse(button) => {
                    usize::try_from(button).ok().is_some_and(|i| {
                        self.state()
                            .behaviors
                            .borrow()
                            .crosshair_clicker
                            .mouse_states
                            .get(i)
                            .copied()
                            .unwrap_or(false)
                    })
                }
                _ => false,
            }
        };
        Some(Context {
            hit,
            enemy,
            physical_attack: physical("attackKey", "attackKeyboard"),
            physical_use: physical("useKey", "useKeyboard"),
            weapon: v["weapon"].as_u64().unwrap_or(1) as u16,
            using: v["using"].as_bool().unwrap_or(false),
            destroying: v["destroying"].as_bool().unwrap_or(false),
            main_item: v["mainItem"].as_str().unwrap_or("minecraft:air").into(),
            off_item: v["offItem"].as_str().unwrap_or("minecraft:air").into(),
            main_named: v["mainNamed"].as_bool().unwrap_or(false),
            main_block_item: v["mainBlockItem"].as_bool().unwrap_or(false),
            off_block_item: v["offBlockItem"].as_bool().unwrap_or(false),
            hit_block: v["hitBlock"].as_str().map(str::to_owned),
        })
    }
    pub(crate) fn crosshair_cooldown(&self, host: &dyn Host) -> Option<(Cooldown, serde_json::Value)> {
        let v = host.client_crosshair_clicker_critical()?;
        let clock = self.state().behaviors.borrow().combat_cooldown.clone();
        let cooldown = clock.snapshot(
            v["tickRate"].as_f64()? as f32,
            v["attackDelay"].as_f64()? as f32,
            v["hasCooldown"].as_bool()?,
        );
        Some((cooldown, v))
    }
    fn crosshair_critical(&self, host: &dyn Host) -> Option<CriticalState> {
        let (cooldown, v) = self.crosshair_cooldown(host)?;
        let (jump_height,adjust_next_jump,blink_in_state)=self.state().behaviors.borrow().criticals.selection();
        Some(CriticalState {
            cooldown,
            allows_ignoring_ground: v["allowsCritical"].as_bool()?,
            on_ground: v["onGround"].as_bool()?,
            jumping: v["jumping"].as_bool()?||adjust_next_jump,
            fall_flying: v["fallFlying"].as_bool()?,
            velocity_y: v["velocityY"].as_f64()?,
            fall_distance: v["fallDistance"].as_f64()?,
            sprinting: v["sprinting"].as_bool()?,
            legacy: v["legacy"].as_bool()?,
            jump_height,
            blink_in_state,
        })
    }
    pub(crate) fn crosshair_initialize(&self, host: &dyn Host) {
        if self
            .state()
            .behaviors
            .borrow()
            .crosshair_clicker
            .initialized
        {
            return;
        }
        let blocks = host.client_crosshair_clicker_blocks();
        self.state()
            .behaviors
            .borrow_mut()
            .crosshair_clicker
            .initialize(math(), blocks.into_iter().collect());
    }
    pub(crate) fn crosshair_first_tick(&self, host: &mut dyn Host) {
        self.crosshair_initialize(host);
        clicker::first_tick(&mut Adapter { hooks: self, host });
    }
    pub(crate) fn crosshair_tick(&self, host: &mut dyn Host) {
        {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            b.crosshair_clicker.auto.last_finish_break = b
                .crosshair_break_observer
                .last_finish
                .load(Ordering::Acquire);
        }
        clicker::tick(&mut Adapter { hooks: self, host });
    }
    pub(crate) fn crosshair_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        id: Id,
        ticket: u64,
    ) -> bool {
        if !in_game {
            self.state()
                .behaviors
                .borrow_mut()
                .crosshair_clicker
                .cancel(id);
            return false;
        }
        clicker::callback(&mut Adapter { hooks: self, host }, id, ticket)
    }
    pub(crate) fn crosshair_player_press(&self, host: &mut dyn Host, key: &str, pressed: bool) {
        let Some(v) = host.client_crosshair_clicker_context() else {
            return;
        };
        let attack = v["attackKey"].as_str() == Some(key);
        let use_key = v["useKey"].as_str() == Some(key);
        let open = host.client_state().is_none_or(|c| {
            !matches!(c.screen, horizon_client::mod_api::NativeScreenKind::Playing)
        });
        self.projectile_puncher_player_press(attack,open,pressed);
        self.kill_aura_player_press(attack,open,pressed);
        self.auto_shoot_player_press(use_key,open,pressed);
        self.scaffold_player_press(use_key,open,pressed);
        clicker::player_press(
            &mut Adapter { hooks: self, host },
            attack,
            use_key,
            open,
            pressed,
        );
    }
    pub(crate) fn crosshair_mouse(&self, button: i32, pressed: bool) {
        if let Some(button) = crate::input::mouse_button(button) {
            self.state()
                .behaviors
                .borrow_mut()
                .crosshair_clicker
                .mouse_event(button, pressed);
        }
    }
    pub(crate) fn crosshair_attack_key(&self, pressed: &mut bool) {
        let running = self.crosshair_running();
        *pressed = self
            .state()
            .behaviors
            .borrow()
            .crosshair_clicker
            .attack_key_down(*pressed, running);
    }
    /// SprintEvent INPUT / MOVEMENT_TICK, using only read Host calls.
    pub(crate) fn crosshair_stop_sprint(&self, host: &dyn Host) -> bool {
        let running = self.crosshair_running();
        let Some(c) = self.crosshair_context(host) else {
            return false;
        };
        if !matches!(c.hit, Hit::Entity(_)) {
            return false;
        }
        let Some(crit) = self.crosshair_critical(host) else {
            return false;
        };
        if crit.legacy || crit.on_ground {
            return false;
        }
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let s = &mut b.crosshair_clicker;
        for (id, index) in [(Id::Attack, 1), (Id::Trigger, 0)] {
            let attacker = if id == Id::Trigger {
                s.trigger.attacker
            } else {
                s.auto.attacker
            };
            if running[index]
                && attacker.criticals != Criticals::Ignore
                && clicker::is_attacking(id, attacker, s.auto.objective, &c)
                && s.clicker(id).amount_at(1, crit.cooldown) > 0
            {
                return true;
            }
        }
        false
    }
}
struct Adapter<'a> {
    hooks: &'a LiquidBounceHooks,
    host: &'a mut dyn Host,
}
impl clicker::Host for Adapter<'_> {
    fn with_state<T>(&mut self, f: impl FnOnce(&mut State) -> T) -> T {
        f(&mut self.hooks.state().behaviors.borrow_mut().crosshair_clicker)
    }
    fn running(&mut self) -> [bool; 3] {
        self.hooks.crosshair_running()
    }
    fn auto_running(&mut self) -> bool {
        let s = self.hooks.state();
        let b = s.behaviors.borrow();
        b.query_running(&s.registry, "AutoClicker")
    }
    fn monotonic_millis(&mut self) -> i64 {
        self.hooks.started.elapsed().as_millis() as i64
    }
    fn wall_millis(&mut self) -> i64 {
        crate::behaviors::current_time_millis()
    }
    fn context(&mut self) -> Option<Context> {
        self.hooks.crosshair_context(self.host)
    }
    fn critical_state(&mut self) -> Option<CriticalState> {
        self.hooks.crosshair_critical(self.host)
    }
    fn falling_collision(&mut self, ticks: i32, add_y: f64) -> Result<Option<i32>, String> {
        self.host.client_crosshair_clicker_falling(ticks, add_y)
    }
    fn auto_weapon_target(&mut self, target: i32) {
        use liquidbounce_behaviors::auto_weapon::{REQUESTER, Snapshot, Target};
        let weapon = self
            .host
            .client_weapon_context()
            .as_ref()
            .and_then(Snapshot::parse);
        let auto_buff_owns_slot=self.host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);
        let target = self.host.client_crosshair_clicker_target(target);
        let switch = {
            let s = self.hooks.state();
            let b = s.behaviors.borrow();
            weapon.as_ref().and_then(|weapon| {
                b.auto_weapon.on_target(
                    b.query_running(&s.registry, "AutoWeapon"),
                    &weapon.context(
                        auto_buff_owns_slot,
                        s.registry.enabled_without_world(b.query_ids["MaceKill"]),
                    ),
                    Some(Target {
                        living: target[0],
                        would_block_hit: target[1],
                    }),
                )
            })
        };
        if let Some(switch) = switch {
            if let Err(e) =
                self.host
                    .client_select_slot_silently(REQUESTER, switch.slot, switch.ticks)
            {
                eprintln!("[LiquidBounce] AutoWeapon onTarget: {e}");
            }
        }
    }
    fn release_using(&mut self) {
        self.host.client_crosshair_clicker_release();
    }
    fn click_success(&mut self,id:Id){self.hooks.projectile_puncher_other_click(id);self.hooks.kill_aura_other_click(id);self.hooks.auto_shoot_other_click(id);self.hooks.scaffold_other_click(id);}
    fn key_click(&mut self, id: Id) {
        self.host.client_crosshair_clicker_click(id.attack());
    }
    fn cooldown_and_miss(&mut self) -> Option<(Cooldown, i32)> {
        self.hooks
            .crosshair_cooldown(self.host)
            .map(|(c, v)| (c, v["missTime"].as_i64().unwrap_or(0) as i32))
    }
    fn sample_cooldown(&mut self, range: [f32; 2]) -> f32 {
        range[0] + (range[1] - range[0]) * liquidbounce_behaviors::random::random_unit_f32()
    }
    fn register_wait(&mut self, id: Id, ticket: u64) {
        self.hooks
            .state()
            .behaviors
            .borrow()
            .coroutine_ticker
            .register(owner(id), ticket);
    }
}
