// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::extinguish_runtime::ExtinguishRuntime;
use liquidbounce_behaviors::no_rotate_set::NoRotateSet;
#[derive(Clone, Debug, PartialEq)]
pub enum ExtinguishSetting {
    Value(String, Value),
    Rotation(NoRotateSetSetting),
}
impl ExtinguishSetting {
    pub fn apply(self, r: &mut ExtinguishRuntime) -> Result<(), ConfigError> {
        match self {
            Self::Rotation(v) => {
                let mut proxy = NoRotateSet {
                    rotations: std::mem::take(&mut r.rotations),
                    ..Default::default()
                };
                v.apply(&mut proxy);
                r.rotations = proxy.rotations;
            }
            Self::Value(path, v) => {
                let p = format!("Extinguish.{path}");
                let s = &mut r.behavior.settings;
                match path.as_str() {
                    "Cooldown" => s.cooldown = parse_float(&v, &p)?,
                    "NotDuringCombat" => s.not_during_combat = parse_bool(&v, &p)?,
                    "Pickup.Enabled" => s.pickup = parse_bool(&v, &p)?,
                    "Pickup.PickupSpan" => {
                        let span = parse_float_pair(&v, &p)?;
                        if span[0] > span[1] {
                            return Err(ConfigError::new(&p, "empty pickup time range"));
                        }
                        s.pickup_span = span;
                    }
                    _ => return Err(ConfigError::new(&p, "unknown Extinguish setting")),
                }
            }
        }
        Ok(())
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.extinguish.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "Extinguish.Rotations",
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
pub fn serialize_extinguish(
    r: &ExtinguishRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = &r.behavior.settings;
    let key = |p: &str| format!("liquidbounce.module.extinguish.{p}");
    let mut values = t.common(ModuleKind::Extinguish, c);
    values.push(t.ranged(
        "Cooldown",
        "FLOAT",
        float_json(s.cooldown, "Extinguish.Cooldown")?,
        json!({"from":0.,"to":20.}),
        "s",
        &key("cooldown"),
    ));
    values.push(t.leaf(
        "NotDuringCombat",
        "BOOLEAN",
        json!(s.not_during_combat),
        &key("notDuringCombat"),
        None,
    ));
    values.push(t.group("Pickup","TOGGLEABLE",vec![
        t.leaf("Enabled","BOOLEAN",json!(s.pickup),&key("pickup.enabled"),None),
        t.ranged("PickupSpan","FLOAT_RANGE",json!({"from":float_json(s.pickup_span[0],"Extinguish.Pickup.PickupSpan")?,"to":float_json(s.pickup_span[1],"Extinguish.Pickup.PickupSpan")?}),json!({"from":0.,"to":20.}),"s",&key("pickup.pickupSpan")),
    ]));
    let describe = |k: &str| d(&k.replace(OLD, NEW));
    let proxy = NoRotateSet {
        rotations: r.rotations.clone(),
        ..Default::default()
    };
    let mut tree = serialize_no_rotate_set(&proxy, c, f, &describe).map_err(remap)?;
    let mut rotations = tree["value"]
        .as_array_mut()
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|v| v.first_mut())
        .map(Value::take)
        .ok_or_else(|| ConfigError::new("Extinguish.Rotations", "shared rotation tree changed"))?;
    keys(&mut rotations);
    values.push(rotations);
    Ok(t.group("Extinguish", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
    leaf(p, n, name, &format!("Extinguish.{path}"), |v, _| {
        let s = ExtinguishSetting::Value(path.into(), v.clone());
        s.clone().apply(&mut ExtinguishRuntime::default())?;
        Ok(SettingChange::Extinguish(s))
    });
}
pub(super) fn parse_extinguish(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["Cooldown", "NotDuringCombat"] {
        add(p, root, name, name);
    }
    if let Some(v) = root.get("Pickup") {
        match nodes(v, "Pickup", "Extinguish.Pickup") {
            Ok(n) => {
                for name in ["Enabled", "PickupSpan"] {
                    add(p, &n, name, &format!("Pickup.{name}"));
                }
            }
            Err(e) => p.issues.push(e),
        }
    }
    if let Some(v) = root.get("Rotations") {
        let tree = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[v]}}}]});
        let root = nodes(&tree, "NoRotateSet", "NoRotateSet").unwrap();
        let mut parsed = SettingsPatch {
            module: ModuleKind::NoRotateSet,
            changes: vec![],
            issues: vec![],
        };
        no_rotate_set::parse_no_rotate_set(&mut parsed, &root);
        for c in parsed.changes {
            if let SettingChange::NoRotateSet(v) = c {
                if !matches!(v, NoRotateSetSetting::Mode(_)) {
                    p.changes
                        .push(SettingChange::Extinguish(ExtinguishSetting::Rotation(v)));
                }
            }
        }
        p.issues.extend(parsed.issues.into_iter().map(remap));
    }
}
