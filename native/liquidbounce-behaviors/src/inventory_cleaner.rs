// SPDX-License-Identifier: GPL-3.0-or-later
//! InventoryCleaner's shared ItemCategorization, armor evaluation and ItemPacker.
//! Stable sorts and the source's lower-bound amount constraints are intentional.
use crate::{
    auto_weapon::{self, WeaponStack},
    inventory_manager::Slot,
    vomit::Constraints,
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
};

pub const CHOICES: [&str; 24] = [
    "Sword",
    "Weapon",
    "Spear",
    "Mace",
    "BestKnockback",
    "Bow",
    "Crossbow",
    "Axe",
    "Pickaxe",
    "Shovel",
    "Hoe",
    "Rod",
    "Shield",
    "Water",
    "Lava",
    "Milk",
    "Pearl",
    "Gapple",
    "Food",
    "Potion",
    "Block",
    "Throwables",
    "Ignore",
    "None",
];
pub const SLOT_ORDER: [u8; 41] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 40, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Armor,
    Sword,
    Weapon,
    Spear,
    Mace,
    Knockback,
    Bow,
    Crossbow,
    Arrow,
    Tool,
    Rod,
    Throwable,
    Shield,
    Food,
    Bucket,
    Pearl,
    Gapple,
    Totem,
    Potion,
    Block,
    Junk,
    None,
}
impl Kind {
    pub fn priority(self) -> i32 {
        match self {
            Self::Armor => 40,
            Self::Sword | Self::Spear => 35,
            Self::Weapon | Self::Mace => 30,
            Self::Tool | Self::Pearl | Self::Gapple => 20,
            _ => 0,
        }
    }
    fn one(self) -> bool {
        matches!(
            self,
            Self::Armor
                | Self::Sword
                | Self::Weapon
                | Self::Spear
                | Self::Mace
                | Self::Knockback
                | Self::Bow
                | Self::Crossbow
                | Self::Arrow
                | Self::Tool
                | Self::Rod
                | Self::Shield
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Category(pub Kind, pub i32);
pub fn choice_category(choice: usize) -> Category {
    Category(
        match choice {
            0 => Kind::Sword,
            1 => Kind::Weapon,
            2 => Kind::Spear,
            3 => Kind::Mace,
            4 => Kind::Knockback,
            5 => Kind::Bow,
            6 => Kind::Crossbow,
            7..=10 => Kind::Tool,
            11 => Kind::Rod,
            12 => Kind::Shield,
            13..=15 => Kind::Bucket,
            16 => Kind::Pearl,
            17 => Kind::Gapple,
            18 => Kind::Food,
            19 => Kind::Potion,
            20 => Kind::Block,
            21 => Kind::Throwable,
            _ => Kind::None,
        },
        match choice {
            7 => 1,
            8 => 2,
            9 => 4,
            10 => 8,
            14 => 1,
            15 => 2,
            _ => 0,
        },
    )
}
#[derive(Clone, Debug, Default)]
pub struct BlockFacts {
    pub valid: bool,
    pub favourable: bool,
    pub full: bool,
    pub solid: bool,
    pub friction: f32,
    pub jump: f32,
    pub speed: f32,
    pub hardness: f32,
}
#[derive(Clone, Debug, Default)]
pub struct Effect {
    pub id: String,
    pub amplifier: i32,
    pub duration: i32,
}
#[derive(Clone, Debug)]
pub struct Item {
    pub slot: Slot,
    pub id: String,
    pub components: String,
    pub patch: String,
    pub count: i32,
    pub maximum: i32,
    pub default_maximum: i32,
    pub stackable: bool,
    pub weapon: WeaponStack,
    pub classes: BTreeSet<String>,
    pub enchants: BTreeMap<String, i32>,
    pub enchantment_count: i32,
    pub armor: Option<u8>,
    pub defense: f32,
    pub toughness: f32,
    pub curse_mask: u8,
    pub food: Option<(i32, f32)>,
    pub block: Option<BlockFacts>,
    pub mining_speed: f32,
    pub custom_name: bool,
    pub effects: Vec<Effect>,
    pub bucket_subtype: i32,
}
impl Default for Item {
    fn default() -> Self {
        Self {
            slot: Slot::Player(0),
            id: String::new(),
            components: String::new(),
            patch: String::new(),
            count: 0,
            maximum: 1,
            default_maximum: 1,
            stackable: false,
            weapon: WeaponStack::default(),
            classes: BTreeSet::new(),
            enchants: BTreeMap::new(),
            enchantment_count: 0,
            armor: None,
            defense: 0.,
            toughness: 0.,
            curse_mask: 0,
            food: None,
            block: None,
            mining_speed: 0.,
            custom_name: false,
            effects: vec![],
            bucket_subtype: 0,
        }
    }
}
impl Item {
    pub fn empty(&self) -> bool {
        self.count <= 0
    }
    fn is(&self, class: &str) -> bool {
        self.classes.contains(class)
    }
    pub fn hotbar(&self) -> bool {
        matches!(self.slot, Slot::Player(0..=8 | 40))
    }
    pub fn equipped(&self) -> bool {
        matches!(self.slot, Slot::Player(36..=39))
    }
    pub fn enchant(&self, name: &str) -> i32 {
        *self.enchants.get(name).unwrap_or(&0)
    }
    fn weighted(&self, weights: &[(&str, f32)]) -> f32 {
        weights
            .iter()
            .fold(0., |v, (id, w)| v + self.enchant(id) as f32 * w)
    }
    pub fn same(&self, other: &Self) -> bool {
        !self.empty() && !other.empty() && self.components == other.components
    }
    pub fn matches(&self, other: &Self) -> bool {
        (self.empty() && other.empty()) || (self.count == other.count && self.same(other))
    }
    pub fn mergeable(&self, other: &Self) -> bool {
        self.same(other) && self.count.wrapping_add(other.count) <= self.maximum
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ArmorMode {
    #[default]
    Smart,
    RawDefense,
}
#[derive(Clone, Debug)]
pub struct ArmorSettings {
    pub mode: ArmorMode,
    pub durability_threshold: i32,
    pub projectile: bool,
    pub skip_cursed: u8,
}
impl Default for ArmorSettings {
    fn default() -> Self {
        Self {
            mode: ArmorMode::Smart,
            durability_threshold: i32::MIN,
            projectile: true,
            skip_cursed: 0,
        }
    }
}
impl ArmorSettings {
    pub fn allowed(&self, item: &Item) -> bool {
        item.equipped() || self.skip_cursed & item.curse_mask == 0
    }
    fn reduction(&self, item: &Item, parameters: [f32; 2]) -> f32 {
        let armor = parameters[0] + item.defense;
        if self.mode == ArmorMode::RawDefense {
            let epf = |level: i32, m: f32| {
                if level <= 0 {
                    0
                } else {
                    (f64::from((6i32.wrapping_add(level.wrapping_mul(level))) as f32 * m) / 3.)
                        .floor() as i32
                }
            };
            let protection =
                epf(item.enchant("protection"), 0.75).wrapping_add(if self.projectile {
                    epf(item.enchant("projectile_protection"), 1.5)
                } else {
                    0
                });
            1. - (1. - (armor * 0.04).min(0.8)) * (1. - (protection as f32 * 0.04).min(0.8))
        } else {
            let toughness = parameters[1] + item.toughness;
            let g = (armor - 6. / (2. + toughness / 4.)).clamp(armor * 0.2, 20.);
            1. - (1. - g / 25.)
                * (1.
                    - item.weighted(&[
                        ("protection", 1.2 * 0.04),
                        ("projectile_protection", 0.4 * 0.08),
                        ("fire_protection", 0.39 * 0.15),
                        ("blast_protection", 0.38 * 0.08),
                    ]))
        }
    }
    pub fn compare(&self, a: &Item, b: &Item, parameters: &[[f32; 2]; 4]) -> Ordering {
        // Kotlin Float.toBigDecimal uses its shortest decimal string, HALF_UP.
        let round = |v: f32| {
            let scale = if self.mode == ArmorMode::Smart {
                1000.
            } else {
                10000.
            };
            ((v.to_string().parse::<f64>().unwrap_or(f64::from(v)) * scale).round() / scale) as f32
        };
        (a.weapon.durability > self.durability_threshold)
            .cmp(&(b.weapon.durability > self.durability_threshold))
            .then_with(|| {
                auto_weapon::java_float_compare(
                    round(self.reduction(a, parameters[a.armor.unwrap() as usize])),
                    round(self.reduction(b, parameters[b.armor.unwrap() as usize])),
                )
            })
            .then_with(|| {
                if self.mode == ArmorMode::RawDefense {
                    a.enchant("protection").cmp(&b.enchant("protection"))
                } else {
                    Ordering::Equal
                }
            })
            .then_with(|| {
                auto_weapon::java_float_compare(
                    a.weighted(&[
                        ("feather_falling", 3.),
                        ("thorns", 1.),
                        ("respiration", 0.1),
                        ("aqua_affinity", 0.05),
                        ("unbreaking", 0.01),
                    ]),
                    b.weighted(&[
                        ("feather_falling", 3.),
                        ("thorns", 1.),
                        ("respiration", 0.1),
                        ("aqua_affinity", 0.05),
                        ("unbreaking", 0.01),
                    ]),
                )
            })
            .then_with(|| a.enchantment_count.cmp(&b.enchantment_count))
            .then_with(|| a.weapon.enchantable.cmp(&b.weapon.enchantable))
            .then_with(|| a.equipped().cmp(&b.equipped()))
            .then_with(|| a.hotbar().cmp(&b.hotbar()))
    }
    pub fn parameters(items: &[Item], kit: [Option<usize>; 4]) -> [[f32; 2]; 4] {
        let total = kit.iter().flatten().fold([0., 0.], |v, &i| {
            [v[0] + items[i].defense, v[1] + items[i].toughness]
        });
        kit.map(|i| {
            i.map_or(total, |i| {
                [total[0] - items[i].defense, total[1] - items[i].toughness]
            })
        })
    }
    pub fn best_with(&self, items: &[Item], parameters: &[[f32; 2]; 4]) -> [Option<usize>; 4] {
        let mut best = [None; 4];
        for (i, item) in items
            .iter()
            .enumerate()
            .filter(|(_, s)| s.armor.is_some() && self.allowed(s))
        {
            let slot = item.armor.unwrap() as usize;
            if best[slot].is_none_or(|old| self.compare(item, &items[old], parameters).is_gt()) {
                best[slot] = Some(i);
            }
        }
        best
    }
    pub fn best(&self, items: &[Item]) -> [Option<usize>; 4] {
        let mut best: [Option<usize>; 4] = [None; 4];
        for (i, item) in items
            .iter()
            .enumerate()
            .filter(|(_, s)| s.armor.is_some() && self.allowed(s))
        {
            let slot = item.armor.unwrap() as usize;
            if best[slot].is_none_or(|old| {
                auto_weapon::java_float_compare(item.toughness, items[old].toughness).is_gt()
            }) {
                best[slot] = Some(i);
            }
        }
        for _ in 0..2 {
            best = self.best_with(items, &Self::parameters(items, best));
        }
        best
    }
}
#[derive(Clone, Debug)]
pub struct InventoryCleaner {
    pub constraints: Constraints,
    pub distance: bool,
    pub max_blocks: i32,
    pub max_arrows: i32,
    pub max_throwables: i32,
    pub max_food: i32,
    pub max_water: i32,
    pub max_lava: i32,
    pub max_milk: i32,
    pub max_totems: i32,
    pub blacklist: BTreeSet<String>,
    pub throw_unknown: bool,
    pub greedy: bool,
    pub slots: [usize; 10],
}
impl Default for InventoryCleaner {
    fn default() -> Self {
        Self {
            constraints: Constraints::default(),
            distance: false,
            max_blocks: 512,
            max_arrows: 128,
            max_throwables: 64,
            max_food: 200,
            max_water: 2,
            max_lava: 2,
            max_milk: 2,
            max_totems: 37,
            blacklist: BTreeSet::new(),
            throw_unknown: false,
            greedy: true,
            slots: [12, 1, 5, 8, 7, 23, 19, 18, 20, 20],
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum FacetKind {
    Base,
    Primitive(i32),
    Weapon,
    Knockback,
    Bow,
    Crossbow,
    Arrow,
    Tool,
    Rod,
    Shield,
    Throwable,
    Food,
    Potion,
    Block,
    Armor,
    Mace,
    GodAxe,
}
#[derive(Clone, Copy, Debug)]
struct Facet {
    item: usize,
    category: Category,
    kind: FacetKind,
    keep: bool,
}
fn potion_tier(id: &str) -> i32 {
    match id {
        "instant_health" => 6,
        "regeneration" | "resistance" | "fire_resistance" | "health_boost" | "absorption" => 5,
        "speed" | "strength" | "slow_falling" | "invisibility" => 4,
        "saturation" | "water_breathing" | "jump_boost" | "haste" | "night_vision" => 3,
        "luck" => 2,
        _ => 0,
    }
}
fn good_potion(id: &str) -> bool {
    potion_tier(id) > 0
        || matches!(
            id,
            "conduit_power" | "dolphins_grace" | "hero_of_the_village"
        )
}
fn bad_potion(id: &str) -> bool {
    matches!(
        id,
        "slowness"
            | "mining_fatigue"
            | "instant_damage"
            | "nausea"
            | "blindness"
            | "hunger"
            | "weakness"
            | "poison"
            | "wither"
            | "glowing"
            | "levitation"
            | "unluck"
            | "bad_omen"
            | "darkness"
    )
}
pub struct Categorization<'a> {
    items: &'a [Item],
    legacy: bool,
    baseline: f64,
    knockback: i32,
    armor: &'a ArmorSettings,
    armor_parameters: [[f32; 2]; 4],
    future: BTreeSet<Slot>,
}
impl<'a> Categorization<'a> {
    /// ItemCategorization.Default has no available inventory or future armor kit.
    pub fn default_value(item: &Item, legacy: bool, baseline: f64, armor: &ArmorSettings) -> i32 {
        let items = std::slice::from_ref(item);
        Categorization {
            items,
            legacy,
            baseline,
            knockback: 0,
            armor,
            armor_parameters: [[0.; 2]; 4],
            future: BTreeSet::new(),
        }
        .value(0)
    }
    pub fn new(items: &'a [Item], legacy: bool, baseline: f64, armor: &'a ArmorSettings) -> Self {
        let mut result = Self {
            items,
            legacy,
            baseline,
            knockback: 0,
            armor,
            armor_parameters: ArmorSettings::parameters(items, armor.best(items)),
            future: BTreeSet::new(),
        };
        let mut best: Option<usize> = None;
        for (i, s) in items.iter().enumerate().filter(|(_, s)| {
            s.weapon.kinds
                & (auto_weapon::SWORD | auto_weapon::SPEAR | auto_weapon::MACE | auto_weapon::AXE)
                != 0
        }) {
            if best.is_none_or(|b| result.weapon_compare(s, &items[b], false).is_gt()) {
                best = Some(i);
            }
        }
        result.knockback = best.map_or(0, |i| items[i].enchant("knockback"));
        if armor.mode != ArmorMode::RawDefense {
            result.future = armor
                .best_with(items, &[[17., 6.], [14., 6.], [12., 6.], [17., 6.]])
                .into_iter()
                .flatten()
                .map(|i| items[i].slot)
                .collect();
        }
        result
    }
    fn weapon_compare(&self, a: &Item, b: &Item, mace: bool) -> Ordering {
        let secondary = |i: &Item| {
            i.weighted(&[
                ("looting", 0.05),
                ("unbreaking", 0.05),
                ("mending", 0.1),
                ("vanishing_curse", -0.1),
                ("sweeping_edge", 0.2),
                ("knockback", 0.25),
            ])
        };
        auto_weapon::java_double_compare(
            if mace {
                a.weapon.mace_score()
            } else {
                a.weapon.damage_score(self.legacy)
            },
            if mace {
                b.weapon.mace_score()
            } else {
                b.weapon.damage_score(self.legacy)
            },
        )
        .then_with(|| {
            if mace {
                Ordering::Equal
            } else {
                auto_weapon::java_float_compare(secondary(a), secondary(b))
            }
        })
        .then_with(|| {
            (a.weapon.kinds
                & if mace {
                    auto_weapon::MACE
                } else {
                    auto_weapon::SWORD
                }
                != 0)
                .cmp(
                    &(b.weapon.kinds
                        & if mace {
                            auto_weapon::MACE
                        } else {
                            auto_weapon::SWORD
                        }
                        != 0),
                )
        })
        .then_with(|| a.weapon.durability.cmp(&b.weapon.durability))
        .then_with(|| a.weapon.enchantable.cmp(&b.weapon.enchantable))
        .then_with(|| tie(a, b))
    }
    fn facets(&self, i: usize) -> Vec<Facet> {
        let s = &self.items[i];
        if s.empty() {
            return vec![];
        }
        let mut out = vec![];
        let mut add = |kind, category, keep| {
            out.push(Facet {
                item: i,
                kind,
                category,
                keep,
            })
        };
        if s.weapon.damage_score(self.legacy) > self.baseline {
            add(FacetKind::Weapon, Category(Kind::Weapon, 0), false);
        }
        if s.enchant("knockback") > self.knockback {
            add(FacetKind::Knockback, Category(Kind::Knockback, 0), false);
        }
        let mut plain = |kind, category| add(kind, category, false);
        let base = Category(Kind::None, 0);
        if s.is("BowItem") {
            plain(FacetKind::Bow, Category(Kind::Bow, 0));
        } else if s.is("CrossbowItem") {
            plain(FacetKind::Crossbow, Category(Kind::Crossbow, 0));
        } else if s.is("ArrowItem") {
            plain(FacetKind::Arrow, Category(Kind::Arrow, 0));
        } else if s.is("FishingRodItem") {
            plain(FacetKind::Rod, Category(Kind::Rod, 0));
        } else if s.id == "minecraft:shield" {
            plain(FacetKind::Shield, Category(Kind::Shield, 0));
        } else if s.is("BlockItem") {
            if s.block.as_ref().is_some_and(|b| b.valid && b.favourable) {
                plain(FacetKind::Block, Category(Kind::Block, 0));
            } else {
                plain(FacetKind::Base, base);
            }
        } else if s.id == "minecraft:milk_bucket" {
            plain(FacetKind::Primitive(0), Category(Kind::Bucket, 2));
        } else if s.is("BucketItem") {
            plain(
                FacetKind::Primitive(0),
                Category(Kind::Bucket, s.bucket_subtype),
            );
        } else if s.is("PotionItem") {
            let useful = !s.effects.is_empty() && s.effects.iter().all(|e| good_potion(&e.id));
            let junk = !s.custom_name
                && (s.effects.is_empty()
                    || (!s.is("SplashPotionItem")
                        && !s.is("LingeringPotionItem")
                        && s.effects.iter().any(|e| bad_potion(&e.id))));
            if useful {
                plain(FacetKind::Potion, Category(Kind::Potion, 0));
            } else if junk {
                plain(FacetKind::Primitive(0), Category(Kind::Junk, 0));
            } else {
                plain(FacetKind::Base, base);
            }
        } else if s.id == "minecraft:glass_bottle" {
            plain(
                if s.custom_name {
                    FacetKind::Base
                } else {
                    FacetKind::Primitive(0)
                },
                if s.custom_name {
                    base
                } else {
                    Category(Kind::Junk, 0)
                },
            );
        } else if s.is("EnderpearlItem") {
            plain(FacetKind::Primitive(0), Category(Kind::Pearl, 0));
        } else if s.id == "minecraft:totem_of_undying" {
            plain(FacetKind::Primitive(0), Category(Kind::Totem, 0));
        } else if matches!(
            s.id.as_str(),
            "minecraft:golden_apple" | "minecraft:enchanted_golden_apple"
        ) {
            plain(FacetKind::Food, Category(Kind::Food, 0));
            plain(
                FacetKind::Primitive(i32::from(s.id == "minecraft:enchanted_golden_apple")),
                Category(Kind::Gapple, 0),
            );
        } else if s.is("EggItem") || s.is("SnowballItem") || s.is("WindChargeItem") {
            plain(FacetKind::Throwable, Category(Kind::Throwable, 0));
        } else {
            let kinds = s.weapon.kinds;
            let tool = (if kinds & auto_weapon::AXE != 0 { 1 } else { 0 })
                | (if kinds & auto_weapon::PICKAXE != 0 {
                    2
                } else {
                    0
                })
                | (if kinds & auto_weapon::SHOVEL != 0 {
                    4
                } else {
                    0
                })
                | (if kinds & auto_weapon::HOE != 0 { 8 } else { 0 });
            if kinds & auto_weapon::AXE != 0 {
                let sharp = s.enchant("sharpness");
                if sharp >= 100 {
                    add(FacetKind::GodAxe, base, true);
                } else if sharp >= 5 {
                    add(FacetKind::Weapon, Category(Kind::Weapon, 0), true);
                } else {
                    plain(FacetKind::Tool, Category(Kind::Tool, tool));
                }
            } else if s.armor.is_some() && self.armor.allowed(s) {
                add(
                    FacetKind::Armor,
                    Category(Kind::Armor, s.armor.unwrap() as i32),
                    self.future.contains(&s.slot),
                );
            } else if kinds & auto_weapon::SWORD != 0 {
                plain(FacetKind::Weapon, Category(Kind::Sword, 0));
            } else if kinds & auto_weapon::SPEAR != 0 {
                plain(FacetKind::Weapon, Category(Kind::Spear, 0));
            } else if kinds & auto_weapon::MACE != 0 {
                plain(FacetKind::Mace, Category(Kind::Mace, 0));
            } else if tool != 0 {
                plain(FacetKind::Tool, Category(Kind::Tool, tool));
            } else if s.food.is_some() {
                plain(FacetKind::Food, Category(Kind::Food, 0));
            } else if s.custom_name {
                plain(FacetKind::Primitive(0), Category(Kind::None, 1));
            } else {
                plain(FacetKind::Base, base);
            }
        }
        out
    }
    pub fn value(&self, index: usize) -> i32 {
        self.facets(index)
            .into_iter()
            .map(|f| f.category.0.priority())
            .max()
            .unwrap_or(i32::MIN)
    }
    fn compare(&self, a: Facet, b: Facet) -> Ordering {
        let x = &self.items[a.item];
        let y = &self.items[b.item];
        let weight =
            |w: &[(&str, f32)]| auto_weapon::java_float_compare(x.weighted(w), y.weighted(w));
        let count = || x.count.cmp(&y.count);
        match a.kind {
            FacetKind::Base | FacetKind::GodAxe => x.hotbar().cmp(&y.hotbar()),
            FacetKind::Weapon => self.weapon_compare(x, y, false),
            FacetKind::Mace => self.weapon_compare(x, y, true),
            FacetKind::Armor => self.armor.compare(x, y, &self.armor_parameters),
            FacetKind::Primitive(w) => w
                .cmp(&if let FacetKind::Primitive(w) = b.kind {
                    w
                } else {
                    0
                })
                .then_with(count)
                .then_with(|| tie(x, y)),
            FacetKind::Arrow | FacetKind::Throwable => count().then_with(|| tie(x, y)),
            FacetKind::Bow => weight(&[
                ("power", 0.25),
                ("punch", 0.33),
                ("flame", 3.6),
                ("infinity", 4.),
                ("unbreaking", 0.1),
                ("vanishing_curse", -0.1),
                ("mending", -0.2),
            ])
            .then_with(|| tie(x, y)),
            FacetKind::Crossbow => weight(&[
                ("quick_charge", 0.2),
                ("multishot", 1.5),
                ("piercing", 1.),
                ("mending", 0.2),
                ("unbreaking", 0.1),
                ("vanishing_curse", -0.25),
            ])
            .then_with(|| tie(x, y)),
            FacetKind::Rod | FacetKind::Shield => {
                weight(&[("unbreaking", 0.4)]).then_with(|| tie(x, y))
            }
            FacetKind::Knockback => x
                .enchant("knockback")
                .cmp(&y.enchant("knockback"))
                .then_with(|| x.weapon.durability.cmp(&y.weapon.durability))
                .then_with(|| x.weapon.enchantable.cmp(&y.weapon.enchantable))
                .then_with(|| tie(x, y)),
            FacetKind::Tool => auto_weapon::java_float_compare(x.mining_speed, y.mining_speed)
                .then_with(|| weight(&[("silk_touch", 1.), ("unbreaking", 0.2), ("fortune", 0.33)]))
                .then_with(|| x.weapon.durability.cmp(&y.weapon.durability))
                .then_with(|| tie(x, y)),
            FacetKind::Food => {
                let food = |i: &Item| {
                    if i.id == "minecraft:enchanted_golden_apple" {
                        1
                    } else if i.id == "minecraft:golden_apple" {
                        0
                    } else {
                        -1
                    }
                };
                let a = x.food.unwrap_or((0, 0.));
                let b = y.food.unwrap_or((0, 0.));
                food(x)
                    .cmp(&food(y))
                    .then_with(|| {
                        auto_weapon::java_double_compare(
                            f64::from(a.1) / f64::from(a.0),
                            f64::from(b.1) / f64::from(b.0),
                        )
                    })
                    .then_with(|| a.0.cmp(&b.0))
                    .then_with(|| auto_weapon::java_float_compare(a.1, b.1))
                    .then_with(count)
                    .then_with(|| tie(x, y))
            }
            FacetKind::Potion => {
                fn sorted(i: &Item) -> Vec<&Effect> {
                    let mut v = i.effects.iter().collect::<Vec<_>>();
                    v.sort_by_key(|e| std::cmp::Reverse(potion_tier(&e.id)));
                    v
                }
                let a = sorted(x);
                let b = sorted(y);
                let tier = |i: &Item| {
                    if i.is("SplashPotionItem") {
                        6
                    } else if i.is("LingeringPotionItem") {
                        4
                    } else {
                        5
                    }
                };
                a.iter()
                    .map(|e| potion_tier(&e.id))
                    .cmp(b.iter().map(|e| potion_tier(&e.id)))
                    .then_with(|| {
                        a.iter()
                            .map(|e| e.amplifier)
                            .cmp(b.iter().map(|e| e.amplifier))
                    })
                    .then_with(|| tier(x).cmp(&tier(y)))
                    .then_with(|| {
                        a.iter()
                            .map(|e| e.duration)
                            .cmp(b.iter().map(|e| e.duration))
                    })
                    .then_with(|| tie(x, y))
            }
            FacetKind::Block => {
                let a = x.block.as_ref().unwrap();
                let b = y.block.as_ref().unwrap();
                let distance = |h: f32, neutral: bool| {
                    if neutral && (0.8..=2.).contains(&f64::from(h)) {
                        0.
                    } else {
                        (1.7 - f64::from(h)).abs()
                    }
                };
                a.favourable
                    .cmp(&b.favourable)
                    .then_with(|| a.full.cmp(&b.full))
                    .then_with(count)
                    .then_with(|| a.solid.cmp(&b.solid))
                    .then_with(|| auto_weapon::java_float_compare(a.friction, b.friction))
                    .then_with(|| {
                        auto_weapon::java_double_compare(
                            (f64::from(a.jump) - 1.).abs(),
                            (f64::from(b.jump) - 1.).abs(),
                        )
                    })
                    .then_with(|| {
                        auto_weapon::java_double_compare(
                            (f64::from(a.speed) - 1.).abs(),
                            (f64::from(b.speed) - 1.).abs(),
                        )
                    })
                    .then_with(|| {
                        auto_weapon::java_double_compare(
                            distance(b.hardness, true),
                            distance(a.hardness, true),
                        )
                    })
                    .then_with(|| x.hotbar().cmp(&y.hotbar()))
                    .then_with(|| {
                        auto_weapon::java_double_compare(
                            distance(b.hardness, false),
                            distance(a.hardness, false),
                        )
                    })
                    .then_with(|| x.weapon.identity_hash.cmp(&y.weapon.identity_hash))
            }
        }
    }
}
fn tie(a: &Item, b: &Item) -> Ordering {
    a.hotbar()
        .cmp(&b.hotbar())
        .then_with(|| a.weapon.identity_hash.cmp(&b.weapon.identity_hash))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Swap {
    pub from: Slot,
    pub to: Slot,
    pub priority: i32,
}
#[derive(Clone, Debug, Default)]
pub struct CleanupPlan {
    pub useful: BTreeSet<Slot>,
    pub swaps: Vec<Swap>,
    pub mergeable: Vec<Vec<Slot>>,
}
impl CleanupPlan {
    pub fn merge_slots(&self, items: &[Item]) -> Vec<Slot> {
        let mut result = vec![];
        for group in &self.mergeable {
            let mut stacks = group
                .iter()
                .filter_map(|slot| items.iter().find(|i| i.slot == *slot))
                .map(|i| (i.slot, i.count, i.maximum))
                .collect::<Vec<_>>();
            let Some(&(_, _, max)) = stacks.first() else {
                continue;
            };
            if max <= 0
                || stacks.len() as f64
                    <= (stacks.iter().fold(0i32, |v, s| v.wrapping_add(s.1)) as f64
                        / f64::from(max))
                    .ceil()
            {
                continue;
            }
            stacks.sort_by_key(|s| s.1);
            let mut stacks: VecDeque<_> = stacks.into();
            while stacks.len() > 1 {
                while !stacks.is_empty() && stacks.back().unwrap().1.wrapping_add(stacks[0].1) > max
                {
                    stacks.pop_back();
                }
                let Some((slot, count, _)) = stacks.pop_back() else {
                    break;
                };
                result.push(slot);
                let mut remaining = max.wrapping_sub(count);
                while remaining > 0 && !stacks.is_empty() {
                    let transferred = stacks[0].1.min(remaining);
                    if stacks[0].1 <= remaining {
                        stacks.pop_front();
                    } else {
                        stacks[0].1 = stacks[0].1.wrapping_sub(transferred);
                    }
                    remaining = remaining.wrapping_sub(transferred);
                }
            }
        }
        result
    }
}
impl InventoryCleaner {
    fn amount(&self, facet: Facet, item: &Item) -> (Category, i32, i32) {
        if matches!(facet.kind, FacetKind::Weapon | FacetKind::Mace) {
            return (Category(Kind::Weapon, -1), 1, 1);
        }
        if matches!(facet.kind, FacetKind::Food) {
            return (
                Category(Kind::Food, -1),
                self.max_food,
                item.count.wrapping_mul(item.food.unwrap_or((0, 0.)).0),
            );
        }
        let desired = match facet.category {
            Category(Kind::Block, 0) => self.max_blocks,
            Category(Kind::Arrow, 0) => self.max_arrows,
            Category(Kind::Throwable, 0) => self.max_throwables,
            Category(Kind::Bucket, 0) => self.max_water,
            Category(Kind::Bucket, 1) => self.max_lava,
            Category(Kind::Bucket, 2) => self.max_milk,
            Category(Kind::Totem, 0) => self.max_totems,
            Category(Kind::Junk, 0) => 0,
            Category(Kind::None, 0) => {
                if self.throw_unknown {
                    0
                } else {
                    i32::MAX
                }
            }
            _ => {
                if facet.category.0.one() {
                    1
                } else {
                    i32::MAX
                }
            }
        };
        (facet.category, desired, item.count)
    }
    pub fn plan(
        &self,
        items: &[Item],
        legacy: bool,
        baseline: f64,
        armor: &ArmorSettings,
        locked: &BTreeSet<Slot>,
        offhand_operating: bool,
        offhand_swap: bool,
    ) -> CleanupPlan {
        let mut forbidden = locked.clone();
        forbidden.extend((36..40).map(Slot::Player));
        let mut forbidden_fill = locked.clone();
        if offhand_operating {
            forbidden.insert(Slot::Player(40));
            forbidden_fill.insert(Slot::Player(40));
        }
        let targets = self
            .slots
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 0 || offhand_swap)
            .map(|(i, &v)| (Slot::Player(if i == 0 { 40 } else { (i - 1) as u8 }), v))
            .collect::<Vec<_>>();
        forbidden.extend(
            targets
                .iter()
                .filter(|(_, choice)| *choice == 22)
                .map(|(s, _)| *s),
        );
        let available = items
            .iter()
            .filter(|i| !i.empty() && !self.blacklist.contains(&i.id))
            .cloned()
            .collect::<Vec<_>>();
        let categorizer = Categorization::new(&available, legacy, baseline, armor);
        let mut groups: Vec<(Category, Vec<Facet>)> = vec![];
        for i in 0..available.len() {
            for facet in categorizer.facets(i) {
                if let Some((_, group)) = groups.iter_mut().find(|(c, _)| *c == facet.category) {
                    group.push(facet);
                } else {
                    groups.push((facet.category, vec![facet]));
                }
            }
        }
        groups.sort_by_key(|(c, _)| std::cmp::Reverse(c.0.priority()));
        let mut plan = CleanupPlan::default();
        let mut allocated = BTreeSet::new();
        let mut limits = BTreeMap::<Category, i32>::new();
        for (category, mut facets) in groups {
            facets.sort_by(|a, b| categorizer.compare(*b, *a));
            let target_slots = targets
                .iter()
                .filter(|(_, c)| choice_category(*c) == category && category.0 != Kind::None)
                .map(|(s, _)| *s)
                .collect::<Vec<_>>();
            let mut target = 0;
            let mut count = 0;
            for facet in &facets {
                let item = &available[facet.item];
                let (group, desired, added) = self.amount(*facet, item);
                let current = *limits.get(&group).unwrap_or(&0);
                if (count >= target_slots.len() && current >= desired)
                    || allocated.contains(&item.slot)
                {
                    continue;
                }
                plan.useful.insert(item.slot);
                limits.insert(group, current.wrapping_add(added));
                count += 1;
                if forbidden.contains(&item.slot) {
                    continue;
                }
                while let Some(&to) = target_slots.get(target) {
                    target += 1;
                    if item.slot == to {
                        allocated.insert(to);
                        break;
                    }
                    let same = items
                        .iter()
                        .find(|i| i.slot == to)
                        .is_some_and(|other| item.matches(other));
                    if same {
                        allocated.insert(to);
                        continue;
                    }
                    allocated.insert(item.slot);
                    allocated.insert(to);
                    if !forbidden_fill.contains(&to) {
                        plan.swaps.push(Swap {
                            from: item.slot,
                            to,
                            priority: category.0.priority(),
                        });
                    }
                    break;
                }
            }
            plan.useful.extend(
                facets
                    .into_iter()
                    .filter(|f| f.keep)
                    .map(|f| available[f.item].slot),
            );
        }
        plan.useful.extend(forbidden);
        let mut merge: Vec<(String, Vec<Slot>)> = vec![];
        for item in &available {
            if !item.stackable || item.count >= item.maximum {
                continue;
            }
            if let Some((_, group)) = merge
                .iter_mut()
                .find(|(identity, _)| *identity == item.patch)
            {
                group.push(item.slot);
            } else {
                merge.push((item.patch.clone(), vec![item.slot]));
            }
        }
        plan.mergeable = merge.into_iter().map(|(_, slots)| slots).collect();
        plan
    }
}
