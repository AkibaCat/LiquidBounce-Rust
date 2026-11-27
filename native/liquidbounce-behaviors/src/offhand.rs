// SPDX-License-Identifier: GPL-3.0-or-later
//! LiquidBounce ae0eb396 Offhand/Totem modes, persistence and inventory plan.
use crate::vomit::Constraints;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    Totem,
    Strength,
    Gapple,
    Crystal,
    Block,
    Back,
    #[default]
    None,
}
pub const MODES: [Mode; 7] = [
    Mode::Totem,
    Mode::Strength,
    Mode::Gapple,
    Mode::Crystal,
    Mode::Block,
    Mode::Back,
    Mode::None,
];
impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Totem => "Totem",
            Self::Strength => "Strength",
            Self::Gapple => "Gapple",
            Self::Crystal => "Crystal",
            Self::Block => "Block",
            Self::Back => "Back",
            Self::None => "None",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchMode {
    Smart,
    #[default]
    Switch,
    PickUp,
    Automatic,
}
impl SwitchMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Smart => "Smart",
            Self::Switch => "Switch",
            Self::PickUp => "PickUp",
            Self::Automatic => "Automatic",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Settings {
    pub constraints: Constraints,
    pub switch_mode: SwitchMode,
    pub delay: [i32; 2],
    pub cycle: String,
    pub totem: bool,
    pub totem_delay: [i32; 2],
    pub back_delay: i32,
    pub health: bool,
    pub threshold: i32,
    pub safety: bool,
    pub safe_threshold: i32,
    pub subtract: bool,
    pub entities: bool,
    pub blocks: bool,
    pub sphere_initialized: bool,
    pub fall: bool,
    pub ignore_elytra: bool,
    pub missing_armor: bool,
    pub switch_back: bool,
    pub mace: bool,
    pub mace_range: f32,
    pub mace_speed: f32,
    pub urgency: bool,
    pub min_factor: f32,
    pub urgency_range: f32,
    pub direct: bool,
    pub crystal: bool,
    pub crystal_only_ca: bool,
    pub crystal_no_totems: bool,
    pub crystal_bind: String,
    pub gapple: bool,
    pub gapple_sword: bool,
    pub gapple_only_ka: bool,
    pub gapple_bind: String,
    pub strength: bool,
    pub strength_sword: bool,
    pub strength_only_ka: bool,
    pub strength_bind: String,
    pub block: bool,
    pub block_scaffold: bool,
    pub block_eagle: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            constraints: Constraints::default(),
            switch_mode: SwitchMode::Switch,
            delay: [0, 0],
            cycle: "key.keyboard.h".into(),
            totem: true,
            totem_delay: [0, 0],
            back_delay: 40,
            health: true,
            threshold: 14,
            safety: true,
            safe_threshold: 10,
            subtract: false,
            entities: true,
            blocks: false,
            sphere_initialized: false,
            fall: true,
            ignore_elytra: false,
            missing_armor: true,
            switch_back: true,
            mace: false,
            mace_range: 4.,
            mace_speed: 0.3,
            urgency: false,
            min_factor: 0.,
            urgency_range: 6.,
            direct: false,
            crystal: true,
            crystal_only_ca: false,
            crystal_no_totems: true,
            crystal_bind: "key.keyboard.unknown".into(),
            gapple: true,
            gapple_sword: true,
            gapple_only_ka: true,
            gapple_bind: "key.keyboard.unknown".into(),
            strength: false,
            strength_sword: true,
            strength_only_ka: true,
            strength_bind: "key.keyboard.unknown".into(),
            block: false,
            block_scaffold: true,
            block_eagle: true,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub now: i64,
    pub creative: bool,
    pub spectator: bool,
    pub dead: bool,
    pub health: f32,
    pub max_health: f32,
    pub missing_armor: bool,
    pub burrowed: bool,
    pub hole: bool,
    pub fall_distance: f64,
    pub fall_flying_pose: bool,
    pub no_fall: bool,
    pub fall_damage: f32,
    pub entity_damage: Vec<f32>,
    pub block_damage: Vec<f32>,
    pub enemy_distance_squared: Vec<f64>,
    pub mace_diving: bool,
    pub sword: bool,
    pub strength_effect: bool,
    pub kill_aura: bool,
    pub crystal_aura: bool,
    pub scaffold_enabled: bool,
    pub eagle_enabled: bool,
    pub modern: bool,
    pub selected: u8,
}
impl Settings {
    pub fn predicted_damage(&self, c: &Context, allowed: f32) -> f32 {
        let mut damage = 0f32;
        if self.entities {
            for &d in &c.entity_damage {
                damage = damage.max(d);
                if damage >= allowed {
                    return damage;
                }
            }
        }
        if damage >= allowed {
            return damage;
        }
        if self.blocks && self.sphere_initialized {
            for &d in &c.block_damage {
                damage = damage.max(d);
                if damage >= allowed {
                    return damage;
                }
            }
        }
        if damage >= allowed {
            return damage;
        }
        damage
            + if self.fall
                && !c.no_fall
                && c.fall_distance > 3.
                && !(self.ignore_elytra && c.fall_flying_pose)
            {
                c.fall_damage
            } else {
                0.
            }
    }
    pub fn below_threshold(&self, c: &Context) -> bool {
        if !self.health || self.missing_armor && c.missing_armor {
            return true;
        }
        let safe = self.safety && (c.burrowed || c.hole);
        let mut allowed = c.health
            - if safe {
                self.safe_threshold
            } else {
                self.threshold
            } as f32;
        if allowed <= 0. {
            return true;
        }
        if !self.subtract {
            allowed = c.health;
        }
        self.predicted_damage(c, allowed) >= allowed
    }
    pub fn urgency_factor(&self, c: &Context) -> f32 {
        if !self.urgency {
            return 1.;
        }
        let damage = self.predicted_damage(c, c.health) / c.health;
        let enemy = c
            .enemy_distance_squared
            .iter()
            .copied()
            .reduce(|a, b| {
                if a.is_nan() || b.is_nan() {
                    f64::NAN
                } else {
                    a.min(b)
                }
            })
            .map_or(0., |d| 1. - d.sqrt() as f32 / self.urgency_range);
        // Kotlin maxOf/coerceIn propagate NaN, unlike f32::max/clamp.
        let health = 1. - c.health / c.max_health;
        let danger = if health.is_nan() || damage.is_nan() || enemy.is_nan() {
            f32::NAN
        } else {
            health.max(damage).max(enemy).clamp(0., 1.)
        };
        1. - danger * (1. - self.min_factor)
    }
    pub fn can_cycle(&self, m: Mode, c: &Context) -> bool {
        match m {
            Mode::Totem => self.totem,
            Mode::Gapple => self.gapple,
            Mode::Crystal => self.crystal && (!self.crystal_only_ca || c.crystal_aura),
            Mode::Block => self.block,
            _ => false,
        }
    }
    pub fn should_equip(&self, m: Mode, c: &Context) -> bool {
        match m {
            Mode::Totem => {
                self.totem
                    && !c.creative
                    && !c.spectator
                    && !c.dead
                    && (self.below_threshold(c) || self.mace && c.mace_diving)
            }
            Mode::Strength => {
                self.strength
                    && (!self.strength_only_ka || c.kill_aura)
                    && !c.strength_effect
                    && (c.sword || !self.strength_sword)
            }
            Mode::Gapple => {
                self.gapple && c.sword && self.gapple_sword && (!self.gapple_only_ka || c.kill_aura)
            }
            Mode::Block => {
                self.block
                    && ((self.block_eagle && c.eagle_enabled)
                        || (self.block_scaffold && c.scaffold_enabled))
            }
            _ => false,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Item {
    pub item: String,
    pub components: String,
    pub count: i32,
    pub death_protection: bool,
    pub strength: bool,
    pub valid_block: bool,
}
impl Item {
    fn primary(&self, m: Mode) -> bool {
        match m {
            Mode::Totem => self.death_protection,
            Mode::Strength => self.item == "minecraft:potion" && self.strength,
            Mode::Gapple => self.item == "minecraft:enchanted_golden_apple",
            Mode::Crystal => self.item == "minecraft:end_crystal",
            Mode::Block => self.valid_block,
            _ => false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Swap(u8),
    Pickup(u8),
    Held(u8),
    SwapOffhand,
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub mode: Mode,
    pub slot: u8,
    pub stack: Item,
    pub actions: Vec<Action>,
    pub direct: bool,
    pub smart: bool,
}
#[derive(Clone, Debug)]
pub struct State {
    pub settings: Settings,
    pub active: Mode,
    pub last_mode: Option<Mode>,
    pub last_tag: Mode,
    pub static_mode: Mode,
    pub last: Option<(String, u8)>,
    pub roll: f32,
    pub changed_at: i64,
    pub back_started: bool,
    pub back_at: i64,
    previous: [Option<Mode>; 7],
}
impl Default for State {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            active: Mode::None,
            last_mode: None,
            last_tag: Mode::None,
            static_mode: Mode::None,
            last: None,
            roll: 0.,
            changed_at: 0,
            back_started: false,
            back_at: 0,
            previous: [None; 7],
        }
    }
}
impl State {
    pub fn enable(&mut self, c: &Context) {
        self.static_mode = if self.settings.can_cycle(Mode::Crystal, c) {
            Mode::Crystal
        } else if self.settings.gapple {
            Mode::Gapple
        } else if self.settings.totem && !self.settings.health {
            Mode::Totem
        } else {
            Mode::None
        };
    }
    pub fn key(&mut self, key: &str, c: &Context) {
        let m = if key == self.settings.gapple_bind {
            Some(Mode::Gapple)
        } else if key == self.settings.crystal_bind {
            Some(Mode::Crystal)
        } else if key == self.settings.strength_bind {
            self.settings.strength.then_some(Mode::Strength)
        } else {
            None
        };
        if let Some(m) = m {
            let i = m as usize;
            if self.active == m && self.previous[i].is_some_and(|m| self.settings.can_cycle(m, c)) {
                self.static_mode = self.previous[i].take().unwrap();
            } else if self.settings.can_cycle(m, c) {
                self.previous[i] = Some(self.static_mode);
                self.static_mode = m;
            } else {
                self.previous[i] = None;
            }
            return;
        }
        // A disabled strength binding still consumes the matching key, as Kotlin when does.
        if key == self.settings.strength_bind {
            return;
        }
        if key == self.settings.cycle {
            let start = self.static_mode as usize;
            for n in 1..7 {
                let m = MODES[(start + n) % 7];
                if self.settings.can_cycle(m, c) {
                    self.static_mode = m;
                    break;
                }
            }
        }
    }
    pub fn wanted(&self, c: &Context) -> Mode {
        let m = MODES
            .into_iter()
            .find(|&m| self.settings.should_equip(m, c))
            .unwrap_or(self.static_mode);
        if m == Mode::None && self.settings.switch_back && self.last_mode == Some(Mode::Totem) {
            Mode::Back
        } else {
            m
        }
    }
    pub fn operating(&self, running: bool) -> bool {
        running && self.active != Mode::None
    }
    pub fn may_continue(&self, c: &Context, p: &Plan, source: Option<(&str, i32)>) -> bool {
        self.wanted(c) == p.mode
            || source.is_none_or(|(components, count)| {
                components != p.stack.components || count != p.stack.count
            })
    }
    pub fn slot(&self, m: Mode, items: &[Item]) -> Option<u8> {
        if m == Mode::Back {
            return self
                .last
                .as_ref()
                .filter(|(item, slot)| items.get(*slot as usize).is_some_and(|s| s.item == *item))
                .map(|(_, slot)| *slot);
        }
        let find = |fallback: bool| {
            let matches = |i: usize| {
                items.get(i).is_some_and(|s| {
                    if fallback {
                        s.item == "minecraft:golden_apple"
                    } else {
                        s.primary(m)
                    }
                })
            };
            if matches(40) {
                return Some(40);
            }
            let order: Vec<usize> = if m == Mode::Totem {
                (0..36).collect()
            } else {
                (9..36).chain(0..9).collect()
            };
            order.into_iter().find(|&i| matches(i)).map(|i| i as u8)
        };
        let found = find(false).or_else(|| if m == Mode::Gapple { find(true) } else { None });
        found.or_else(|| {
            if m == Mode::Totem && self.settings.crystal && self.settings.crystal_no_totems {
                self.slot(Mode::Crystal, items)
            } else {
                None
            }
        })
    }
    pub fn plan(&mut self, c: &Context, items: &[Item]) -> Option<Plan> {
        let mode = self.wanted(c);
        self.active = mode;
        self.last_tag = mode;
        if Some(mode) != self.last_mode && self.last_mode == Some(Mode::Totem) {
            if !self.back_started {
                self.back_at = c.now;
            }
            self.back_started = true;
            if self
                .back_at
                .wrapping_add(i64::from(self.settings.back_delay))
                >= c.now
            {
                return None;
            }
        }
        self.back_started = false;
        let range = if mode == Mode::Totem {
            self.settings.totem_delay
        } else {
            self.settings.delay
        };
        let delay = (range[0] as f32 + (range[1] - range[0]) as f32 * self.roll)
            * if mode == Mode::Totem {
                self.settings.urgency_factor(c)
            } else {
                1.
            };
        if self.changed_at.wrapping_add(delay as i64) >= c.now {
            return None;
        }
        let slot = self.slot(mode, items)?;
        if slot == 40 {
            self.last_mode = Some(mode);
            return None;
        }
        let stack = items.get(slot as usize)?.clone();
        let switch = match self.settings.switch_mode {
            SwitchMode::Automatic => {
                if c.modern {
                    SwitchMode::Switch
                } else {
                    SwitchMode::PickUp
                }
            }
            m => m,
        };
        let smart = switch == SwitchMode::Smart && slot < 9;
        let actions = if smart {
            let mut a = Vec::new();
            if c.selected != slot {
                a.push(Action::Held(slot));
            }
            a.push(Action::SwapOffhand);
            if c.selected != slot {
                a.push(Action::Held(c.selected));
            }
            a
        } else if switch == SwitchMode::Switch {
            vec![Action::Swap(slot)]
        } else {
            let mut a = vec![Action::Pickup(slot), Action::Pickup(40)];
            if items
                .get(40)
                .is_some_and(|s| s.count > 0 && s.item != "minecraft:air")
            {
                a.push(Action::Pickup(slot));
            }
            a
        };
        Some(Plan {
            mode,
            slot,
            stack,
            actions,
            direct: mode == Mode::Totem && self.settings.direct,
            smart,
        })
    }
    pub fn complete(&mut self, p: &Plan, now: i64, roll: f32) {
        self.last_mode = Some(p.mode);
        if self.settings.switch_back {
            self.last = Some((p.stack.item.clone(), p.slot));
        }
        self.changed_at = now;
        self.roll = roll;
    }
}
