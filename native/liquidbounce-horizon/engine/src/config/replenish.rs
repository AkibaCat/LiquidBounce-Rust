// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    elytra_swap::ElytraSwap,
    replenish::{FEATURES, INSIDE_OF, Replenish},
};
#[derive(Clone, Debug, PartialEq)]
pub enum ReplenishSetting {
    Constraints(ElytraSwapSetting),
    Value(&'static str, Value),
}
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e.path.replace("ElytraSwap", "Replenish");
    e
}
fn multi(v: &Value, choices: &[&str], path: &str) -> Result<u32, ConfigError> {
    let entries = match v {
        Value::Array(a) => a.clone(),
        _ => vec![v.clone()],
    };
    let mut result = 0;
    for v in entries {
        let tag = scalar_string(&v).ok_or_else(|| ConfigError::new(path, "expected enum entry"))?;
        if let Some(i) = choices.iter().position(|s| s.eq_ignore_ascii_case(&tag)) {
            result |= 1 << i;
        }
    }
    Ok(result)
}
impl ReplenishSetting {
    pub fn apply(self, s: &mut Replenish) -> Result<(), ConfigError> {
        match self {
            Self::Constraints(v) => {
                let mut p = ElytraSwap {
                    constraints: s.constraints.clone(),
                    ..Default::default()
                };
                v.apply(&mut p).map_err(remap)?;
                s.constraints = p.constraints;
            }
            Self::Value(name, v) => {
                let path = format!("Replenish.{name}");
                match name {
                    "ItemThreshold" => s.threshold = parse_int(&v, &path)?,
                    "Delay" => s.delay = parse_int(&v, &path)?,
                    "ReplenishEmpty" => s.replenish_empty = parse_bool(&v, &path)?,
                    "Features" => s.features = multi(&v, &FEATURES, &path)?,
                    "InsideOf" => s.inside_of = multi(&v, &INSIDE_OF, &path)?,
                    _ => return Err(ConfigError::new(&path, "unknown setting")),
                }
            }
        }
        Ok(())
    }
}
fn rename(v: &mut Value) {
    match v {
        Value::String(s) => *s = s.replace("elytraSwap", "replenish"),
        Value::Array(a) => a.iter_mut().for_each(rename),
        Value::Object(o) => o.values_mut().for_each(rename),
        _ => {}
    }
}
pub fn serialize_replenish(
    s: &Replenish,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let key = |v: &str| format!("liquidbounce.module.replenish.{v}");
    let p = ElytraSwap {
        constraints: s.constraints.clone(),
        ..Default::default()
    };
    let mut constraints =
        serialize_elytra_swap(&p, c, f, &|k| d(&k.replace("elytraSwap", "replenish")))?["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == "Constraints")
            .unwrap()
            .clone();
    rename(&mut constraints);
    let mut values = t.common(ModuleKind::Replenish, c);
    values.push(constraints);
    values.push(t.ranged(
        "ItemThreshold",
        "INT",
        json!(s.threshold),
        json!({"from":0,"to":63}),
        "",
        &key("itemThreshold"),
    ));
    values.push(t.ranged(
        "Delay",
        "INT",
        json!(s.delay),
        json!({"from":0,"to":1000}),
        "ms",
        &key("delay"),
    ));
    values.push(t.leaf(
        "ReplenishEmpty",
        "BOOLEAN",
        json!(s.replenish_empty),
        &key("replenishEmpty"),
        None,
    ));
    values.push(t.multi("Features", s.features, &FEATURES, &key("features")));
    values.push(t.multi("InsideOf", s.inside_of, &INSIDE_OF, &key("insideOf")));
    Ok(t.group("Replenish", "TOGGLEABLE", values))
}
pub(super) fn parse_replenish(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let mut proxy = SettingsPatch {
        module: ModuleKind::ElytraSwap,
        changes: vec![],
        issues: vec![],
    };
    elytra_swap::parse_elytra_swap(&mut proxy, root);
    for c in proxy.changes {
        if let SettingChange::ElytraSwap(c) = c {
            p.changes
                .push(SettingChange::Replenish(ReplenishSetting::Constraints(c)));
        }
    }
    p.issues.extend(proxy.issues.into_iter().map(remap));
    for name in [
        "ItemThreshold",
        "Delay",
        "ReplenishEmpty",
        "Features",
        "InsideOf",
    ] {
        leaf(p, root, name, &format!("Replenish.{name}"), |v, _| {
            Ok(SettingChange::Replenish(ReplenishSetting::Value(
                name,
                v.clone(),
            )))
        });
    }
}
