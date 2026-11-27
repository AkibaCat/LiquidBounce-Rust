// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::auto_wind_charge_runtime::AutoWindChargeRuntime;
use liquidbounce_behaviors::{
    camera_clip::ModifierKey,
    no_rotate_set::{FloatRange, NoRotateSet},
};
#[derive(Clone, Debug, PartialEq)]
pub enum AutoWindChargeSetting {
    Value(String, Value),
    Rotation(NoRotateSetSetting),
}
impl AutoWindChargeSetting {
    pub fn apply(self, s: &mut AutoWindChargeRuntime) -> Result<(), ConfigError> {
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
        let p = format!("AutoWindCharge.{path}");
        let s = &mut s.behavior.settings;
        match path.as_str() {
            "Rotate.Enabled" => s.rotate = parse_bool(&v, &p)?,
            "HorizontalBoost.Enabled" => s.horizontal_boost = parse_bool(&v, &p)?,
            "HorizontalBoost.Pitch" => {
                let r = parse_float_pair(&v, &p)?;
                s.pitch = FloatRange(r[0], r[1]);
            }
            "HorizontalBoost.Key" => {
                let name =
                    scalar_string(&v).ok_or_else(|| ConfigError::new(&p, "expected key name"))?;
                s.key = ModifierKey::from_name(&name)
                    .ok_or_else(|| ConfigError::new(&p, "unknown InputConstants key"))?
                    .name;
            }
            "CombatPauseTime" => s.combat_pause = parse_int(&v, &p)?,
            "SlotResetDelay" => {
                let r = parse_int_pair(&v, &p)?;
                if r[0] > r[1] {
                    return Err(ConfigError::new(&p, "empty slot reset range"));
                }
                s.slot_reset = r;
            }
            _ => return Err(ConfigError::new(&p, "unknown AutoWindCharge setting")),
        }
        Ok(())
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.autoWindCharge.rotate.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "AutoWindCharge.Rotate.Rotations",
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
pub fn serialize_auto_wind_charge(
    s: &AutoWindChargeRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::AutoWindCharge, c);
    let b = &s.behavior.settings;
    let key = |p: &str| format!("liquidbounce.module.autoWindCharge.{p}");
    values.push(t.group(
        "HorizontalBoost",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(b.horizontal_boost),
                &key("horizontalBoost.enabled"),
                None,
            ),
            t.ranged(
                "Pitch",
                "FLOAT_RANGE",
                json!({"from":b.pitch.0,"to":b.pitch.1}),
                json!({"from":0.,"to":90.}),
                "",
                &key("horizontalBoost.pitch"),
            ),
            t.leaf(
                "Key",
                "KEY",
                json!(b.key),
                &key("horizontalBoost.key"),
                None,
            ),
        ],
    ));
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
            ConfigError::new(
                "AutoWindCharge.Rotate.Rotations",
                "shared rotation tree changed",
            )
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
    Ok(t.group("AutoWindCharge", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
    leaf(p, n, name, &format!("AutoWindCharge.{path}"), |v, _| {
        let s = AutoWindChargeSetting::Value(path.into(), v.clone());
        s.clone().apply(&mut AutoWindChargeRuntime::default())?;
        Ok(SettingChange::AutoWindCharge(s))
    });
}
pub(super) fn parse_auto_wind_charge(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["CombatPauseTime", "SlotResetDelay"] {
        add(p, root, name, name);
    }
    for (group, names) in [
        ("HorizontalBoost", vec!["Enabled", "Pitch", "Key"]),
        ("Rotate", vec!["Enabled"]),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("AutoWindCharge.{group}")) {
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
                                        p.changes.push(SettingChange::AutoWindCharge(
                                            AutoWindChargeSetting::Rotation(v),
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
}
