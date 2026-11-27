// SPDX-License-Identifier: GPL-3.0-or-later
//! TriggerBot and both AutoClicker buttons, in source declaration order.
use super::*;
use liquidbounce_behaviors::{
    clicking::{Settings as ClickSettings, Technique},
    crosshair_clicker::{Attacker, Criticals, Id, Objective, OnItemUse, State},
};
use std::collections::BTreeSet;
const WEAPONS: [&str; 10] = liquidbounce_behaviors::auto_weapon::PREFERRED;
#[derive(Clone, Debug, PartialEq)]
pub struct CrosshairClickerSetting {
    pub id: Id,
    pub path: String,
    pub value: Value,
}
impl CrosshairClickerSetting {
    pub fn apply(
        self,
        s: &mut State,
        items: &[String],
        blocks: &[String],
    ) -> Result<(), ConfigError> {
        let path = format!("{}.{}", self.id.module(), self.path);
        let v = &self.value;
        let a = if self.id == Id::Trigger {
            &mut s.trigger.attacker
        } else {
            &mut s.auto.attacker
        };
        match self.path.as_str() {
            "OnItemUse" => {
                a.on_item_use = match choose(v, &path, &["Wait", "Stop", "Ignore"])? {
                    0 => OnItemUse::Wait,
                    1 => OnItemUse::Stop,
                    _ => OnItemUse::Ignore,
                }
            }
            "Weapon" => {
                // The source deserializer mutates the set entry by entry, so
                // a malformed later entry retains the successfully read prefix.
                let mut parser = liquidbounce_behaviors::auto_weapon::AutoWeapon::default();
                parser.preferred = a.weapon;
                let result = parser.deserialize_preferred(v);
                a.weapon = parser.preferred;
                result.map_err(|error| ConfigError::new(&path, &error))?;
            }
            "Criticals" => {
                a.criticals = match choose(v, &path, &["Smart", "Ignore", "Always"])? {
                    0 => Criticals::Smart,
                    1 => Criticals::Ignore,
                    _ => Criticals::Always,
                }
            }
            "DelayPostStopUse" => a.delay_post_stop_use = parse_int(v, &path)?,
            "Enabled" => {
                if self.id == Id::Use {
                    s.auto.use_enabled = parse_bool(v, &path)?
                } else {
                    s.auto.attack_enabled = parse_bool(v, &path)?
                }
            }
            "DelayOnBroken" => s.auto.delay_on_broken = parse_bool(v, &path)?,
            "Objective" => {
                s.auto.objective = match choose(v, &path, &["Enemy", "Entity", "Block", "Any"])? {
                    0 => Objective::Enemy,
                    1 => Objective::Entity,
                    2 => Objective::Block,
                    _ => Objective::Any,
                }
            }
            "HoldingItemsForIgnore" => s.auto.holding_items_for_ignore = registry(v, items, &path)?,
            "BlocksForIgnore" => {
                s.auto.blocks_for_ignore = registry(v, blocks, &path)?;
                s.blocks_configured = true;
            }
            "DelayStart" => s.auto.delay_start = parse_bool(v, &path)?,
            "OnlyBlock" => s.auto.only_block = parse_bool(v, &path)?,
            "RequiresNoInput" => s.auto.requires_no_input = parse_bool(v, &path)?,
            "Technique" => {
                s.clicker(self.id).settings.technique =
                    if choose(v, &path, &["Human", "Constant"])? == 0 {
                        Technique::Human
                    } else {
                        Technique::Constant
                    }
            }
            "CPS" => s.clicker(self.id).settings.cps = parse_int_pair(v, &path)?,
            "MaxPerTick" => s.clicker(self.id).settings.max_per_tick = parse_int(v, &path)?,
            "Minimum" => {
                s.clicker(self.id).settings.item_cooldown = Some(parse_float_pair(v, &path)?)
            }
            "MissCooldown" => s.clicker(self.id).settings.miss_cooldown = parse_bool(v, &path)?,
            _ => return Err(ConfigError::new(&path, "unknown clicker setting")),
        }
        Ok(())
    }
}
fn choose(v: &Value, path: &str, choices: &[&str]) -> Result<usize, ConfigError> {
    scalar_string(v)
        .and_then(|v| choices.iter().position(|tag| tag.eq_ignore_ascii_case(&v)))
        .ok_or_else(|| ConfigError::new(path, "unknown choice"))
}
fn registry(v: &Value, available: &[String], path: &str) -> Result<BTreeSet<String>, ConfigError> {
    let resolve = |v: &Value| {
        let text = if v.is_string() || v.is_number() {
            scalar_string(v)
        } else {
            None
        }
        .ok_or_else(|| ConfigError::new(path, "registry entry must be a string or number"))?;
        let id = match text.find(':') {
            None => format!("minecraft:{text}"),
            Some(0) => format!("minecraft{text}"),
            _ => text,
        };
        Ok(if available.contains(&id) {
            id
        } else {
            "minecraft:air".into()
        })
    };
    if let Some(values) = v.as_array() {
        values.iter().map(resolve).collect()
    } else {
        Ok([resolve(v)?].into())
    }
}
fn sorted(values: &BTreeSet<String>) -> Value {
    let mut v: Vec<_> = values.iter().collect();
    v.sort_by(|a, b| {
        let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
        let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
        ap.cmp(bp).then(an.cmp(bn))
    });
    json!(v)
}
fn choice(tree: &Tree<'_>, name: &str, value: &str, choices: &[&str], key: &str) -> Value {
    let mut v = tree.leaf(name, "CHOOSE", json!(value), key, None);
    if tree.format == JsonFormat::Interop {
        v["choices"] = json!(choices);
    }
    v
}
pub(super) fn click_tree(tree: &Tree<'_>, s: ClickSettings, key: &str) -> Result<Value, ConfigError> {
    let mut v = vec![
        choice(
            tree,
            "Technique",
            if s.technique == Technique::Human {
                "Human"
            } else {
                "Constant"
            },
            &["Human", "Constant"],
            &format!("{key}.technique"),
        ),
        tree.ranged(
            "CPS",
            "INT_RANGE",
            json!({"from":s.cps[0],"to":s.cps[1]}),
            json!({"from":1,"to":30}),
            "clicks",
            &format!("{key}.cps"),
        ),
        tree.ranged(
            "MaxPerTick",
            "INT",
            json!(s.max_per_tick),
            json!({"from":1,"to":5}),
            "clicks",
            &format!("{key}.maxPerTick"),
        ),
    ];
    if let Some(c) = s.item_cooldown {
        v.push(tree.group(
            "ItemCooldown",
            "CONFIGURABLE",
            vec![tree.ranged(
                "Minimum",
                "FLOAT_RANGE",
                json!({"from":float_json(c[0],key)?,"to":float_json(c[1],key)?}),
                json!({"from":0.0,"to":2.0}),
                "",
                &format!("{key}.itemCooldown.minimum"),
            )],
        ));
        v.push(tree.leaf(
            "MissCooldown",
            "BOOLEAN",
            json!(s.miss_cooldown),
            &format!("{key}.missCooldown"),
            None,
        ));
    }
    Ok(tree.group("Clicker", "CONFIGURABLE", v))
}
fn attack_tree(tree: &Tree<'_>, a: Attacker, key: &str) -> Vec<Value> {
    let mut weapon = tree.multi(
        "Weapon",
        u32::from(a.weapon),
        &WEAPONS,
        &format!("{key}.weapon"),
    );
    if tree.format == JsonFormat::Interop {
        weapon["canBeNone"] = json!(false);
    }
    vec![
        choice(
            tree,
            "OnItemUse",
            match a.on_item_use {
                OnItemUse::Wait => "Wait",
                OnItemUse::Stop => "Stop",
                OnItemUse::Ignore => "Ignore",
            },
            &["Wait", "Stop", "Ignore"],
            &format!("{key}.onItemUse"),
        ),
        weapon,
        choice(
            tree,
            "Criticals",
            match a.criticals {
                Criticals::Smart => "Smart",
                Criticals::Ignore => "Ignore",
                Criticals::Always => "Always",
            },
            &["Smart", "Ignore", "Always"],
            &format!("{key}.criticals"),
        ),
        tree.ranged(
            "DelayPostStopUse",
            "INT",
            json!(a.delay_post_stop_use),
            json!({"from":0,"to":20}),
            "ticks",
            &format!("{key}.delayPostStopUse"),
        ),
    ]
}
pub fn serialize_trigger_bot(
    s: &State,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let key = "liquidbounce.module.triggerBot";
    let mut v = t.common(ModuleKind::TriggerBot, common);
    v.push(click_tree(
        &t,
        s.trigger.clicker.settings,
        &format!("{key}.clicker"),
    )?);
    v.extend(attack_tree(&t, s.trigger.attacker, key));
    Ok(t.group("TriggerBot", "TOGGLEABLE", v))
}
pub fn serialize_auto_clicker(
    s: &State,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let key = "liquidbounce.module.autoClicker";
    let mut v = t.common(ModuleKind::AutoClicker, common);
    let mut attack = vec![
        t.leaf(
            "Enabled",
            "BOOLEAN",
            json!(s.auto.attack_enabled),
            &format!("{key}.attack.enabled"),
            None,
        ),
        click_tree(&t, s.auto.attack.settings, &format!("{key}.attack.clicker"))?,
        t.leaf(
            "DelayOnBroken",
            "BOOLEAN",
            json!(s.auto.delay_on_broken),
            &format!("{key}.attack.delayOnBroken"),
            None,
        ),
        choice(
            &t,
            "Objective",
            match s.auto.objective {
                Objective::Enemy => "Enemy",
                Objective::Entity => "Entity",
                Objective::Block => "Block",
                Objective::Any => "Any",
            },
            &["Enemy", "Entity", "Block", "Any"],
            &format!("{key}.attack.objective"),
        ),
    ];
    attack.extend(attack_tree(&t, s.auto.attacker, &format!("{key}.attack")));
    v.push(t.group("Attack", "TOGGLEABLE", attack));
    let mut use_key = vec![
        t.leaf(
            "Enabled",
            "BOOLEAN",
            json!(s.auto.use_enabled),
            &format!("{key}.use.enabled"),
            None,
        ),
        click_tree(&t, s.auto.use_key.settings, &format!("{key}.use.clicker"))?,
    ];
    for (name, set, registry) in [
        (
            "HoldingItemsForIgnore",
            &s.auto.holding_items_for_ignore,
            "item",
        ),
        ("BlocksForIgnore", &s.auto.blocks_for_ignore, "block"),
    ] {
        let mut leaf = t.leaf(
            name,
            "REGISTRY_LIST",
            sorted(set),
            &format!(
                "{key}.use.{}",
                if name == "BlocksForIgnore" {
                    "blocksForIgnore"
                } else {
                    "holdingItemsForIgnore"
                }
            ),
            None,
        );
        if format == JsonFormat::Interop {
            leaf["registry"] = json!(registry);
        }
        use_key.push(leaf);
    }
    for (name, value, suffix) in [
        ("DelayStart", s.auto.delay_start, "delayStart"),
        ("OnlyBlock", s.auto.only_block, "onlyBlock"),
        (
            "RequiresNoInput",
            s.auto.requires_no_input,
            "requiresNoInput",
        ),
    ] {
        use_key.push(t.leaf(
            name,
            "BOOLEAN",
            json!(value),
            &format!("{key}.use.{suffix}"),
            None,
        ));
    }
    v.push(t.group("Use", "TOGGLEABLE", use_key));
    Ok(t.group("AutoClicker", "TOGGLEABLE", v))
}
fn collect(patch: &mut SettingsPatch, root: &Nodes<'_>, id: Id, names: &[&str]) {
    for name in names {
        if let Some(node) = root.get(*name) {
            match value_of(node, name) {
                Ok(value) => {
                    patch
                        .changes
                        .push(SettingChange::CrosshairClicker(CrosshairClickerSetting {
                            id,
                            path: (*name).into(),
                            value: value.clone(),
                        }))
                }
                Err(e) => patch.issues.push(e),
            }
        }
    }
}
fn group<'a>(
    patch: &mut SettingsPatch,
    root: &Nodes<'a>,
    name: &str,
    alias: &str,
) -> Option<Nodes<'a>> {
    let node = root.get(name).or_else(|| root.get(alias))?;
    let actual = node["name"].as_str().unwrap_or(name);
    match nodes(node, actual, name) {
        Ok(v) => Some(v),
        Err(e) => {
            patch.issues.push(e);
            None
        }
    }
}
fn collect_clicker(p: &mut SettingsPatch, root: &Nodes<'_>, id: Id) {
    let Some(c) = group(p, root, "Clicker", "ClickScheduler") else {
        return;
    };
    collect(p, &c, id, &["Technique", "CPS", "MaxPerTick"]);
    if id.attack() {
        if let Some(cooldown) = group(p, &c, "ItemCooldown", "Cooldown") {
            collect(p, &cooldown, id, &["Minimum"]);
        }
        let mut c = c;
        if !c.contains_key("MissCooldown") {
            if let Some(v) = c.remove("AttackCooldown") {
                c.insert("MissCooldown".into(), v);
            }
        }
        collect(p, &c, id, &["MissCooldown"]);
    }
}
pub(super) fn parse_trigger_bot(p: &mut SettingsPatch, root: &Nodes<'_>) {
    collect_clicker(p, root, Id::Trigger);
    collect(
        p,
        root,
        Id::Trigger,
        &["OnItemUse", "Weapon", "Criticals", "DelayPostStopUse"],
    );
}
pub(super) fn parse_auto_clicker(p: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(attack) = group(p, root, "Attack", "Attack") {
        collect(p, &attack, Id::Attack, &["Enabled"]);
        collect_clicker(p, &attack, Id::Attack);
        collect(
            p,
            &attack,
            Id::Attack,
            &[
                "DelayOnBroken",
                "Objective",
                "OnItemUse",
                "Weapon",
                "Criticals",
                "DelayPostStopUse",
            ],
        );
    }
    if let Some(use_key) = group(p, root, "Use", "Use") {
        collect(p, &use_key, Id::Use, &["Enabled"]);
        collect_clicker(p, &use_key, Id::Use);
        collect(
            p,
            &use_key,
            Id::Use,
            &[
                "HoldingItemsForIgnore",
                "BlocksForIgnore",
                "DelayStart",
                "OnlyBlock",
                "RequiresNoInput",
            ],
        );
    }
}
