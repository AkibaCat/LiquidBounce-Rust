// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::auto_pearl_runtime::AutoPearlRuntime;
use liquidbounce_behaviors::{auto_pearl::Mode, no_rotate_set::NoRotateSet};
#[derive(Clone, Debug, PartialEq)]
pub enum AutoPearlSetting {
    Value(String, Value),
    Rotation(NoRotateSetSetting),
}
impl AutoPearlSetting {
    pub fn apply(self, s: &mut AutoPearlRuntime) -> Result<(), ConfigError> {
        if let Self::Rotation(v) = self {
            let mut proxy = NoRotateSet {
                rotations: std::mem::take(&mut s.rotations),
                ..Default::default()
            };
            v.apply(&mut proxy);
            s.rotations = proxy.rotations;
            return Ok(());
        }
        let Self::Value(path, v) = self else {
            unreachable!()
        };
        let p = format!("AutoPearl.{path}");
        let s = &mut s.behavior.settings;
        match path.as_str() {
            "Rotate.Enabled" => s.rotate = parse_bool(&v, &p)?,
            "Mode" => {
                s.mode = scalar_string(&v)
                    .and_then(|n| Mode::parse(&n))
                    .ok_or_else(|| ConfigError::new(&p, "expected Trigger or Target"))?
            }
            "Limits.Enabled" => s.limits = parse_bool(&v, &p)?,
            "Limits.Angle" => s.angle = parse_int(&v, &p)?,
            "Limits.MinDistance" => s.min_distance = parse_float(&v, &p)?,
            "Limits.DestinationDistance" => s.destination_distance = parse_float(&v, &p)?,
            "CombatPauseTime" => s.combat_pause = parse_int(&v, &p)?,
            "SlotResetDelay" => {
                let r = parse_int_pair(&v, &p)?;
                if r[0] > r[1] {
                    return Err(ConfigError::new(&p, "empty slot reset range"));
                }
                s.slot_reset = r;
            }
            _ => return Err(ConfigError::new(&p, "unknown AutoPearl setting")),
        }
        Ok(())
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.autoPearl.rotate.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "AutoPearl.Rotate.Rotations",
        )
        .replace(OLD, NEW);
    e
}
fn keys(v: &mut Value) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(OLD, NEW);
            }
            for v in o.values_mut() {
                keys(v);
            }
        }
        Value::Array(a) => {
            for v in a {
                keys(v);
            }
        }
        _ => {}
    }
}
pub fn serialize_auto_pearl(
    s: &AutoPearlRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::AutoPearl, c);
    let b = &s.behavior.settings;
    let key = |p: &str| format!("liquidbounce.module.autoPearl.{p}");
    let mut mode = t.leaf("Mode", "CHOOSE", json!(b.mode.name()), &key("mode"), None);
    if f == JsonFormat::Interop {
        mode["choices"] = json!(["Trigger", "Target"]);
    }
    values.push(mode);
    let describe = |k: &str| d(&k.replace(OLD, NEW));
    let proxy = NoRotateSet {
        rotations: s.rotations.clone(),
        ..Default::default()
    };
    let mut tree = serialize_no_rotate_set(&proxy, c, f, &describe).map_err(remap)?;
    let mut rotations = tree["value"]
        .as_array_mut()
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|v| v.first_mut())
        .map(Value::take)
        .ok_or_else(|| {
            ConfigError::new("AutoPearl.Rotate.Rotations", "shared rotation tree changed")
        })?;
    keys(&mut rotations);
    values.push(t.group(
        "Rotate",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(b.rotate),
                &key("rotate.enabled"),
                None,
            ),
            rotations,
        ],
    ));
    values.push(t.group(
        "Limits",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(b.limits),
                &key("limits.enabled"),
                None,
            ),
            t.ranged(
                "Angle",
                "INT",
                json!(b.angle),
                json!({"from":0,"to":180}),
                "°",
                &key("limits.angle"),
            ),
            t.ranged(
                "MinDistance",
                "FLOAT",
                float_json(b.min_distance, "AutoPearl.Limits.MinDistance")?,
                json!({"from":0.,"to":10.}),
                "m",
                &key("limits.minDistance"),
            ),
            t.ranged(
                "DestinationDistance",
                "FLOAT",
                float_json(
                    b.destination_distance,
                    "AutoPearl.Limits.DestinationDistance",
                )?,
                json!({"from":0.,"to":30.}),
                "m",
                &key("limits.destinationDistance"),
            ),
        ],
    ));
    values.push(t.ranged(
        "CombatPauseTime",
        "INT",
        json!(b.combat_pause),
        json!({"from":0,"to":40}),
        "ticks",
        &key("combatPauseTime"),
    ));
    values.push(t.ranged(
        "SlotResetDelay",
        "INT_RANGE",
        json!({"from":b.slot_reset[0],"to":b.slot_reset[1]}),
        json!({"from":0,"to":40}),
        "ticks",
        &key("slotResetDelay"),
    ));
    Ok(t.group("AutoPearl", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
    leaf(p, n, name, &format!("AutoPearl.{path}"), |v, _| {
        let s = AutoPearlSetting::Value(path.into(), v.clone());
        s.clone().apply(&mut AutoPearlRuntime::default())?;
        Ok(SettingChange::AutoPearl(s))
    });
}
pub(super) fn parse_auto_pearl(p: &mut SettingsPatch, root: &Nodes<'_>) {
    add(p, root, "Mode", "Mode");
    for (group, names) in [
        ("Rotate", vec!["Enabled"]),
        (
            "Limits",
            vec!["Enabled", "Angle", "MinDistance", "DestinationDistance"],
        ),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("AutoPearl.{group}")) {
                Ok(n) => {
                    for name in names {
                        add(p, &n, name, &format!("{group}.{name}"));
                    }
                    if group == "Rotate" {
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
                                if let SettingChange::NoRotateSet(v) = c {
                                    if !matches!(v, NoRotateSetSetting::Mode(_)) {
                                        p.changes.push(SettingChange::AutoPearl(
                                            AutoPearlSetting::Rotation(v),
                                        ));
                                    }
                                }
                            }
                            p.issues.extend(parsed.issues.into_iter().map(remap));
                        }
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
    for name in ["CombatPauseTime", "SlotResetDelay"] {
        add(p, root, name, name);
    }
}
