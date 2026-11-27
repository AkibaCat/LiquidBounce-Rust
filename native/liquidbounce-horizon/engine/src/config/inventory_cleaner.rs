// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    elytra_swap::ElytraSwap,
    inventory_cleaner::{CHOICES, InventoryCleaner},
};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq)]
pub enum InventoryCleanerSetting {
    Constraints(ElytraSwapSetting),
    Value(&'static str, Value),
}
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e.path.replace("ElytraSwap", "InventoryCleaner");
    e
}
const INTS: [(&str, i32); 8] = [
    ("MaximumBlocks", 2500),
    ("MaximumArrows", 2500),
    ("MaximumThrowables", 600),
    ("MaximumFoodPoints", 2000),
    ("MaximumWaterBuckets", 16),
    ("MaximumLavaBuckets", 16),
    ("MaximumMilkBuckets", 16),
    ("MaximumTotems", 37),
];
const SLOTS: [&str; 10] = [
    "OffHandItem",
    "SlotItem-1",
    "SlotItem-2",
    "SlotItem-3",
    "SlotItem-4",
    "SlotItem-5",
    "SlotItem-6",
    "SlotItem-7",
    "SlotItem-8",
    "SlotItem-9",
];
impl InventoryCleanerSetting {
    pub fn apply(self, s: &mut InventoryCleaner, items: &[String]) -> Result<(), ConfigError> {
        let Self::Value(name, v) = self else {
            if let Self::Constraints(c) = self {
                let mut p = ElytraSwap {
                    constraints: s.constraints.clone(),
                    ..Default::default()
                };
                c.apply(&mut p).map_err(remap)?;
                s.constraints = p.constraints;
            }
            return Ok(());
        };
        let path = format!("InventoryCleaner.{name}");
        if let Some(i) = SLOTS.iter().position(|k| *k == name) {
            let raw =
                scalar_string(&v).ok_or_else(|| ConfigError::new(&path, "expected enum tag"))?;
            s.slots[i] = CHOICES
                .iter()
                .position(|s| s.eq_ignore_ascii_case(&raw))
                .ok_or_else(|| ConfigError::new(&path, "unknown item category"))?;
            return Ok(());
        }
        match name {
            "Order" => {
                s.distance = match scalar_string(&v).as_deref() {
                    Some(s) if s.eq_ignore_ascii_case("Value") => false,
                    Some(s) if s.eq_ignore_ascii_case("Distance") => true,
                    _ => return Err(ConfigError::new(&path, "unknown order")),
                }
            }
            "MaximumBlocks" => s.max_blocks = parse_int(&v, &path)?,
            "MaximumArrows" => s.max_arrows = parse_int(&v, &path)?,
            "MaximumThrowables" => s.max_throwables = parse_int(&v, &path)?,
            "MaximumFoodPoints" => s.max_food = parse_int(&v, &path)?,
            "MaximumWaterBuckets" => s.max_water = parse_int(&v, &path)?,
            "MaximumLavaBuckets" => s.max_lava = parse_int(&v, &path)?,
            "MaximumMilkBuckets" => s.max_milk = parse_int(&v, &path)?,
            "MaximumTotems" => s.max_totems = parse_int(&v, &path)?,
            "ThrowUnknownItems" => s.throw_unknown = parse_bool(&v, &path)?,
            "Greedy" => s.greedy = parse_bool(&v, &path)?,
            "ItemsBlacklist" => {
                let entries = match &v {
                    Value::Array(a) => a.clone(),
                    _ => vec![v],
                };
                let mut set = BTreeSet::new();
                for entry in entries {
                    let id = match &entry {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        _ => {
                            return Err(ConfigError::new(
                                &path,
                                "registry entry requires string or number",
                            ));
                        }
                    };
                    let id = match id.find(':') {
                        None => format!("minecraft:{id}"),
                        Some(0) => format!("minecraft{id}"),
                        _ => id,
                    };
                    set.insert(if items.contains(&id) {
                        id
                    } else {
                        "minecraft:air".into()
                    });
                }
                s.blacklist = set;
            }
            _ => return Err(ConfigError::new(&path, "unknown setting")),
        }
        Ok(())
    }
}
fn rename(v: &mut Value) {
    match v {
        Value::String(s) => *s = s.replace("elytraSwap", "inventoryCleaner"),
        Value::Array(a) => a.iter_mut().for_each(rename),
        Value::Object(o) => o.values_mut().for_each(rename),
        _ => {}
    }
}
pub fn serialize_inventory_cleaner(
    s: &InventoryCleaner,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let key = |s: &str| {
        format!(
            "liquidbounce.module.inventoryCleaner.{}{}",
            s[..1].to_lowercase(),
            &s[1..]
        )
    };
    let p = ElytraSwap {
        constraints: s.constraints.clone(),
        ..Default::default()
    };
    let mut constraints = serialize_elytra_swap(&p, c, f, &|k| {
        d(&k.replace("elytraSwap", "inventoryCleaner"))
    })?["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Constraints")
        .unwrap()
        .clone();
    rename(&mut constraints);
    let mut values = t.common(ModuleKind::InventoryCleaner, c);
    values.push(constraints);
    let choose = |name: &str, value: &str, choices: &[&str]| {
        let mut v = t.leaf(name, "CHOOSE", json!(value), &key(name), None);
        if f == JsonFormat::Interop {
            v["choices"] = json!(choices);
        }
        v
    };
    values.push(choose(
        "Order",
        if s.distance { "Distance" } else { "Value" },
        &["Value", "Distance"],
    ));
    for ((name, max), value) in INTS.into_iter().zip([
        s.max_blocks,
        s.max_arrows,
        s.max_throwables,
        s.max_food,
        s.max_water,
        s.max_lava,
        s.max_milk,
        s.max_totems,
    ]) {
        values.push(t.ranged(
            name,
            "INT",
            json!(value),
            json!({"from":0,"to":max}),
            "",
            &key(name),
        ));
    }
    let mut blacklist = t.leaf(
        "ItemsBlacklist",
        "REGISTRY_LIST",
        json!(s.blacklist),
        &key("ItemsBlacklist"),
        None,
    );
    if f == JsonFormat::Interop {
        blacklist["registry"] = json!("item");
    }
    values.push(blacklist);
    for (name, value) in [("ThrowUnknownItems", s.throw_unknown), ("Greedy", s.greedy)] {
        values.push(t.leaf(name, "BOOLEAN", json!(value), &key(name), None));
    }
    for (name, &value) in SLOTS.iter().zip(&s.slots) {
        values.push(choose(name, CHOICES[value], &CHOICES));
    }
    Ok(t.group("InventoryCleaner", "TOGGLEABLE", values))
}
pub(super) fn parse_inventory_cleaner(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let mut proxy = SettingsPatch {
        module: ModuleKind::ElytraSwap,
        changes: vec![],
        issues: vec![],
    };
    elytra_swap::parse_elytra_swap(&mut proxy, root);
    for c in proxy.changes {
        if let SettingChange::ElytraSwap(c) = c {
            p.changes.push(SettingChange::InventoryCleaner(
                InventoryCleanerSetting::Constraints(c),
            ));
        }
    }
    p.issues.extend(proxy.issues.into_iter().map(remap));
    for name in ["Order", "ItemsBlacklist", "ThrowUnknownItems", "Greedy"]
        .into_iter()
        .chain(INTS.map(|p| p.0))
        .chain(SLOTS)
    {
        leaf(
            p,
            root,
            name,
            &format!("InventoryCleaner.{name}"),
            |v, _| {
                Ok(SettingChange::InventoryCleaner(
                    InventoryCleanerSetting::Value(name, v.clone()),
                ))
            },
        );
    }
}
