// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::auto_deposit_runtime::AutoDepositRuntime;
use liquidbounce_behaviors::{
    auto_deposit::{PAUSES, SWINGS},
    no_rotate_set::NoRotateSet,
};
#[derive(Clone, Debug, PartialEq)]
pub enum AutoDepositSetting {
    Inventory(ChestCleanerSetting),
    Value(&'static str, Value),
    Rotation(NoRotateSetSetting),
}
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e.path.replace("ChestCleaner", "AutoDeposit").replace(
        "NoRotateSet.Mode.ResetRotation.Rotations",
        "AutoDeposit.PunchToDeposit.Rotations",
    );
    e
}
impl AutoDepositSetting {
    pub fn apply(
        self,
        s: &mut AutoDepositRuntime,
        items: &[String],
        blocks: &[String],
    ) -> Result<(), ConfigError> {
        match self {
            Self::Inventory(v) => v.apply(&mut s.settings.inventory, items).map_err(remap)?,
            Self::Rotation(v) => {
                let mut proxy = NoRotateSet {
                    rotations: std::mem::take(&mut s.rotations),
                    ..Default::default()
                };
                v.apply(&mut proxy);
                s.rotations = proxy.rotations;
            }
            Self::Value(path, v) => {
                let full = format!("AutoDeposit.PunchToDeposit.{path}");
                let p = &mut s.settings.punch;
                match path {
                    "Enabled" => {
                        let enabled = parse_bool(&v, &full)?;
                        if p.enabled && !enabled {
                            s.target = None;
                            s.waiting = false;
                            s.wait = 0;
                            s.callback = s.callback.wrapping_add(1);
                        }
                        p.enabled = enabled;
                    }
                    "Range" => p.range = parse_float(&v, &full)?,
                    "WallRange" => p.wall_range = parse_float(&v, &full)?.min(p.range),
                    "Delay" => p.delay = parse_int(&v, &full)?,
                    "RotateToTarget" => p.rotate = parse_bool(&v, &full)?,
                    "SwingMode" => {
                        let tag = scalar_string(&v)
                            .ok_or_else(|| ConfigError::new(&full, "expected choice"))?;
                        p.swing = SWINGS
                            .iter()
                            .position(|s| s.eq_ignore_ascii_case(&tag))
                            .ok_or_else(|| ConfigError::new(&full, "unknown swing mode"))?;
                    }
                    "PauseOn" => {
                        p.pauses = 0;
                        let entries = match &v {
                            Value::Array(a) => a.clone(),
                            _ => vec![v],
                        };
                        for v in entries {
                            let tag = scalar_string(&v)
                                .ok_or_else(|| ConfigError::new(&full, "expected enum entry"))?;
                            if let Some(i) =
                                PAUSES.iter().position(|s| s.eq_ignore_ascii_case(&tag))
                            {
                                p.pauses |= 1 << i;
                            }
                        }
                    }
                    "ValidStorageBlocks" => {
                        p.blocks.clear();
                        let entries = match &v {
                            Value::Array(a) => a.clone(),
                            _ => vec![v],
                        };
                        for v in entries {
                            let raw = match v {
                                Value::String(s) => s,
                                Value::Number(n) => n.to_string(),
                                _ => {
                                    return Err(ConfigError::new(&full, "expected registry entry"));
                                }
                            };
                            let id = if raw.starts_with(':') {
                                format!("minecraft{raw}")
                            } else if raw.contains(':') {
                                raw
                            } else {
                                format!("minecraft:{raw}")
                            };
                            p.blocks.insert(if blocks.contains(&id) {
                                id
                            } else {
                                "minecraft:air".into()
                            });
                        }
                    }
                    _ => return Err(ConfigError::new(&full, "unknown setting")),
                }
            }
        }
        Ok(())
    }
}
fn rename(v: &mut Value) {
    match v {
        Value::String(s) => {
            *s = s
                .replace("ChestCleaner", "AutoDeposit")
                .replace("chestCleaner", "autoDeposit")
                .replace(
                    "liquidbounce.module.noRotateSet.mode.resetRotation.rotations",
                    "liquidbounce.module.autoDeposit.punchToDeposit.rotations",
                )
        }
        Value::Array(a) => a.iter_mut().for_each(rename),
        Value::Object(o) => o.values_mut().for_each(rename),
        _ => {}
    }
}
pub fn serialize_auto_deposit(
    s: &AutoDepositRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let mut tree = serialize_chest_cleaner(&s.settings.inventory, c, f, &|k| {
        d(&k.replace("chestCleaner", "autoDeposit"))
    })
    .map_err(remap)?;
    rename(&mut tree);
    let t = Tree {
        format: f,
        describe: d,
    };
    let p = &s.settings.punch;
    let key = |v: &str| format!("liquidbounce.module.autoDeposit.punchToDeposit.{v}");
    let mut swing = t.leaf(
        "SwingMode",
        "CHOOSE",
        json!(SWINGS[p.swing]),
        &key("swingMode"),
        None,
    );
    if f == JsonFormat::Interop {
        swing["choices"] = json!(SWINGS);
    }
    let mut blocks: Vec<_> = p.blocks.iter().collect();
    blocks.sort_by(|a, b| {
        let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
        let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
        ap.cmp(bp).then_with(|| an.cmp(bn))
    });
    let mut block = t.leaf(
        "ValidStorageBlocks",
        "REGISTRY_LIST",
        json!(blocks),
        &key("validStorageBlocks"),
        None,
    );
    if f == JsonFormat::Interop {
        block["registry"] = json!("block");
    }
    let proxy = NoRotateSet {
        rotations: s.rotations.clone(),
        ..Default::default()
    };
    let mut rotation = serialize_no_rotate_set(&proxy, c, f, &|k| {
        d(&k.replace(
            "liquidbounce.module.noRotateSet.mode.resetRotation.rotations",
            "liquidbounce.module.autoDeposit.punchToDeposit.rotations",
        ))
    })?;
    let mut rotations = rotation["value"]
        .as_array_mut()
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|v| v.first_mut())
        .map(Value::take)
        .ok_or_else(|| {
            ConfigError::new(
                "AutoDeposit.PunchToDeposit.Rotations",
                "shared rotation tree changed",
            )
        })?;
    rename(&mut rotations);
    let punch = t.group(
        "PunchToDeposit",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(p.enabled),
                &key("enabled"),
                None,
            ),
            t.ranged(
                "Range",
                "FLOAT",
                float_json(p.range, "AutoDeposit.PunchToDeposit.Range")?,
                json!({"from":1.,"to":6.}),
                "",
                &key("range"),
            ),
            t.ranged(
                "WallRange",
                "FLOAT",
                float_json(p.wall_range, "AutoDeposit.PunchToDeposit.WallRange")?,
                json!({"from":0.,"to":6.}),
                "",
                &key("wallRange"),
            ),
            t.ranged(
                "Delay",
                "INT",
                json!(p.delay),
                json!({"from":1,"to":80}),
                "ticks",
                &key("delay"),
            ),
            swing,
            t.multi("PauseOn", p.pauses, &PAUSES, &key("pauseOn")),
            block,
            t.leaf(
                "RotateToTarget",
                "BOOLEAN",
                json!(p.rotate),
                &key("rotateToTarget"),
                None,
            ),
            rotations,
        ],
    );
    tree["value"].as_array_mut().unwrap().push(punch);
    Ok(tree)
}
pub(super) fn parse_auto_deposit(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let alias = root.get("CheckTitle").map(|v| {
        let mut v = (*v).clone();
        v["name"] = json!("CheckScreenTitle");
        v
    });
    let mut roots = root.clone();
    if !roots.contains_key("CheckScreenTitle")
        && let Some(alias) = alias.as_ref()
    {
        roots.insert("CheckScreenTitle".into(), alias);
    }
    let root = &roots;
    let mut proxy = SettingsPatch {
        module: ModuleKind::ChestCleaner,
        changes: vec![],
        issues: vec![],
    };
    chest_cleaner::parse_chest_cleaner(&mut proxy, root);
    for c in proxy.changes {
        if let SettingChange::ChestCleaner(c) = c {
            p.changes
                .push(SettingChange::AutoDeposit(AutoDepositSetting::Inventory(c)));
        }
    }
    p.issues.extend(proxy.issues.into_iter().map(remap));
    let Some(v) = root.get("PunchToDeposit") else {
        return;
    };
    let n = match nodes(v, "PunchToDeposit", "AutoDeposit.PunchToDeposit") {
        Ok(n) => n,
        Err(e) => {
            p.issues.push(e);
            return;
        }
    };
    for name in [
        "Enabled",
        "Range",
        "WallRange",
        "Delay",
        "SwingMode",
        "PauseOn",
        "ValidStorageBlocks",
        "RotateToTarget",
    ] {
        leaf(
            p,
            &n,
            name,
            &format!("AutoDeposit.PunchToDeposit.{name}"),
            |v, _| {
                Ok(SettingChange::AutoDeposit(AutoDepositSetting::Value(
                    name,
                    v.clone(),
                )))
            },
        );
    }
    if let Some(v) = n.get("Rotations") {
        let proxy = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[v]}}}]});
        let root = nodes(&proxy, "NoRotateSet", "NoRotateSet").unwrap();
        let mut parsed = SettingsPatch {
            module: ModuleKind::NoRotateSet,
            changes: vec![],
            issues: vec![],
        };
        no_rotate_set::parse_no_rotate_set(&mut parsed, &root);
        for c in parsed.changes {
            if let SettingChange::NoRotateSet(v) = c
                && !matches!(v, NoRotateSetSetting::Mode(_))
            {
                p.changes
                    .push(SettingChange::AutoDeposit(AutoDepositSetting::Rotation(v)));
            }
        }
        p.issues.extend(parsed.issues.into_iter().map(remap));
    }
}
