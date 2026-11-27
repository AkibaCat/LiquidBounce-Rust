// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    auto_armor::{AutoArmor, CURSES},
    elytra_swap::ElytraSwap,
    inventory_cleaner::ArmorMode,
};
#[derive(Clone, Debug, PartialEq)]
pub enum AutoArmorSetting {
    Constraints(ElytraSwapSetting),
    Value(&'static str, Value),
}
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e.path.replace("ElytraSwap", "AutoArmor");
    e
}
impl AutoArmorSetting {
    pub fn apply(self, s: &mut AutoArmor) -> Result<(), ConfigError> {
        match self {
            Self::Constraints(v) => {
                let mut proxy = ElytraSwap {
                    constraints: s.constraints.clone(),
                    ..Default::default()
                };
                v.apply(&mut proxy).map_err(remap)?;
                s.constraints = proxy.constraints;
            }
            Self::Value(name, v) => {
                let path = format!("AutoArmor.{name}");
                match name {
                    "Mode" => {
                        s.mode = match scalar_string(&v).as_deref() {
                            Some("Smart") => ArmorMode::Smart,
                            Some("RawDefense") => ArmorMode::RawDefense,
                            _ => {
                                return Err(ConfigError::new(&path, "unknown case-sensitive mode"));
                            }
                        }
                    }
                    "Mode.RawDefense.ConsiderProjectileProtection" => {
                        s.projectile = parse_bool(&v, &path)?
                    }
                    "Hotbar.Enabled" => s.hotbar = parse_bool(&v, &path)?,
                    "Hotbar.CanSwapArmor" => s.can_swap = parse_bool(&v, &path)?,
                    "SaveArmor.Enabled" => s.save = parse_bool(&v, &path)?,
                    "SaveArmor.DurabilityThreshold" => s.threshold = parse_int(&v, &path)?,
                    "SaveArmor.AutoOpenInventory" => s.auto_open = parse_bool(&v, &path)?,
                    "SkipCursed" => {
                        s.skip_cursed = 0;
                        let entries = match &v {
                            Value::Array(a) => a.clone(),
                            _ => vec![v],
                        };
                        for entry in entries {
                            let tag = scalar_string(&entry)
                                .ok_or_else(|| ConfigError::new(&path, "expected curse tag"))?;
                            if let Some(i) =
                                CURSES.iter().position(|s| s.eq_ignore_ascii_case(&tag))
                            {
                                s.skip_cursed |= 1 << i;
                            }
                        }
                    }
                    _ => return Err(ConfigError::new(&path, "unknown setting")),
                }
            }
        }
        Ok(())
    }
}
fn rename(v: &mut Value) {
    match v {
        Value::String(s) => *s = s.replace("elytraSwap", "autoArmor"),
        Value::Array(a) => a.iter_mut().for_each(rename),
        Value::Object(o) => o.values_mut().for_each(rename),
        _ => {}
    }
}
pub fn serialize_auto_armor(
    s: &AutoArmor,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let key = |v: &str| format!("liquidbounce.module.autoArmor.{v}");
    let proxy = ElytraSwap {
        constraints: s.constraints.clone(),
        ..Default::default()
    };
    let mut constraints = serialize_elytra_swap(&proxy, c, f, &|k| {
        d(&k.replace("elytraSwap", "autoArmor"))
    })?["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Constraints")
        .unwrap()
        .clone();
    rename(&mut constraints);
    let mut values = t.common(ModuleKind::AutoArmor, c);
    values.push(constraints);
    let mut modes = Map::new();
    modes.insert("Smart".into(), t.group("Smart", "CONFIGURABLE", vec![]));
    modes.insert(
        "RawDefense".into(),
        t.group(
            "RawDefense",
            "CONFIGURABLE",
            vec![t.leaf(
                "ConsiderProjectileProtection",
                "BOOLEAN",
                json!(s.projectile),
                &key("mode.rawDefense.considerProjectileProtection"),
                None,
            )],
        ),
    );
    values.push(t.mode(
        if s.mode == ArmorMode::Smart {
            "Smart"
        } else {
            "RawDefense"
        },
        modes,
    ));
    values.push(t.group(
        "Hotbar",
        "TOGGLEABLE",
        vec![
            t.leaf("Enabled", "BOOLEAN", json!(s.hotbar), &key("hotbar"), None),
            t.leaf(
                "CanSwapArmor",
                "BOOLEAN",
                json!(s.can_swap),
                &key("hotbar.canSwapArmor"),
                None,
            ),
        ],
    ));
    values.push(t.group(
        "SaveArmor",
        "TOGGLEABLE",
        vec![
            t.leaf("Enabled", "BOOLEAN", json!(s.save), &key("saveArmor"), None),
            t.ranged(
                "DurabilityThreshold",
                "INT",
                json!(s.threshold),
                json!({"from":0,"to":100}),
                "",
                &key("saveArmor.durabilityThreshold"),
            ),
            t.leaf(
                "AutoOpenInventory",
                "BOOLEAN",
                json!(s.auto_open),
                &key("saveArmor.autoOpenInventory"),
                None,
            ),
        ],
    ));
    values.push(t.multi(
        "SkipCursed",
        u32::from(s.skip_cursed),
        &CURSES,
        &key("skipCursed"),
    ));
    Ok(t.group("AutoArmor", "TOGGLEABLE", values))
}
pub(super) fn parse_auto_armor(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let mut proxy = SettingsPatch {
        module: ModuleKind::ElytraSwap,
        changes: vec![],
        issues: vec![],
    };
    elytra_swap::parse_elytra_swap(&mut proxy, root);
    for change in proxy.changes {
        if let SettingChange::ElytraSwap(v) = change {
            p.changes
                .push(SettingChange::AutoArmor(AutoArmorSetting::Constraints(v)));
        }
    }
    p.issues.extend(proxy.issues.into_iter().map(remap));
    if let Some(mode) = root.get("Mode") {
        match mode.get("active").and_then(scalar_string) {
            Some(tag) if matches!(tag.as_str(), "Smart" | "RawDefense") => {
                p.changes
                    .push(SettingChange::AutoArmor(AutoArmorSetting::Value(
                        "Mode",
                        json!(tag),
                    )))
            }
            _ => p.issues.push(ConfigError::new(
                "AutoArmor.Mode.active",
                "missing or unknown case-sensitive mode",
            )),
        }
        if let Some(choices) = mode.get("choices").and_then(Value::as_object) {
            for tag in ["Smart", "RawDefense"] {
                let path = format!("AutoArmor.Mode.{tag}");
                match choices.get(tag) {
                    Some(group) => match nodes(group, tag, &path) {
                        Ok(n) => {
                            if tag == "RawDefense" {
                                leaf(
                                    p,
                                    &n,
                                    "ConsiderProjectileProtection",
                                    &format!("{path}.ConsiderProjectileProtection"),
                                    |v, _| {
                                        Ok(SettingChange::AutoArmor(AutoArmorSetting::Value(
                                            "Mode.RawDefense.ConsiderProjectileProtection",
                                            v.clone(),
                                        )))
                                    },
                                );
                            }
                        }
                        Err(e) => p.issues.push(e),
                    },
                    None => p.issues.push(ConfigError::new(&path, "choice not found")),
                }
            }
        } else {
            p.issues.push(ConfigError::new(
                "AutoArmor.Mode.choices",
                "expected choices object",
            ));
        }
        if let Err(e) = nodes(mode, "Mode", "AutoArmor.Mode") {
            p.issues.push(e);
        }
    }
    for (group, settings) in [
        (
            "Hotbar",
            &[
                ("Enabled", "Hotbar.Enabled"),
                ("CanSwapArmor", "Hotbar.CanSwapArmor"),
            ][..],
        ),
        (
            "SaveArmor",
            &[
                ("Enabled", "SaveArmor.Enabled"),
                ("DurabilityThreshold", "SaveArmor.DurabilityThreshold"),
                ("AutoOpenInventory", "SaveArmor.AutoOpenInventory"),
            ][..],
        ),
    ] {
        if let Some(value) = root.get(group) {
            match nodes(value, group, &format!("AutoArmor.{group}")) {
                Ok(n) => {
                    for &(name, path) in settings {
                        leaf(p, &n, name, &format!("AutoArmor.{path}"), |v, _| {
                            Ok(SettingChange::AutoArmor(AutoArmorSetting::Value(
                                path,
                                v.clone(),
                            )))
                        });
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
    leaf(p, root, "SkipCursed", "AutoArmor.SkipCursed", |v, _| {
        Ok(SettingChange::AutoArmor(AutoArmorSetting::Value(
            "SkipCursed",
            v.clone(),
        )))
    });
}
