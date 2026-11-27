// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396, Copyright (c) 2015-2026 CCBlueX.
//! TriggerBot, AutoClicker and their shared CrosshairAttacker continuation.
use crate::clicking::{Clicker, Cooldown, JavaRandom, Math, Settings as ClickSettings};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Id {
    Trigger,
    Attack,
    Use,
}
/// Event-listener construction order (separate from the running-state indices).
pub const IDS: [Id; 3] = [Id::Attack, Id::Use, Id::Trigger];
impl Id {
    pub fn attack(self) -> bool {
        self != Self::Use
    }
    pub fn module(self) -> &'static str {
        if self == Self::Trigger {
            "TriggerBot"
        } else {
            "AutoClicker"
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnItemUse {
    #[default]
    Wait,
    Stop,
    Ignore,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Criticals {
    #[default]
    Smart,
    Ignore,
    Always,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Objective {
    Enemy,
    Entity,
    Block,
    #[default]
    Any,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attacker {
    pub on_item_use: OnItemUse,
    pub weapon: u16,
    pub criticals: Criticals,
    pub delay_post_stop_use: i32,
}
impl Default for Attacker {
    fn default() -> Self {
        Self {
            on_item_use: OnItemUse::Wait,
            weapon: 1,
            criticals: Criticals::Smart,
            delay_post_stop_use: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Hit {
    Entity(i32),
    Block,
    #[default]
    Miss,
}
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub hit: Hit,
    pub enemy: bool,
    pub physical_attack: bool,
    pub physical_use: bool,
    /// ANY and actual WeaponType predicates on the player's main-hand stack.
    pub weapon: u16,
    pub using: bool,
    pub destroying: bool,
    pub main_item: String,
    pub off_item: String,
    pub main_named: bool,
    pub main_block_item: bool,
    pub off_block_item: bool,
    pub hit_block: Option<String>,
}
#[derive(Clone, Copy, Debug)]
pub struct CriticalState {
    pub cooldown: Cooldown,
    pub allows_ignoring_ground: bool,
    pub on_ground: bool,
    pub jumping: bool,
    pub fall_flying: bool,
    pub velocity_y: f64,
    pub fall_distance: f64,
    pub sprinting: bool,
    pub legacy: bool,
    /// Actual CriticalsJump setting; default 0.42f while that module is absent.
    pub jump_height: f32,
    pub blink_in_state: bool,
}
#[derive(Clone, Debug)]
pub struct TriggerBot {
    pub attacker: Attacker,
    pub clicker: Clicker,
    wait: Option<Wait>,
}
impl Default for TriggerBot {
    fn default() -> Self {
        Self {
            attacker: Default::default(),
            clicker: Default::default(),
            wait: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct AutoClicker {
    pub attack_enabled: bool,
    pub attack: Clicker,
    pub attacker: Attacker,
    pub delay_on_broken: bool,
    pub objective: Objective,
    pub use_enabled: bool,
    pub use_key: Clicker,
    pub holding_items_for_ignore: BTreeSet<String>,
    pub blocks_for_ignore: BTreeSet<String>,
    pub delay_start: bool,
    pub only_block: bool,
    pub requires_no_input: bool,
    pub need_to_wait: bool,
    pub last_finish_break: i64,
    wait: Option<Wait>,
}
impl Default for AutoClicker {
    fn default() -> Self {
        Self {
            attack_enabled: true,
            attack: Default::default(),
            attacker: Default::default(),
            delay_on_broken: true,
            objective: Objective::Any,
            use_enabled: false,
            use_key: Clicker::new(
                ClickSettings::use_key(),
                false,
                JavaRandom::default(),
                Math::default(),
            ),
            holding_items_for_ignore: [
                "water_bucket",
                "lava_bucket",
                "ender_pearl",
                "ender_eye",
                "player_head",
            ]
            .map(|s| format!("minecraft:{s}"))
            .into(),
            blocks_for_ignore: BTreeSet::new(),
            delay_start: false,
            only_block: false,
            requires_no_input: false,
            need_to_wait: true,
            last_finish_break: 0,
            wait: None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum Stage {
    WaitUse,
    PostUse(i32),
    DelayStart(i32),
}
#[derive(Clone, Copy, Debug)]
struct Wait {
    id: u64,
    stage: Stage,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub trigger: TriggerBot,
    pub auto: AutoClicker,
    pub mouse_states: [bool; 32],
    pub initialized: bool,
    pub blocks_configured: bool,
    next_wait: u64,
}
impl State {
    pub fn new(math: Math, ignored_blocks: BTreeSet<String>) -> Self {
        let mut state = Self::default();
        state.initialize(math, ignored_blocks);
        state
    }
    pub fn initialize(&mut self, math: Math, ignored_blocks: BTreeSet<String>) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        if !self.blocks_configured {
            self.auto.blocks_for_ignore = ignored_blocks;
        }
        for id in IDS {
            self.clicker(id).plan.set_math(math);
        }
    }
    pub fn clicker(&mut self, id: Id) -> &mut Clicker {
        match id {
            Id::Trigger => &mut self.trigger.clicker,
            Id::Attack => &mut self.auto.attack,
            Id::Use => &mut self.auto.use_key,
        }
    }
    fn attacker(&self, id: Id) -> Attacker {
        if id == Id::Trigger {
            self.trigger.attacker
        } else {
            self.auto.attacker
        }
    }
    fn wait(&self, id: Id) -> Option<Wait> {
        if id == Id::Trigger {
            self.trigger.wait
        } else {
            self.auto.wait
        }
    }
    fn wait_slot(&mut self, id: Id) -> &mut Option<Wait> {
        if id == Id::Trigger {
            &mut self.trigger.wait
        } else {
            &mut self.auto.wait
        }
    }
    fn waiting(&mut self, id: Id, stage: Stage) -> u64 {
        self.next_wait = self
            .next_wait
            .checked_add(1)
            .expect("clicker continuation exhausted");
        let ticket = self.next_wait;
        *self.wait_slot(id) = Some(Wait { id: ticket, stage });
        ticket
    }
    pub fn cancel(&mut self, id: Id) {
        *self.wait_slot(id) = None;
    }
    pub fn mouse_event(&mut self, button: i32, pressed: bool) {
        if let Ok(index) = usize::try_from(button) {
            if let Some(value) = self.mouse_states.get_mut(index) {
                *value = pressed;
            }
        }
    }
    pub fn attack_key_down(&self, original: bool, running: [bool; 3]) -> bool {
        // Clicker listeners follow source construction order: AutoClicker then TriggerBot.
        let mut result = original;
        if running[1] {
            result = self.auto.attack.attack_key_down(result);
        }
        if running[0] {
            result = self.trigger.clicker.attack_key_down(result);
        }
        result
    }
}

pub trait Host {
    fn with_state<T>(&mut self, f: impl FnOnce(&mut State) -> T) -> T;
    /// TriggerBot; AutoClicker.Attack; AutoClicker.Use listener running states.
    fn running(&mut self) -> [bool; 3];
    fn auto_running(&mut self) -> bool;
    fn monotonic_millis(&mut self) -> i64;
    fn wall_millis(&mut self) -> i64;
    fn context(&mut self) -> Option<Context>;
    fn critical_state(&mut self) -> Option<CriticalState>;
    /// Genuine isolated local-player simulation; tick zero is the next movement.
    fn falling_collision(&mut self, ticks: i32, add_velocity_y: f64)
    -> Result<Option<i32>, String>;
    fn auto_weapon_target(&mut self, target: i32);
    fn release_using(&mut self);
    fn key_click(&mut self, id: Id);
    /// Notify running clickers outside this crosshair family after success.
    fn click_success(&mut self,_id:Id) {}
    fn cooldown_and_miss(&mut self) -> Option<(Cooldown, i32)>;
    fn sample_cooldown(&mut self, range: [f32; 2]) -> f32;
    fn register_wait(&mut self, id: Id, ticket: u64);
}
fn running_index(id: Id) -> usize {
    match id {
        Id::Trigger => 0,
        Id::Attack => 1,
        Id::Use => 2,
    }
}
pub fn is_attacking(id: Id, attacker: Attacker, objective: Objective, c: &Context) -> bool {
    let wants = if id == Id::Trigger {
        matches!(c.hit, Hit::Entity(_)) && c.enemy
    } else {
        c.physical_attack
            && match objective {
                Objective::Enemy => matches!(c.hit, Hit::Entity(_)) && c.enemy,
                Objective::Entity => matches!(c.hit, Hit::Entity(_)),
                Objective::Block => c.hit == Hit::Block,
                Objective::Any => true,
            }
    };
    wants && attacker.weapon & c.weapon != 0
}
pub fn first_tick(host: &mut impl Host) {
    for id in IDS {
        if host.running()[running_index(id)] {
            let now = host.monotonic_millis();
            host.with_state(|s| s.clicker(id).tick(now));
        }
    }
}
pub fn player_press(
    host: &mut impl Host,
    attack_mapping: bool,
    use_mapping: bool,
    screen_open: bool,
    pressed: bool,
) {
    if !pressed || screen_open {
        return;
    }
    let running = host.running();
    host.with_state(|s| {
        for id in IDS {
            if running[running_index(id)]
                && if id.attack() {
                    attack_mapping
                } else {
                    use_mapping
                }
            {
                s.clicker(id).plan.pressed_by_player();
            }
        }
    });
}
fn click(host: &mut impl Host, id: Id) {
    let mut clicks: i32 = 0;
    loop {
        let Some((cooldown, _)) = host.cooldown_and_miss() else {
            break;
        };
        if !host.with_state(|s| s.clicker(id).take_attempt(cooldown)) {
            break;
        }
        let Some((cooldown, miss)) = host.cooldown_and_miss() else {
            break;
        };
        if !host.with_state(|s| s.clicker(id).passes_gate(cooldown, miss)) {
            continue;
        }
        host.key_click(id);
        let range = host.with_state(|s| s.clicker(id).settings.item_cooldown);
        let sampled = range.map(|range| host.sample_cooldown(range));
        let running = host.running();
        let now = host.monotonic_millis();
        host.with_state(|s| {
            s.clicker(id).record_success(sampled);
            let leader = s.clicker(id).plan.clone();
            for other in IDS {
                if other != id && other.attack() == id.attack() && running[running_index(other)] {
                    s.clicker(other).plan.pressed_by(&leader, now);
                }
            }
        });
        host.click_success(id);
        clicks = clicks.wrapping_add(1);
    }
    host.with_state(|s| s.clicker(id).finish_click(clicks));
}
pub fn critical_ready(host: &mut impl Host, mode: Criticals) -> bool {
    if mode == Criticals::Ignore {
        return true;
    }
    let Some(c) = host.critical_state() else {
        return false;
    };
    if mode == Criticals::Always {
        return c.allows_ignoring_ground
            && !c.on_ground
            && c.cooldown.progress(0) > 0.9
            && (!c.sprinting || c.legacy)
            && c.fall_distance > 0.0;
    }
    if c.blink_in_state || c.fall_flying || !c.allows_ignoring_ground || c.on_ground && !c.jumping {
        return true;
    }
    let duration = c.cooldown.attack_delay * 0.9 - 0.5;
    let next =
        ((duration - c.cooldown.ticks(0) as f32) / c.cooldown.server_tick_speed as f32).max(0.0);
    if !c.on_ground && c.velocity_y <= 0.0 {
        if c.fall_distance > 0.0 && c.cooldown.progress(0) > 0.9 {
            return true;
        }
        return match host.falling_collision((next + 1.0) as i32, 0.0) {
            Ok(hit) => hit.is_some_and(|tick| tick < (next as i32)),
            Err(_) => false,
        };
    }
    let initial = if c.on_ground {
        f64::from(c.jump_height)
    } else {
        c.velocity_y
    };
    let until_fall = (initial / 0.08) as f32;
    let until_crit = next.max(until_fall);
    match host.falling_collision(
        (until_crit + 5.0) as i32,
        if c.on_ground { initial } else { 0.0 },
    ) {
        Ok(hit) => hit.is_some_and(|tick| tick < (until_fall as i32)),
        Err(_) => false,
    }
}
fn attack(host: &mut impl Host, id: Id) -> bool {
    let Some(c) = host.context() else {
        return true;
    };
    if let Hit::Entity(target) = c.hit {
        host.auto_weapon_target(target);
        let mode = host.with_state(|s| s.attacker(id).criticals);
        if !critical_ready(host, mode) {
            return true;
        }
    }
    let using = host.context().is_some_and(|c| c.using);
    if using {
        let settings = host.with_state(|s| s.attacker(id));
        match settings.on_item_use {
            OnItemUse::Wait => {
                let ticket = host.with_state(|s| s.waiting(id, Stage::WaitUse));
                host.register_wait(id, ticket);
                return false;
            }
            OnItemUse::Stop => {
                host.release_using();
                let delay = host.with_state(|s| s.attacker(id).delay_post_stop_use);
                if delay != 0 {
                    let ticket = host.with_state(|s| s.waiting(id, Stage::PostUse(delay)));
                    host.register_wait(id, ticket);
                }
                return false;
            }
            OnItemUse::Ignore => {}
        }
    }
    click(host, id);
    true
}
fn use_tick(host: &mut impl Host) {
    if !host.with_state(|s| s.auto.use_enabled) {
        return;
    }
    let Some(c) = host.context() else {
        return;
    };
    if !c.physical_use && !host.with_state(|s| s.auto.requires_no_input) {
        host.with_state(|s| s.auto.need_to_wait = true);
        return;
    }
    const SPECIAL: [&str; 9] = [
        "red_bed",
        "player_head",
        "compass",
        "emerald",
        "lapis_lazuli",
        "green_dye",
        "gray_dye",
        "pink_dye",
        "slime_ball",
    ];
    if c.main_named
        && SPECIAL
            .iter()
            .any(|s| c.main_item.strip_prefix("minecraft:") == Some(*s))
    {
        return;
    }
    let skip = host.with_state(|s| {
        s.auto.holding_items_for_ignore.contains(&c.main_item)
            || s.auto.holding_items_for_ignore.contains(&c.off_item)
            || s.auto.only_block && !c.main_block_item && !c.off_block_item
            || c.hit == Hit::Block
                && c.hit_block
                    .as_ref()
                    .is_some_and(|b| s.auto.blocks_for_ignore.contains(b))
    });
    if skip {
        return;
    }
    let start = host.with_state(|s| {
        if s.auto.delay_start && s.auto.need_to_wait {
            s.auto.need_to_wait = false;
            Some(s.waiting(Id::Use, Stage::DelayStart(2)))
        } else {
            None
        }
    });
    if let Some(ticket) = start {
        host.register_wait(Id::Use, ticket);
        return;
    }
    click(host, Id::Use);
}
pub fn tick(host: &mut impl Host) {
    // ModuleManager constructs AutoClicker before TriggerBot. Equal-priority
    // tick handlers and clicker isDown listeners retain that order.
    auto_tick(host);
    let running = host.running();
    if running[0] && !host.with_state(|s| s.trigger.wait.is_some()) {
        if let Some(c) = host.context() {
            if host
                .with_state(|s| is_attacking(Id::Trigger, s.trigger.attacker, Objective::Any, &c))
            {
                attack(host, Id::Trigger);
            }
        }
    }
}
fn auto_tick(host: &mut impl Host) {
    let running = host.running();
    if !host.auto_running() || host.with_state(|s| s.auto.wait.is_some()) {
        return;
    }
    if running[1] {
        if let Some(c) = host.context() {
            let attacking = host
                .with_state(|s| is_attacking(Id::Attack, s.auto.attacker, s.auto.objective, &c));
            let wall = host.wall_millis();
            let broken = host.with_state(|s| {
                s.auto.delay_on_broken && wall.wrapping_sub(s.auto.last_finish_break) < 300
            });
            if attacking && !c.destroying && !broken && !attack(host, Id::Attack) {
                return;
            }
        }
    }
    use_tick(host);
}
/// Resume before FIRST_PRIORITY GameTick, preserving source CoroutineTicker FIFO.
pub fn callback(host: &mut impl Host, id: Id, ticket: u64) -> bool {
    let running = if id == Id::Trigger {
        host.running()[0]
    } else {
        host.auto_running()
    };
    let Some(wait) = host.with_state(|s| s.wait(id)) else {
        return false;
    };
    if wait.id != ticket {
        return false;
    }
    if !running {
        host.with_state(|s| s.cancel(id));
        return false;
    }
    match wait.stage {
        Stage::WaitUse => {
            if host.context().is_none_or(|c| c.using) {
                return true;
            }
            let delay = host.with_state(|s| s.attacker(id).delay_post_stop_use);
            host.with_state(|s| s.cancel(id));
            if delay != 0 {
                let ticket = host.with_state(|s| s.waiting(id, Stage::PostUse(delay)));
                host.register_wait(id, ticket);
            }
        }
        Stage::PostUse(ticks) | Stage::DelayStart(ticks) => {
            let next = ticks.wrapping_sub(1);
            if next > 0 {
                host.with_state(|s| {
                    if let Some(w) = s.wait_slot(id) {
                        w.stage = match w.stage {
                            Stage::DelayStart(_) => Stage::DelayStart(next),
                            _ => Stage::PostUse(next),
                        };
                    }
                });
                return true;
            }
            host.with_state(|s| s.cancel(id));
        }
    }
    // These waits end the current tickHandler body; the next GameTick may start
    // a new body normally. No delayed click is injected by the continuation.
    false
}
pub fn stop_sprinting(host: &mut impl Host) -> bool {
    let running = host.running();
    let Some(c) = host.context() else {
        return false;
    };
    if !matches!(c.hit, Hit::Entity(_)) {
        return false;
    }
    let Some(crit) = host.critical_state() else {
        return false;
    };
    if crit.legacy || crit.on_ground {
        return false;
    }
    for id in [Id::Attack, Id::Trigger] {
        if !running[running_index(id)] {
            continue;
        }
        let applies = host.with_state(|s| {
            s.attacker(id).criticals != Criticals::Ignore
                && is_attacking(id, s.attacker(id), s.auto.objective, &c)
        });
        if applies && host.with_state(|s| s.clicker(id).amount_at(1, crit.cooldown) > 0) {
            return true;
        }
    }
    false
}
