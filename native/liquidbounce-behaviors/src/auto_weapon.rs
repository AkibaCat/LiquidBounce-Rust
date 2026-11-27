// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoWeapon and its WeaponItemFacet chain, LiquidBounce ae0eb396.
//! Item facts are snapshots of the actual nine hotbar stack instances.
use serde_json::Value;
use std::cmp::Ordering;

pub const PREFERRED: [&str; 10] = [
    "Any",
    "Sword",
    "Axe",
    "Mace",
    "Spear",
    "Pickaxe",
    "Shovel",
    "Hoe",
    "Knockback",
    "FireAspect",
];
pub const PRIORITIES: [&str; 4] = ["Default", "Knockback", "Damage", "AttackSpeed"];
pub const CHANGE_ON: [&str; 2] = ["OnAttack", "OnTarget"];
pub const ANY: u16 = 1;
pub const SWORD: u16 = 2;
pub const AXE: u16 = 4;
pub const MACE: u16 = 8;
pub const SPEAR: u16 = 16;
pub const PICKAXE: u16 = 32;
pub const SHOVEL: u16 = 64;
pub const HOE: u16 = 128;
pub const KNOCKBACK: u16 = 256;
pub const FIRE_ASPECT: u16 = 512;
pub const ON_ATTACK: u8 = 1;
pub const ON_TARGET: u8 = 2;
/// Stable requester identity; must differ from all other SilentHotbar owners.
pub const REQUESTER: u64 = 0x4155_544f_5745_4150;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Priority {
    #[default]
    Default,
    Knockback,
    Damage,
    AttackSpeed,
}
impl Priority {
    pub fn tag(self) -> &'static str {
        PRIORITIES[self as usize]
    }
    pub fn parse(value: &Value) -> Result<Self, String> {
        let value = gson_string(value)?;
        Ok(
            match PRIORITIES.iter().position(|tag| java_tag_eq(tag, &value)) {
                Some(0) => Self::Default,
                Some(1) => Self::Knockback,
                Some(2) => Self::Damage,
                Some(3) => Self::AttackSpeed,
                _ => return Err(format!("Priority has no option named {value}")),
            },
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoWeapon {
    pub preferred: u16,
    pub priority: Priority,
    pub auto_shield_break: bool,
    pub auto_mace: bool,
    pub switch_back: i32,
    pub change_on: u8,
}
impl Default for AutoWeapon {
    fn default() -> Self {
        Self {
            preferred: SWORD,
            priority: Priority::Default,
            auto_shield_break: true,
            auto_mace: true,
            switch_back: 20,
            change_on: ON_ATTACK,
        }
    }
}

/// Enchantments keep the source's integer levels, including command-created levels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Enchantments {
    pub sharpness: i32,
    pub fire_aspect: i32,
    pub smite: i32,
    pub bane: i32,
    pub knockback: i32,
    pub looting: i32,
    pub unbreaking: i32,
    pub mending: i32,
    pub vanishing: i32,
    pub sweeping: i32,
    pub density: i32,
    pub breach: i32,
    pub wind_burst: i32,
}
/// The specialization in ItemCategorization's final `else` branch. Earlier
/// item-class branches and allowed player armour suppress this specialization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Specialization {
    #[default]
    None,
    Sword,
    Spear,
    SharpAxe,
    Mace,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeaponStack {
    pub slot: i32,
    /// Object identity used by getBest*Item's reference equality.
    pub identity: u64,
    /// Java-compatible signed identity hash, supplied by the stack allocator.
    pub identity_hash: i32,
    pub empty: bool,
    pub kinds: u16,
    pub specialization: Specialization,
    pub attack_damage: f64,
    pub attack_speed: f64,
    pub durability: i32,
    pub enchantable: i32,
    pub enchantments: Enchantments,
}
impl WeaponStack {
    pub fn matches(&self, preferred: u16) -> bool {
        preferred
            & (self.kinds
                | ANY
                | if self.enchantments.knockback > 0 {
                    KNOCKBACK
                } else {
                    0
                }
                | if self.enchantments.fire_aspect > 0 {
                    FIRE_ASPECT
                } else {
                    0
                })
            != 0
    }
    pub fn damage_score(&self, legacy: bool) -> f64 {
        let e = self.enchantments;
        let probability = 0.85_f64
            .powf(1.0 / 20.0)
            .powf((20.0 / self.attack_speed * 0.9).ceil()) as f32;
        let adjusted = if legacy {
            self.attack_damage
        } else {
            self.attack_damage * self.attack_speed * f64::from(probability)
        };
        let fire = (e.fire_aspect as f32 * 4.0 - 1.0).max(0.0) * 0.33;
        let mut additional = 0.0_f32;
        additional += e.smite as f32 * (2.0 * 0.1);
        additional += e.bane as f32 * (2.0 * 0.1);
        additional += e.knockback as f32 * 0.2;
        adjusted * (1.0 + f64::from(additional)) + f64::from(fire)
    }
    pub fn mace_score(&self) -> f64 {
        let e = self.enchantments;
        let probability =
            0.991_906_979_782_139_8_f64.powf((20.0 / self.attack_speed * 0.9).ceil()) as f32;
        let adjusted = self.attack_damage * self.attack_speed * f64::from(probability);
        let mut additional = 0.0_f32;
        additional += e.density as f32 * 0.5;
        additional += e.breach as f32 * 0.15;
        additional += e.smite as f32 * (2.0 * 0.1);
        additional += e.bane as f32 * (2.0 * 0.1);
        additional += e.wind_burst as f32 * 0.2;
        adjusted + f64::from(additional) + 29.0
    }
    fn secondary(&self) -> f32 {
        let e = self.enchantments;
        let mut value = 0.0_f32;
        value += e.looting as f32 * 0.05;
        value += e.unbreaking as f32 * 0.05;
        value += e.mending as f32 * 0.1;
        value += e.vanishing as f32 * -0.1;
        value += e.sweeping as f32 * 0.2;
        value += e.knockback as f32 * 0.25;
        value
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Target {
    pub living: bool,
    pub would_block_hit: bool,
}
pub struct Context<'a> {
    pub hotbar: &'a [WeaponStack],
    /// A default STICK ItemStackTemplate with actual player attribute bases.
    pub stick: &'a WeaponStack,
    pub legacy: bool,
    pub can_smash_attack: bool,
    pub mace_kill_enabled: bool,
    pub auto_buff_owns_slot: bool,
    pub consuming_main_hand: bool,
    pub current_main_hand: Option<&'a WeaponStack>,
    /// The real current ATTACK_SPEED attribute, before this module's override.
    pub current_attack_speed: f64,
}
/// Owned native snapshot. No inventory values are synthesized by the runtime.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub hotbar: Vec<WeaponStack>,
    pub stick: WeaponStack,
    pub legacy: bool,
    pub can_smash_attack: bool,
    pub consuming_main_hand: bool,
    pub current_attack_speed: f64,
    pub selected: i32,
}
impl Snapshot {
    pub fn parse(value: &Value) -> Option<Self> {
        fn stack(value: &Value) -> Option<WeaponStack> {
            let e = value.get("enchantments")?.as_array()?;
            if e.len() != 13 {
                return None;
            }
            let mut levels = [0_i32; 13];
            for (level, value) in levels.iter_mut().zip(e) {
                *level = i32::try_from(value.as_i64()?).ok()?;
            }
            Some(WeaponStack {
                slot: i32::try_from(value.get("slot")?.as_i64()?).ok()?,
                identity: value.get("identity")?.as_u64()?,
                identity_hash: i32::try_from(value.get("identityHash")?.as_i64()?).ok()?,
                empty: value.get("empty")?.as_bool()?,
                kinds: u16::try_from(value.get("kinds")?.as_u64()?).ok()?,
                specialization: match value.get("specialization")?.as_u64()? {
                    0 => Specialization::None,
                    1 => Specialization::Sword,
                    2 => Specialization::Spear,
                    3 => Specialization::SharpAxe,
                    4 => Specialization::Mace,
                    _ => return None,
                },
                attack_damage: value.get("damage")?.as_f64()?,
                attack_speed: value.get("speed")?.as_f64()?,
                durability: i32::try_from(value.get("durability")?.as_i64()?).ok()?,
                enchantable: i32::try_from(value.get("enchantable")?.as_i64()?).ok()?,
                enchantments: Enchantments {
                    sharpness: levels[0],
                    fire_aspect: levels[1],
                    smite: levels[2],
                    bane: levels[3],
                    knockback: levels[4],
                    looting: levels[5],
                    unbreaking: levels[6],
                    mending: levels[7],
                    vanishing: levels[8],
                    sweeping: levels[9],
                    density: levels[10],
                    breach: levels[11],
                    wind_burst: levels[12],
                },
            })
        }
        Some(Self {
            hotbar: value
                .get("hotbar")?
                .as_array()?
                .iter()
                .map(stack)
                .collect::<Option<Vec<_>>>()?,
            stick: stack(value.get("stick")?)?,
            legacy: value.get("legacy")?.as_bool()?,
            can_smash_attack: value.get("canSmash")?.as_bool()?,
            consuming_main_hand: value.get("consumingMainHand")?.as_bool()?,
            current_attack_speed: value.get("currentSpeed")?.as_f64()?,
            selected: i32::try_from(value.get("selected")?.as_i64()?).ok()?,
        })
    }
    pub fn context(&self, auto_buff_owns_slot: bool, mace_kill_enabled: bool) -> Context<'_> {
        Context {
            hotbar: &self.hotbar,
            stick: &self.stick,
            legacy: self.legacy,
            can_smash_attack: self.can_smash_attack,
            consuming_main_hand: self.consuming_main_hand,
            current_attack_speed: self.current_attack_speed,
            current_main_hand: self.hotbar.iter().find(|stack| stack.slot == self.selected),
            auto_buff_owns_slot,
            mace_kill_enabled,
        }
    }
}
impl Context<'_> {
    pub fn busy(&self) -> bool {
        self.auto_buff_owns_slot || self.consuming_main_hand
    }
    pub fn can_mace_smash(&self) -> bool {
        (!self.legacy && self.can_smash_attack) || self.mace_kill_enabled
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Switch {
    pub slot: i32,
    pub ticks: i32,
    pub sync_carried: bool,
}
#[derive(Clone, Copy)]
struct Facet<'a> {
    stack: &'a WeaponStack,
    mace: bool,
}
impl Facet<'_> {
    fn compare(self, other: Self, legacy: bool) -> Ordering {
        // MaceItemFacet.compareTo returns 0 against every generic facet.
        if self.mace && !other.mace {
            return Ordering::Equal;
        }
        let a = self.stack;
        let b = other.stack;
        let score = if self.mace {
            java_double_compare(a.mace_score(), b.mace_score())
        } else {
            java_double_compare(a.damage_score(legacy), b.damage_score(legacy))
        };
        score
            .then_with(|| {
                if self.mace {
                    Ordering::Equal
                } else {
                    java_float_compare(a.secondary(), b.secondary())
                }
            })
            .then_with(|| {
                (a.kinds & if self.mace { MACE } else { SWORD } != 0)
                    .cmp(&(b.kinds & if self.mace { MACE } else { SWORD } != 0))
            })
            .then_with(|| a.durability.cmp(&b.durability))
            .then_with(|| a.enchantable.cmp(&b.enchantable))
            // Both slots are hotbar slots, so PREFER_HOTBAR is equal.
            .then_with(|| a.identity_hash.cmp(&b.identity_hash))
    }
}
impl AutoWeapon {
    pub fn deserialize_preferred(&mut self, value: &Value) -> Result<(), String> {
        deserialize_mask(&mut self.preferred, value, &PREFERRED)
    }
    pub fn deserialize_change_on(&mut self, value: &Value) -> Result<(), String> {
        let mut value_mask = u16::from(self.change_on);
        let result = deserialize_mask(&mut value_mask, value, &CHANGE_ON);
        self.change_on = value_mask as u8;
        result
    }
    pub fn determine_weapon_slot<'a>(
        &self,
        context: &Context<'a>,
        target: Option<Target>,
        enforce_shield: bool,
    ) -> Option<&'a WeaponStack> {
        let shield =
            self.auto_shield_break && (enforce_shield || target.is_some_and(|t| t.would_block_hit));
        let mace = self.auto_mace && context.can_mace_smash();
        let mut priority_stack: Option<&WeaponStack> = None;
        if !mace && !shield && self.priority != Priority::Default {
            for stack in context
                .hotbar
                .iter()
                .filter(|stack| !stack.empty && stack.matches(self.preferred))
            {
                let compare = |a: &WeaponStack, b: &WeaponStack| match self.priority {
                    Priority::Knockback => a.enchantments.knockback.cmp(&b.enchantments.knockback),
                    Priority::Damage => java_double_compare(a.attack_damage, b.attack_damage),
                    Priority::AttackSpeed => java_double_compare(a.attack_speed, b.attack_speed),
                    Priority::Default => Ordering::Equal,
                };
                if priority_stack.is_none_or(|best| compare(stack, best).is_gt()) {
                    priority_stack = Some(stack);
                }
            }
        }
        let baseline = context.stick.damage_score(context.legacy);
        let mut best: Option<Facet<'a>> = None;
        for stack in context.hotbar.iter().filter(|stack| !stack.empty) {
            let accepted = if mace {
                stack.kinds & MACE != 0
            } else if shield {
                stack.kinds & AXE != 0
            } else if self.priority != Priority::Default {
                priority_stack.is_some_and(|best| stack.identity == best.identity)
            } else {
                stack.matches(self.preferred)
            };
            if !accepted {
                continue;
            }
            // Keep duplicate generic/specialized facets in source order.
            let useful = stack.damage_score(context.legacy) > baseline;
            let specialized = stack.specialization != Specialization::None;
            for facet in [
                useful.then_some(Facet { stack, mace: false }),
                specialized.then_some(Facet {
                    stack,
                    mace: stack.specialization == Specialization::Mace,
                }),
            ]
            .into_iter()
            .flatten()
            {
                if best.is_none_or(|current| facet.compare(current, context.legacy).is_gt()) {
                    best = Some(facet);
                }
            }
        }
        best.map(|facet| facet.stack)
    }
    pub fn on_attack(
        &self,
        running: bool,
        context: &Context<'_>,
        target: Target,
    ) -> Option<Switch> {
        if !running || !target.living {
            return None;
        }
        // Source determines the candidate before its isBusy / ChangeOn guards.
        let slot = self
            .determine_weapon_slot(context, Some(target), false)?
            .slot;
        if context.busy() || self.change_on & ON_ATTACK == 0 {
            return None;
        }
        Some(Switch {
            slot,
            ticks: self.switch_back,
            sync_carried: true,
        })
    }
    pub fn on_target(
        &self,
        running: bool,
        context: &Context<'_>,
        target: Option<Target>,
    ) -> Option<Switch> {
        let target = target?;
        if !running || !target.living || context.busy() || self.change_on & ON_TARGET == 0 {
            return None;
        }
        let slot = self
            .determine_weapon_slot(context, Some(target), false)?
            .slot;
        Some(Switch {
            slot,
            ticks: self.switch_back,
            sync_carried: false,
        })
    }
    pub fn will_shield_break(&self, context: &Context<'_>) -> bool {
        !context.legacy
            && (context
                .current_main_hand
                .is_some_and(|stack| stack.kinds & AXE != 0)
                || self
                    .determine_weapon_slot(context, None, true)
                    .is_some_and(|stack| stack.kinds & AXE != 0))
    }
    pub fn will_mace_smash(&self, context: &Context<'_>) -> bool {
        context.can_mace_smash()
            && (context
                .current_main_hand
                .is_some_and(|stack| stack.kinds & MACE != 0)
                || self
                    .determine_weapon_slot(context, None, false)
                    .is_some_and(|stack| stack.kinds & MACE != 0))
    }
    pub fn attack_speed(
        &self,
        running: bool,
        reach_running: bool,
        context: &Context<'_>,
        original: f64,
    ) -> f64 {
        // Actual local-player MixinPlayer guard returns original while Reach runs.
        if reach_running
            || !running
            || self.change_on & ON_ATTACK == 0
            || context.legacy
            || context.current_attack_speed >= 20.0
            || context.current_attack_speed.is_nan()
        {
            return original;
        }
        self.determine_weapon_slot(context, None, false)
            .map_or(original, |stack| stack.attack_speed)
    }
}

