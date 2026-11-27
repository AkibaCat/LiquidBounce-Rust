// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::easy_pearl_runtime::EasyPearlRuntime;
use liquidbounce_behaviors::no_rotate_set::NoRotateSet;
#[derive(Clone, Debug, PartialEq)]
pub enum EasyPearlSetting {
    Value(String, Value),
    Rotation(NoRotateSetSetting),
}
impl EasyPearlSetting {
    pub fn apply(self, s: &mut EasyPearlRuntime) -> Result<(), ConfigError> {
        match self {
            Self::Rotation(v) => {
                let mut proxy = NoRotateSet {
                    rotations: std::mem::take(&mut s.rotations),
                    ..Default::default()
                };
                v.apply(&mut proxy);
                s.rotations = proxy.rotations;
            }
            Self::Value(path, v) => {
                let p = format!("EasyPearl.{path}");
                match path.as_str() {
                    "AimOffThreshold" => {
                        s.behavior.settings.aim_off_threshold = parse_float(&v, &p)?
                    }
                    "ReachableCheck" => s.behavior.settings.reachable_check = parse_bool(&v, &p)?,
                    "ShowDistance" => s.behavior.settings.show_distance = parse_bool(&v, &p)?,
                    _ => return Err(ConfigError::new(&p, "unknown EasyPearl setting")),
                }
            }
        }
        Ok(())
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.easyPearl.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "EasyPearl.Rotations",
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
pub fn serialize_easy_pearl(
    s: &EasyPearlRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let b = &s.behavior.settings;
    let key = |p: &str| format!("liquidbounce.module.easyPearl.{p}");
    let mut values = t.common(ModuleKind::EasyPearl, c);
    values.push(t.ranged(
        "AimOffThreshold",
        "FLOAT",
        float_json(b.aim_off_threshold, "EasyPearl.AimOffThreshold")?,
        json!({"from":0.5,"to":10.}),
        "",
        &key("aimOffThreshold"),
    ));
    values.push(t.leaf(
        "ReachableCheck",
        "BOOLEAN",
        json!(b.reachable_check),
        &key("reachableCheck"),
        None,
    ));
    let proxy = NoRotateSet {
        rotations: s.rotations.clone(),
        ..Default::default()
    };
    let mut tree =
        serialize_no_rotate_set(&proxy, c, f, &|k| d(&k.replace(OLD, NEW))).map_err(remap)?;
    let mut rotations = tree["value"]
        .as_array_mut()
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|v| v.first_mut())
        .map(Value::take)
        .ok_or_else(|| ConfigError::new("EasyPearl.Rotations", "shared rotation tree changed"))?;
    keys(&mut rotations);
    values.push(rotations);
    values.push(t.leaf(
        "ShowDistance",
        "BOOLEAN",
        json!(b.show_distance),
        &key("showDistance"),
        None,
    ));
    Ok(t.group("EasyPearl", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str) {
    leaf(p, n, name, &format!("EasyPearl.{name}"), |v, _| {
        let s = EasyPearlSetting::Value(name.into(), v.clone());
        s.clone().apply(&mut EasyPearlRuntime::default())?;
        Ok(SettingChange::EasyPearl(s))
    });
}
pub(super) fn parse_easy_pearl(p: &mut SettingsPatch, root: &Nodes<'_>) {
    add(p, root, "AimOffThreshold");
    add(p, root, "ReachableCheck");
    if let Some(v) = root.get("Rotations") {
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
                    p.changes
                        .push(SettingChange::EasyPearl(EasyPearlSetting::Rotation(v)));
                }
            }
        }
        p.issues.extend(parsed.issues.into_iter().map(remap));
    }
    add(p, root, "ShowDistance");
}
