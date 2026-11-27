// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned AutoMobHeal decisions. Native code supplies actual components and mobs.
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Kind {
    IronGolem,
    Wolf,
    Cat,
    HorseFamily,
    ZombieHorse,
    Llama,
    Camel,
    CamelHusk,
    Nautilus,
}
impl Kind {
    pub const ALL: [Self; 9] = [
        Self::IronGolem,
        Self::Wolf,
        Self::Cat,
        Self::HorseFamily,
        Self::ZombieHorse,
        Self::Llama,
        Self::Camel,
        Self::CamelHusk,
        Self::Nautilus,
    ];
    pub fn tag(self) -> &'static str {
        match self {
            Self::IronGolem => "IronGolem",
            Self::Wolf => "Wolf",
            Self::Cat => "Cat",
            Self::HorseFamily => "HorseFamily",
            Self::ZombieHorse => "ZombieHorse",
            Self::Llama => "Llama",
            Self::Camel => "Camel",
            Self::CamelHusk => "CamelHusk",
            Self::Nautilus => "Nautilus",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.tag() == tag)
    }
    pub fn food(self) -> bool {
        self != Self::IronGolem
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct TargetSettings {
    pub enabled: bool,
    pub health_threshold: f32,
    pub no_waste: bool,
}
impl Default for TargetSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            health_threshold: 75.,
            no_waste: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub range: f32,
    pub delay: [i32; 2],
    pub slot_reset_delay: i32,
    pub swing_mode: String,
    pub targets: [TargetSettings; 9],
    pub wolf_armor: bool,
    pub wolf_equip: bool,
    pub repair_threshold: f32,
    pub only_when_sitting: bool,
    pub allow_bucket_food: bool,
    pub allow_golden_carrot: bool,
    pub allow_golden_apple: bool,
    pub allow_enchanted_golden_apple: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            range: 4.5,
            delay: [4, 4],
            slot_reset_delay: 2,
            swing_mode: "DoNotHide".into(),
            targets: std::array::from_fn(|_| Default::default()),
            wolf_armor: false,
            wolf_equip: false,
            repair_threshold: 87.5,
            only_when_sitting: true,
            allow_bucket_food: false,
            allow_golden_carrot: false,
            allow_golden_apple: false,
            allow_enchanted_golden_apple: false,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Slot {
    /// 40 is offhand. Input order is offhand, then hotbar 0..8.
    pub index: u8,
    pub item: String,
    pub nutrition: Option<i32>,
    pub wolf_food: bool,
    pub cat_food: bool,
    /// Resolved REPAIRABLE component membership for this target's body stack.
    pub repairs_armor: bool,
}
#[derive(Clone, Debug)]
pub struct Entity {
    pub id: i32,
    pub kind: Kind,
    pub alive: bool,
    pub visible: bool,
    pub health: f32,
    pub max_health: f32,
    pub distance_squared: f64,
    pub baby: bool,
    pub tame: bool,
    pub owned: bool,
    pub sitting: bool,
    pub wearing_body_armor: bool,
    pub armor_empty: bool,
    pub armor_max_damage: i32,
    pub armor_damage: i32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub id: i32,
    pub slot: u8,
    pub distance_squared: f64,
}
#[derive(Clone, Copy)]
struct Candidate<'a> {
    slot: &'a Slot,
    heal: f32,
    bucket: bool,
}
fn nearby(index: u8, selected: u8) -> i32 {
    if index == selected {
        i32::MIN
    } else if index == 40 {
        i32::MIN + 1
    } else {
        (i32::from(selected) - i32::from(index)).abs()
    }
}
fn closest<'a>(
    slots: &'a [Slot],
    selected: u8,
    accept: impl Fn(&Slot) -> bool,
) -> Option<&'a Slot> {
    let mut best: Option<&Slot> = None;
    for slot in slots.iter().filter(|s| accept(s)) {
        if best.is_none_or(|old| nearby(slot.index, selected) < nearby(old.index, selected)) {
            best = Some(slot);
        }
    }
    best
}
fn food(settings: &Settings, kind: Kind, slot: &Slot) -> Option<(f32, bool)> {
    let item = slot.item.strip_prefix("minecraft:").unwrap_or(&slot.item);
    let nutrition = |multiplier: f32| slot.nutrition.unwrap_or(1) as f32 * multiplier;
    let heal = match kind {
        Kind::Wolf if slot.wolf_food => nutrition(2.),
        Kind::Cat if slot.cat_food => nutrition(1.),
        Kind::HorseFamily => match item {
            "sugar" => 1.,
            "wheat" => 2.,
            "apple" | "carrot" => 3.,
            "hay_block" => 20.,
            "golden_carrot" if settings.allow_golden_carrot => 4.,
            "golden_apple" if settings.allow_golden_apple => 10.,
            "enchanted_golden_apple" if settings.allow_enchanted_golden_apple => 10.,
            _ => return None,
        },
        Kind::ZombieHorse if item == "red_mushroom" => 3.,
        Kind::Llama => match item {
            "wheat" => 2.,
            "hay_block" => 10.,
            _ => return None,
        },
        Kind::Camel if item == "cactus" => 2.,
        Kind::CamelHusk if item == "rabbit_foot" => 2.,
        Kind::Nautilus => match item {
            "cod" | "salmon" | "tropical_fish" | "pufferfish" | "cooked_cod" | "cooked_salmon" => {
                nutrition(2.)
            }
            "cod_bucket" | "salmon_bucket" | "tropical_fish_bucket" | "pufferfish_bucket"
                if settings.allow_bucket_food =>
            {
                return Some((1., true));
            }
            _ => return None,
        },
        _ => return None,
    };
    Some((heal, false))
}
fn candidate_cmp(
    a: Candidate<'_>,
    b: Candidate<'_>,
    missing: f32,
    kind: Kind,
    selected: u8,
) -> Ordering {
    (kind == Kind::Nautilus && a.bucket)
        .cmp(&(kind == Kind::Nautilus && b.bucket))
        .then((a.heal > missing).cmp(&(b.heal > missing)))
        .then_with(|| {
            f64::from((a.heal - missing).abs()).total_cmp(&f64::from((b.heal - missing).abs()))
        })
        .then_with(|| nearby(a.slot.index, selected).cmp(&nearby(b.slot.index, selected)))
}
fn food_slot(settings: &Settings, entity: &Entity, slots: &[Slot], selected: u8) -> Option<u8> {
    let missing = (entity.max_health - entity.health).max(0.);
    let no_waste = settings.targets[entity.kind as usize].no_waste;
    let mut best: Option<Candidate<'_>> = None;
    for slot in slots {
        let Some((heal, bucket)) = food(settings, entity.kind, slot) else {
            continue;
        };
        if no_waste && heal > missing {
            continue;
        }
        let candidate = Candidate { slot, heal, bucket };
        if best.is_none_or(|b| candidate_cmp(candidate, b, missing, entity.kind, selected).is_lt())
        {
            best = Some(candidate);
        }
    }
    best.map(|b| b.slot.index)
}
fn health_low(settings: &Settings, e: &Entity) -> bool {
    e.alive
        && e.health
            <= e.max_health
                * (settings.targets[e.kind as usize].health_threshold / 100.).clamp(0., 1.)
}
fn armor_repair(settings: &Settings, e: &Entity) -> bool {
    settings.wolf_armor
        && e.owned
        && e.wearing_body_armor
        && (!settings.only_when_sitting || e.sitting)
        && !e.armor_empty
        && e.armor_max_damage > 0
        && (e.armor_max_damage - e.armor_damage) as f32 / e.armor_max_damage as f32
            <= (settings.repair_threshold / 100.).clamp(0., 1.)
}
fn armor_equip(settings: &Settings, e: &Entity) -> bool {
    settings.wolf_armor && settings.wolf_equip && !e.baby && e.owned && !e.wearing_body_armor
}
pub fn slot(settings: &Settings, e: &Entity, slots: &[Slot], selected: u8) -> Option<u8> {
    match e.kind {
        Kind::IronGolem => {
            closest(slots, selected, |s| s.item == "minecraft:iron_ingot").map(|s| s.index)
        }
        Kind::Wolf => {
            if armor_equip(settings, e) {
                if let Some(s) = closest(slots, selected, |s| s.item == "minecraft:wolf_armor") {
                    return Some(s.index);
                }
            }
            if health_low(settings, e) {
                if let Some(s) = food_slot(settings, e, slots, selected) {
                    return Some(s);
                }
            }
            if armor_repair(settings, e) {
                return closest(slots, selected, |s| s.repairs_armor).map(|s| s.index);
            }
            None
        }
        _ => food_slot(settings, e, slots, selected),
    }
}
pub fn plan(
    settings: &Settings,
    entities: &[Entity],
    selected: u8,
    entity_range: f64,
    secondary_use: bool,
    mut slots: impl FnMut(&Entity) -> Vec<Slot>,
) -> Option<Plan> {
    let range = f64::from(settings.range).min(entity_range);
    let max = range * range;
    let mut best: Option<Plan> = None;
    for kind in Kind::ALL {
        if !settings.targets[kind as usize].enabled {
            continue;
        }
        // Stable class order then render-entity order preserves every source tie.
        let mut target_best: Option<Plan> = None;
        for entity in entities.iter().filter(|e| e.kind == kind) {
            if !entity.visible {
                continue;
            }
            if secondary_use
                && !entity.baby
                && (matches!(kind, Kind::Camel | Kind::CamelHusk)
                    || matches!(
                        kind,
                        Kind::HorseFamily | Kind::ZombieHorse | Kind::Llama | Kind::Nautilus
                    ) && entity.tame)
            {
                continue;
            }
            let heal = health_low(settings, entity);
            if !match kind {
                Kind::Wolf => {
                    entity.tame
                        && (heal || armor_repair(settings, entity) || armor_equip(settings, entity))
                }
                Kind::Cat => entity.owned && heal,
                Kind::Nautilus => !entity.baby && entity.tame && heal,
                _ => heal,
            } {
                continue;
            }
            let distance = entity.distance_squared;
            if distance > max
                || distance >= target_best.map_or(f64::INFINITY, |p| p.distance_squared)
            {
                continue;
            }
            let Some(slot) = slot(settings, entity, &slots(entity), selected) else {
                continue;
            };
            target_best = Some(Plan {
                id: entity.id,
                slot,
                distance_squared: distance,
            });
        }
        if let Some(plan) = target_best {
            if best.is_none_or(|b| plan.distance_squared < b.distance_squared) {
                best = Some(plan);
            }
        }
    }
    best
}