pub fn java_double_compare(a: f64, b: f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    }
    if a > b {
        return Ordering::Greater;
    }
    let canonical = |value: f64| {
        if value.is_nan() {
            0x7ff8_0000_0000_0000_i64
        } else {
            value.to_bits() as i64
        }
    };
    canonical(a).cmp(&canonical(b))
}
pub fn java_float_compare(a: f32, b: f32) -> Ordering {
    if a < b {
        return Ordering::Less;
    }
    if a > b {
        return Ordering::Greater;
    }
    let canonical = |value: f32| {
        if value.is_nan() {
            0x7fc0_0000_i32
        } else {
            value.to_bits() as i32
        }
    };
    canonical(a).cmp(&canonical(b))
}
fn deserialize_mask(mask: &mut u16, value: &Value, tags: &[&str]) -> Result<(), String> {
    *mask = 0;
    let mut include = |value: &Value| -> Result<(), String> {
        let value = gson_string(value)?;
        if let Some(index) = tags.iter().position(|tag| java_tag_eq(tag, &value)) {
            *mask |= 1 << index;
        }
        Ok(())
    };
    match value {
        Value::Array(values) => {
            for value in values {
                include(value)?;
            }
        }
        Value::String(_) | Value::Number(_) | Value::Bool(_) => include(value)?,
        _ => {}
    }
    Ok(())
}
fn gson_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Array(values) if values.len() == 1 => gson_string(&values[0]),
        _ => Err("entry has no Gson string value".to_owned()),
    }
}
fn java_tag_eq(choice: &str, value: &str) -> bool {
    let fold = |character: char| {
        let mut upper = character.to_uppercase();
        let first = upper.next().unwrap_or(character);
        let upper = if upper.next().is_none() {
            first
        } else {
            character
        };
        upper.to_lowercase().next().unwrap_or(upper)
    };
    choice.chars().map(fold).eq(value.chars().map(fold))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn stack(slot: i32, kinds: u16, damage: f64, speed: f64) -> WeaponStack {
        WeaponStack {
            slot,
            identity: (slot as u64).wrapping_add(1),
            identity_hash: slot,
            kinds,
            attack_damage: damage,
            attack_speed: speed,
            ..Default::default()
        }
    }
    fn context<'a>(hotbar: &'a [WeaponStack], stick: &'a WeaponStack) -> Context<'a> {
        Context {
            hotbar,
            stick,
            legacy: false,
            can_smash_attack: false,
            mace_kill_enabled: false,
            auto_buff_owns_slot: false,
            consuming_main_hand: false,
            current_main_hand: None,
            current_attack_speed: 4.0,
        }
    }
    #[test]
    fn native_snapshot_preserves_effective_hand_and_policy_inputs() {
        let item = |slot, kinds, specialization, damage, speed| {
            json!({"slot":slot,"identity":u64::MAX-3,"identityHash":2_147_483_647,
                "empty":false,"kinds":kinds,"specialization":specialization,
                "damage":damage,"speed":speed,"durability":i32::MIN,"enchantable":-9,
                "enchantments":[0,0,0,0,0,2,0,0,0,0,0,0,0]})
        };
        let mut value = json!({"hotbar":[item(2,SWORD,1,7.0,1.6),item(5,AXE,0,9.0,0.8)],
            "stick":item(-1,0,0,1.0,4.0),"legacy":false,"canSmash":false,
            "consumingMainHand":false,"currentSpeed":4.0,"selected":5});
        let snapshot = Snapshot::parse(&value).unwrap();
        let ctx = snapshot.context(false, false);
        assert_eq!(ctx.current_main_hand.unwrap().slot, 5);
        assert_eq!(ctx.hotbar[0].identity, u64::MAX - 3);
        assert_eq!(ctx.hotbar[0].identity_hash, i32::MAX);
        assert_eq!(ctx.hotbar[0].durability, i32::MIN);
        assert_eq!(ctx.hotbar[0].enchantable, -9);
        assert_eq!(ctx.hotbar[0].enchantments.looting, 2);
        let weapon = AutoWeapon::default();
        assert_eq!(
            weapon.on_attack(
                true,
                &ctx,
                Target {
                    living: true,
                    would_block_hit: false
                }
            ),
            Some(Switch {
                slot: 2,
                ticks: 20,
                sync_carried: true
            })
        );
        assert!(snapshot.context(true, false).busy());
        assert!(snapshot.context(false, true).can_mace_smash());
        value["consumingMainHand"] = json!(true);
        assert!(
            Snapshot::parse(&value)
                .unwrap()
                .context(false, false)
                .busy()
        );
        value["hotbar"][0]["enchantments"] = json!([1, 2]);
        assert!(Snapshot::parse(&value).is_none());
        value["hotbar"] = json!([]);
        assert!(
            Snapshot::parse(&value)
                .unwrap()
                .context(false, false)
                .current_main_hand
                .is_none()
        );
        value["stick"]["durability"] = json!(2_147_483_648_u64);
        assert!(Snapshot::parse(&value).is_none());
    }
    #[test]
    fn source_jvm_arithmetic_vectors() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("java-vectors.json");
        if !path.is_file() {
            return;
        }
        let vectors: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let bits = |v: &Value| v.as_str().unwrap().parse::<u64>().unwrap();
        for vector in vectors.as_array().unwrap() {
            let e = vector["e"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect::<Vec<_>>();
            let stack = WeaponStack {
                attack_damage: f64::from_bits(bits(&vector["damage"])),
                attack_speed: f64::from_bits(bits(&vector["speed"])),
                enchantments: Enchantments {
                    sharpness: e[0],
                    fire_aspect: e[1],
                    smite: e[2],
                    bane: e[3],
                    knockback: e[4],
                    looting: e[5],
                    unbreaking: e[6],
                    mending: e[7],
                    vanishing: e[8],
                    sweeping: e[9],
                    density: e[10],
                    breach: e[11],
                    wind_burst: e[12],
                },
                ..Default::default()
            };
            for (key, actual) in [
                ("modern", stack.damage_score(false)),
                ("legacy", stack.damage_score(true)),
                ("mace", stack.mace_score()),
            ] {
                let expected = f64::from_bits(bits(&vector[key]));
                // libm pow implementations may differ by the final bit; the
                // source's probability narrowing to float is still verified.
                assert!(
                    java_double_compare(actual, expected) == Ordering::Equal
                        || (actual - expected).abs() <= expected.abs() * 1.0e-14,
                    "{key}: {actual:?} {expected:?}"
                );
            }
            assert_eq!(
                stack.secondary().to_bits(),
                vector["secondary"].as_u64().unwrap() as u32
            );
        }
    }
    #[test]
    fn all_preferred_filters_and_live_multichoice_failures() {
        let mut state = AutoWeapon::default();
        for (index, tag) in PREFERRED.iter().enumerate() {
            state.deserialize_preferred(&json!(tag)).unwrap();
            assert_eq!(state.preferred, 1 << index);
        }
        assert!(
            state
                .deserialize_preferred(&json!(["Axe", "Mace", null, "Sword"]))
                .is_err()
        );
        assert_eq!(state.preferred, AXE | MACE);
        state
            .deserialize_preferred(&json!([["Pıckaxe"], true, 9, "unknown"]))
            .unwrap();
        assert_eq!(state.preferred, PICKAXE);
        state
            .deserialize_change_on(&json!(["OnTarget", {}]))
            .unwrap_err();
        assert_eq!(state.change_on, ON_TARGET);
        for value in [json!(null), json!({}), json!([])] {
            state.deserialize_change_on(&value).unwrap();
            assert_eq!(state.change_on, 0);
        }
        assert_eq!(
            Priority::parse(&json!("Attackspeed")).unwrap(),
            Priority::AttackSpeed
        );
        assert!(Priority::parse(&json!("other")).is_err());
        assert!(Priority::parse(&json!(null)).is_err());
        let mut enchanted = stack(0, 0, 1.0, 4.0);
        enchanted.enchantments.knockback = 1;
        enchanted.enchantments.fire_aspect = 1;
        assert!(enchanted.matches(ANY | KNOCKBACK | FIRE_ASPECT));
        assert!(!enchanted.matches(SWORD | AXE));
    }
    #[test]
    fn shield_mace_and_priority_precedence_keep_missing_facet_result() {
        let stick = stack(-1, 0, 1.0, 4.0);
        let mut bar = [
            stack(0, SWORD, 7.0, 1.6),
            stack(1, AXE, 9.0, 1.0),
            stack(2, MACE, 6.0, 0.6),
            stack(3, 0, 1.0, 4.0),
        ];
        for item in &mut bar[..3] {
            item.specialization = match item.kinds {
                SWORD => Specialization::Sword,
                MACE => Specialization::Mace,
                _ => Specialization::None,
            };
        }
        let mut state = AutoWeapon::default();
        let mut ctx = context(&bar, &stick);
        assert_eq!(
            state.determine_weapon_slot(&ctx, None, false).unwrap().slot,
            0
        );
        let target = Some(Target {
            living: true,
            would_block_hit: true,
        });
        assert_eq!(
            state
                .determine_weapon_slot(&ctx, target, false)
                .unwrap()
                .slot,
            1
        );
        ctx.can_smash_attack = true;
        assert_eq!(
            state
                .determine_weapon_slot(&ctx, target, false)
                .unwrap()
                .slot,
            2
        );
        ctx.can_smash_attack = false;
        state.auto_shield_break = false;
        state.preferred = ANY;
        state.priority = Priority::AttackSpeed;
        // Nonempty raw best speed candidate has no WeaponItemFacet: no fallback.
        assert!(state.determine_weapon_slot(&ctx, None, false).is_none());
        state.priority = Priority::Damage;
        assert_eq!(
            state.determine_weapon_slot(&ctx, None, false).unwrap().slot,
            1
        );
        state.preferred = 0;
        assert!(state.determine_weapon_slot(&ctx, None, false).is_none());
        ctx.can_smash_attack = true;
        assert_eq!(
            state.determine_weapon_slot(&ctx, None, false).unwrap().slot,
            2
        );
    }
    #[test]
    fn asymmetric_mace_and_signed_java_float_order_are_intentional() {
        let a = stack(0, MACE, 6.0, 0.6);
        let b = stack(1, SWORD, 7.0, 1.6);
        assert_eq!(
            Facet {
                stack: &a,
                mace: true
            }
            .compare(
                Facet {
                    stack: &b,
                    mace: false
                },
                false
            ),
            Ordering::Equal
        );
        assert_eq!(java_double_compare(-0.0, 0.0), Ordering::Less);
        assert_eq!(
            java_double_compare(f64::from_bits(0xfff8_0000_0000_0001), f64::NAN),
            Ordering::Equal
        );
        assert_eq!(
            java_double_compare(f64::NAN, f64::INFINITY),
            Ordering::Greater
        );
        assert_eq!(java_float_compare(-0.0, 0.0), Ordering::Less);
        let mut c = a.clone();
        c.identity = 99;
        c.identity_hash = i32::MAX;
        assert!(
            Facet {
                stack: &c,
                mace: false
            }
            .compare(
                Facet {
                    stack: &a,
                    mace: false
                },
                false
            )
            .is_gt()
        );
    }
    #[test]
    fn attack_target_helpers_and_speed_have_distinct_source_guards() {
        let stick = stack(-1, 0, 1.0, 4.0);
        let mut sword = stack(0, SWORD, 7.0, 1.6);
        sword.specialization = Specialization::Sword;
        let bar = [sword];
        let mut ctx = context(&bar, &stick);
        let mut state = AutoWeapon::default();
        let target = Target {
            living: true,
            would_block_hit: false,
        };
        assert_eq!(
            state.on_attack(true, &ctx, target),
            Some(Switch {
                slot: 0,
                ticks: 20,
                sync_carried: true
            })
        );
        assert!(state.on_attack(false, &ctx, target).is_none());
        assert!(state.on_attack(true, &ctx, Target::default()).is_none());
        assert!(state.on_target(true, &ctx, Some(target)).is_none());
        state.change_on |= ON_TARGET;
        state.switch_back = i32::MIN;
        assert_eq!(
            state.on_target(true, &ctx, Some(target)),
            Some(Switch {
                slot: 0,
                ticks: i32::MIN,
                sync_carried: false
            })
        );
        ctx.auto_buff_owns_slot = true;
        assert!(state.on_attack(true, &ctx, target).is_none());
        assert_eq!(
            state.attack_speed(true, false, &ctx, 4.0),
            1.6,
            "speed query ignores isBusy"
        );
        assert_eq!(
            state.attack_speed(true, true, &ctx, 4.0),
            4.0,
            "Reach guard"
        );
        ctx.current_attack_speed = 20.0;
        assert_eq!(state.attack_speed(true, false, &ctx, 4.0), 4.0);
        ctx.current_attack_speed = 4.0;
        ctx.legacy = true;
        assert_eq!(state.attack_speed(true, false, &ctx, 4.0), 4.0);
        assert!(!state.will_shield_break(&ctx));
        assert!(!state.will_mace_smash(&ctx));
    }
}
