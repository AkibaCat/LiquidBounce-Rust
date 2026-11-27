// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete AutoBuff feature selection, coroutine waits and inventory policy.
use crate::{rotations::Rotation, vomit::Constraints};
use std::collections::BTreeSet;
pub const EFFECTS: [&str; 7] = [
    "instant_health",
    "regeneration",
    "strength",
    "speed",
    "fire_resistance",
    "jump_boost",
    "water_breathing",
];
pub const EFFECT_NAMES: [&str; 7] = [
    "Health",
    "Regen",
    "Strength",
    "Speed",
    "FireResistance",
    "JumpBoost",
    "WaterBreathing",
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feature {
    Soup,
    Head,
    Pot,
    Drink,
    Gapple,
}
pub const FEATURES: [Feature; 5] = [
    Feature::Soup,
    Feature::Head,
    Feature::Pot,
    Feature::Drink,
    Feature::Gapple,
];
impl Feature {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        ["Soup", "Head", "Pot", "Drink", "Gapple"][self.index()]
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BowlMode {
    None,
    #[default]
    Drop,
    Stack,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RotationTiming {
    #[default]
    Normal,
    OnTick,
    OnUse,
}
#[derive(Clone, Debug, PartialEq)]
pub struct HealthSettings {
    pub health: i32,
    pub absorption: bool,
}
impl Default for HealthSettings {
    fn default() -> Self {
        Self {
            health: 40,
            absorption: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct PotionSettings {
    pub enabled: [bool; 7],
    pub health: [i32; 2],
}
impl Default for PotionSettings {
    fn default() -> Self {
        Self {
            enabled: [true; 7],
            health: [40; 2],
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct BowlSettings {
    pub assume_empty: bool,
    pub wait: [i32; 2],
}
impl Default for BowlSettings {
    fn default() -> Self {
        Self {
            assume_empty: true,
            wait: [1, 2],
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enabled: [bool; 5],
    pub health: [HealthSettings; 3],
    pub enchanted: bool,
    pub max_absorption: f32,
    pub head_cooldown: f32,
    pub potion: [PotionSettings; 2],
    pub ground_distance: f32,
    pub no_benefit_others: bool,
    pub lingering: bool,
    pub bowl_mode: BowlMode,
    pub bowl: [BowlSettings; 2],
    pub ignore_inventory_full: bool,
    pub stack_constraints: Constraints,
    pub auto_swap: bool,
    pub delay_in: [i32; 2],
    pub delay_out: [i32; 2],
    pub refill: bool,
    pub refill_constraints: Constraints,
    pub timing: RotationTiming,
    pub combat_pause: i32,
    pub not_in_combat: bool,
    pub ignore_inventory: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: [true; 5],
            health: std::array::from_fn(|_| HealthSettings::default()),
            enchanted: true,
            max_absorption: 1.,
            head_cooldown: 0.,
            potion: std::array::from_fn(|_| PotionSettings::default()),
            ground_distance: 2.,
            no_benefit_others: true,
            lingering: false,
            bowl_mode: BowlMode::Drop,
            bowl: std::array::from_fn(|_| BowlSettings::default()),
            ignore_inventory_full: true,
            stack_constraints: Constraints::default(),
            auto_swap: true,
            delay_in: [1, 1],
            delay_out: [1, 1],
            refill: true,
            refill_constraints: Constraints::default(),
            timing: RotationTiming::Normal,
            combat_pause: 0,
            not_in_combat: false,
            ignore_inventory: false,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item {
    pub slot: u8,
    pub id: String,
    pub components: String,
    pub count: i32,
    pub maximum: i32,
    pub stackable: bool,
    pub effects: Vec<String>,
}
impl Item {
    pub fn empty(&self) -> bool {
        self.count <= 0 || self.id == "minecraft:air"
    }
}
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub now: i64,
    pub health: f32,
    pub max_health: f32,
    pub absorption: f32,
    pub dead: bool,
    pub creative: bool,
    pub spectator: bool,
    pub tick_count: i32,
    pub in_combat: bool,
    pub inventory_open: bool,
    pub selected: u8,
    pub server_selected: u8,
    pub current_pitch: f32,
    pub y: f64,
    pub landing_y: Option<i32>,
    pub enemy_near: bool,
    pub lingering_near: bool,
    pub splash_near: bool,
    pub effects: BTreeSet<String>,
    pub items: Vec<Item>,
}
impl Context {
    pub fn item(&self, slot: u8) -> Option<&Item> {
        self.items.iter().find(|s| s.slot == slot)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Select {
        slot: u8,
        ticks: i32,
    },
    Pause(i32),
    PreparePotion,
    SoupImmediate,
    Rotate(Rotation),
    Use {
        slot: u8,
        reset: i32,
    },
    UsePotion {
        slot: u8,
        rotation: Rotation,
        pre_packet: bool,
        post_packet: bool,
    },
    DropBowl,
    StackBowl(u8),
    ReleaseSelection {
        slot: u8,
        ticks: i32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    DelayIn(i32),
    Execute,
    Aim(Rotation),
    HoldUse,
    SoupWait { mode: BowlMode, ticks: i32 },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Run {
    pub feature: Feature,
    pub slot: u8,
    pub swapped: bool,
    pub phase: Phase,
}
#[derive(Clone, Debug, Default)]
pub struct AutoBuff {
    pub settings: Settings,
    pub ticks_until_next: i32,
    pub run: Option<Run>,
    pub force_use: Option<Feature>,
    pub head_last: i64,
    pub bowl_slot: Option<u8>,
}
impl AutoBuff {
    pub fn clear(&mut self) {
        self.ticks_until_next = 0;
        self.run = None;
        self.force_use = None;
        self.bowl_slot = None;
    }
    pub fn first_tick(&mut self) {
        if self.ticks_until_next > 0 {
            self.ticks_until_next -= 1;
        }
    }
    pub fn health_pass(&self, c: &Context, s: &HealthSettings) -> bool {
        c.health + if s.absorption { c.absorption } else { 0. }
            <= c.max_health * s.health as f32 / 100.
    }
    pub fn passes(&self, f: Feature, c: &Context) -> bool {
        if !self.settings.ignore_inventory && c.inventory_open {
            return false;
        }
        match f {
            Feature::Pot => {
                (!self.settings.no_benefit_others || !c.enemy_near)
                    && !c.lingering_near
                    && c.y - f64::from(c.landing_y.unwrap_or(0))
                        <= f64::from(self.settings.ground_distance)
                    && !c.splash_near
            }
            Feature::Drink => self.settings.enabled[f.index()],
            Feature::Soup => {
                self.settings.enabled[f.index()] && self.health_pass(c, &self.settings.health[0])
            }
            Feature::Gapple => {
                self.settings.enabled[f.index()] && self.health_pass(c, &self.settings.health[2])
            }
            Feature::Head => {
                self.settings.enabled[f.index()]
                    && self.health_pass(c, &self.settings.health[1])
                    && self
                        .head_last
                        .wrapping_add((self.settings.head_cooldown * 1000.) as i64)
                        < c.now
                    && c.absorption <= self.settings.max_absorption
            }
        }
    }
    pub fn valid(&self, f: Feature, s: &Item, for_use: bool, c: &Context) -> bool {
        match f {
            Feature::Soup => s.id == "minecraft:mushroom_stew",
            Feature::Head => s.id == "minecraft:player_head",
            Feature::Gapple => {
                s.id == "minecraft:golden_apple"
                    || self.settings.enchanted && s.id == "minecraft:enchanted_golden_apple"
            }
            Feature::Pot | Feature::Drink => {
                if s.empty()
                    || !if f == Feature::Pot {
                        s.id == "minecraft:splash_potion"
                            || self.settings.lingering && s.id == "minecraft:lingering_potion"
                    } else {
                        s.id == "minecraft:potion" || s.id == "minecraft:lingering_potion"
                    }
                {
                    return false;
                }
                let config = &self.settings.potion[usize::from(f == Feature::Drink)];
                let health = if for_use { c.health } else { 0. };
                s.effects.iter().any(|id| {
                    EFFECTS
                        .iter()
                        .position(|&effect| effect == id.trim_start_matches("minecraft:"))
                        .is_some_and(|i| {
                            config.enabled[i]
                                && !c.effects.contains(EFFECTS[i])
                                && (i > 1
                                    || health <= c.max_health * config.health[i] as f32 / 100.)
                        })
                })
            }
        }
    }
    pub fn usable(&self, r: Run, c: &Context) -> bool {
        c.item(r.slot)
            .is_some_and(|s| self.valid(r.feature, s, true, c))
    }
    pub fn start(&mut self, c: &Context, sample: &mut impl FnMut([i32; 2]) -> i32) -> Vec<Effect> {
        if self.run.is_some()
            || self.ticks_until_next > 0
            || self.settings.not_in_combat && c.in_combat
            || c.dead
            || c.creative
            || c.spectator
            || c.tick_count < 20
        {
            return vec![];
        }
        let mut e = vec![];
        for f in FEATURES {
            if !self.settings.enabled[f.index()] || !self.passes(f, c) {
                continue;
            }
            let key = |s: &Item| {
                if s.slot == c.server_selected {
                    i32::MIN
                } else if s.slot == 40 {
                    i32::MIN + 1
                } else {
                    (i32::from(c.server_selected) - i32::from(s.slot)).abs()
                }
            };
            let slot = c
                .items
                .iter()
                .filter(|s| (s.slot < 9 || s.slot == 40) && self.valid(f, s, true, c))
                .min_by_key(|s| key(s))
                .map(|s| s.slot);
            let Some(slot) = slot else {
                continue;
            };
            let swapped = slot != 40 && slot != c.selected;
            e.push(Effect::Pause(self.settings.combat_pause));
            if swapped && !self.settings.auto_swap {
                continue;
            }
            let delay = if swapped {
                sample(self.settings.delay_in)
            } else {
                0
            };
            self.run = Some(Run {
                feature: f,
                slot,
                swapped,
                phase: if delay == 0 {
                    Phase::Execute
                } else {
                    Phase::DelayIn(delay)
                },
            });
            if swapped {
                e.push(Effect::Select { slot, ticks: 300 });
            }
            e.extend(self.drive(c, false, sample));
            return e;
        }
        e
    }
    pub fn finish(&mut self, r: Run, sample: &mut impl FnMut([i32; 2]) -> i32) -> Vec<Effect> {
        self.run = None;
        self.force_use = None;
        if r.swapped {
            self.ticks_until_next = self
                .ticks_until_next
                .wrapping_add(sample(self.settings.delay_out));
            vec![Effect::ReleaseSelection {
                slot: r.slot,
                ticks: self.ticks_until_next,
            }]
        } else {
            vec![]
        }
    }
    pub fn drive(
        &mut self,
        c: &Context,
        advance: bool,
        sample: &mut impl FnMut([i32; 2]) -> i32,
    ) -> Vec<Effect> {
        let Some(mut r) = self.run else {
            return vec![];
        };
        match r.phase {
            Phase::DelayIn(mut ticks) => {
                if !advance {
                    return vec![];
                }
                ticks = ticks.wrapping_sub(1);
                if ticks > 0 {
                    r.phase = Phase::DelayIn(ticks);
                    self.run = Some(r);
                    return vec![];
                }
                if !self.passes(r.feature, c) || !self.usable(r, c) {
                    return self.finish(r, sample);
                }
                r.phase = Phase::Execute;
                self.run = Some(r);
                self.drive(c, false, sample)
            }
            Phase::Execute => match r.feature {
                Feature::Pot => vec![Effect::PreparePotion],
                Feature::Drink | Feature::Gapple => {
                    self.force_use = Some(r.feature);
                    r.phase = Phase::HoldUse;
                    self.run = Some(r);
                    vec![]
                }
                Feature::Head => {
                    self.head_last = c.now;
                    let mut e = vec![Effect::Use {
                        slot: r.slot,
                        reset: 1,
                    }];
                    e.extend(self.finish(r, sample));
                    e
                }
                Feature::Soup => {
                    let mode = self.settings.bowl_mode;
                    let ticks = match mode {
                        BowlMode::None => 0,
                        BowlMode::Drop => sample(self.settings.bowl[0].wait),
                        BowlMode::Stack => sample(self.settings.bowl[1].wait),
                    };
                    let mut e = vec![Effect::Use {
                        slot: r.slot,
                        reset: ticks.wrapping_add(1),
                    }];
                    if mode == BowlMode::None {
                        e.extend(self.finish(r, sample));
                    } else {
                        r.phase = Phase::SoupWait { mode, ticks };
                        self.run = Some(r);
                        if ticks == 0 {
                            e.push(Effect::SoupImmediate);
                        }
                    }
                    e
                }
            },
            Phase::HoldUse => {
                if advance && (!self.passes(r.feature, c) || !self.usable(r, c)) {
                    self.finish(r, sample)
                } else {
                    vec![]
                }
            }
            Phase::Aim(rotation) => {
                if !advance {
                    return vec![];
                }
                if !self.usable(r, c) {
                    return self.finish(r, sample);
                }
                if c.current_pitch >= rotation.pitch - 1. {
                    self.ticks_until_next = self.ticks_until_next.wrapping_add(1);
                    let mut e = vec![Effect::UsePotion {
                        slot: r.slot,
                        rotation,
                        pre_packet: false,
                        post_packet: self.settings.timing == RotationTiming::OnTick,
                    }];
                    e.extend(self.finish(r, sample));
                    e
                } else {
                    vec![]
                }
            }
            Phase::SoupWait { mode, mut ticks } => {
                if !advance {
                    return vec![];
                }
                ticks = ticks.wrapping_sub(1);
                if ticks > 0 {
                    r.phase = Phase::SoupWait { mode, ticks };
                    self.run = Some(r);
                    return vec![];
                }
                let index = usize::from(mode == BowlMode::Stack);
                let mut e = vec![];
                if r.slot != 40
                    && (self.settings.bowl[index].assume_empty
                        || c.item(r.slot).is_some_and(|s| s.id == "minecraft:bowl"))
                {
                    if mode == BowlMode::Drop {
                        e.push(Effect::DropBowl);
                    } else {
                        self.bowl_slot = Some(r.slot);
                        e.push(Effect::StackBowl(r.slot));
                    }
                }
                e.extend(self.finish(r, sample));
                e
            }
        }
    }
    pub fn potion_prepared(
        &mut self,
        c: &Context,
        rotation: Rotation,
        sample: &mut impl FnMut([i32; 2]) -> i32,
    ) -> Vec<Effect> {
        let Some(mut r) = self.run else {
            return vec![];
        };
        if r.feature != Feature::Pot {
            return vec![];
        }
        if self.settings.timing == RotationTiming::Normal {
            if !self.usable(r, c) {
                return self.finish(r, sample);
            }
            r.phase = Phase::Aim(rotation);
            self.run = Some(r);
            vec![Effect::Rotate(rotation)]
        } else {
            self.ticks_until_next = self.ticks_until_next.wrapping_add(1);
            let mut e = vec![Effect::UsePotion {
                slot: r.slot,
                rotation,
                pre_packet: self.settings.timing == RotationTiming::OnTick,
                post_packet: self.settings.timing == RotationTiming::OnTick,
            }];
            e.extend(self.finish(r, sample));
            e
        }
    }
    pub fn needs_wait(&self) -> bool {
        self.run.is_some_and(|r| !matches!(r.phase, Phase::Execute))
    }
    pub fn force_use_key(&self, running: bool, pressed: &mut bool) {
        if running && self.force_use.is_some() {
            *pressed = true;
        }
    }
    pub fn refill_slots(&self, c: &Context) -> Vec<u8> {
        c.items
            .iter()
            .filter(|s| {
                (9..36).contains(&s.slot)
                    && FEATURES
                        .iter()
                        .any(|&f| self.settings.enabled[f.index()] && self.valid(f, s, false, c))
            })
            .map(|s| s.slot)
            .collect()
    }
    pub fn hotbar_can_receive(c: &Context, slot: u8) -> bool {
        let Some(stack) = c.item(slot) else {
            return false;
        };
        (0..9).any(|i| c.item(i).is_none_or(Item::empty))
            || stack.stackable
                && (0..9)
                    .filter_map(|i| c.item(i))
                    .any(|s| !s.empty() && s.count < s.maximum && s.components == stack.components)
    }
    pub fn stack_bowl_valid(&self, c: &Context) -> Option<u8> {
        let slot = self.bowl_slot?;
        if slot == 40
            || (!self.settings.bowl[1].assume_empty
                && c.item(slot).is_none_or(|s| s.id != "minecraft:bowl"))
            || !self.settings.ignore_inventory_full
                && !(0..36).any(|i| c.item(i).is_none_or(Item::empty))
        {
            None
        } else {
            Some(slot)
        }
    }
}
