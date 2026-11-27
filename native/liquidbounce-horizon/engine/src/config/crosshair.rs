// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::crosshair::{Crosshair, Field, MODES, SOURCES, fields};
#[derive(Clone, Debug, PartialEq)]
pub struct CrosshairSetting(pub String, pub Value);
impl CrosshairSetting {
    pub fn apply(self, s: &mut Crosshair) {
        s.set(self.0, self.1);
    }
}
pub fn crosshair_key(path: &str) -> String {
    format!(
        "liquidbounce.module.crosshair.{}",
        path.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn group(
    t: &Tree,
    s: &Crosshair,
    path: &str,
    name: &str,
    toggle: bool,
) -> Result<Value, ConfigError> {
    let schema = fields();
    let prefix = format!("{path}.");
    let mut values = vec![];
    for f in schema
        .iter()
        .filter(|f| f.path.starts_with(&prefix) && !f.path[prefix.len()..].contains('.'))
    {
        let name = &f.path[prefix.len()..];
        let k = crosshair_key(&f.path);
        let v = s.values[&f.path].clone();
        values.push(if let Some([a, b]) = f.range {
            t.ranged(name, f.kind, v, json!({"from":a,"to":b}), f.suffix, &k)
        } else if f.kind == "MULTI_CHOOSE" {
            let mut leaf = t.leaf(name, f.kind, v, &k, None);
            if t.format == JsonFormat::Interop {
                leaf["choices"] = json!(SOURCES);
                leaf["canBeNone"] = json!(true);
                leaf["isOrderSensitive"] = json!(false);
            }
            leaf
        } else {
            t.leaf(name, f.kind, v, &k, None)
        });
    }
    for (child, toggle) in match path {
        "Mode.Circle" => vec![("Radius", false), ("Color", false)],
        "Mode.CS2" => vec![("Crosshair", false), ("Color", false)],
        "CooldownBar" => vec![("ReadyFlash", true)],
        _ => vec![],
    } {
        values.push(group(t, s, &format!("{path}.{child}"), child, toggle)?);
    }
    let mut out = t.group(
        name,
        if toggle { "TOGGLEABLE" } else { "CONFIGURABLE" },
        values,
    );
    if t.format == JsonFormat::Interop {
        out["key"] = json!(crosshair_key(path));
        out["description"] = json!((t.describe)(&format!(
            "{}.description",
            crosshair_key(path)
        )));
    }
    Ok(out)
}
pub fn serialize_crosshair(
    s: &Crosshair,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut out = t.common(ModuleKind::Crosshair, common);
    let mut choices = Map::new();
    for mode in MODES {
        choices.insert(
            mode.into(),
            group(&t, s, &format!("Mode.{mode}"), mode, false)?,
        );
    }
    let mut mode = t.mode(s.mode(), choices);
    if format == JsonFormat::Interop {
        mode["key"] = json!(crosshair_key("Mode"));
        mode["description"] = json!(describe(&format!("{}.description", crosshair_key("Mode"))));
    }
    out.push(mode);
    out.push(group(&t, s, "CooldownBar", "CooldownBar", true)?);
    out.push(group(&t, s, "HitFlash", "HitFlash", true)?);
    Ok(t.group("Crosshair", "TOGGLEABLE", out))
}
fn parse_value(v: &Value, f: &Field) -> Result<Value, ConfigError> {
    let p = format!("Crosshair.{}", f.path);
    Ok(match f.kind {
        "BOOLEAN" => json!(parse_bool(v, &p)?),
        "FLOAT" => float_json(parse_float(v, &p)?, &p)?,
        "INT" => json!(parse_int(v, &p)?),
        "COLOR" => {
            let n = if let Some(s) = v.as_str() {
                let s = s.strip_prefix('#').unwrap_or(s);
                if !matches!(s.len(), 6 | 8) {
                    return Err(ConfigError::new(&p, "expected 6/8 hex digits"));
                }
                i64::from_str_radix(s, 16).ok().map(|n| {
                    n as i32
                        | if s.len() == 6 {
                            0xff000000u32 as i32
                        } else {
                            0
                        }
                })
            } else {
                v.as_number()
                    .and_then(|n| decimal_int(&n.to_string()))
                    .map(|n| n as i32)
            };
            json!(n.ok_or_else(|| ConfigError::new(&p, "invalid ARGB color"))?)
        }
        "FLOAT_RANGE" => {
            let a = parse_float(&v["from"], &p)?;
            let b = parse_float(&v["to"], &p)?;
            if a > b {
                return Err(ConfigError::new(&p, "reversed range"));
            }
            json!({"from":a,"to":b})
        }
        "MULTI_CHOOSE" => {
            let a = v
                .as_array()
                .ok_or_else(|| ConfigError::new(&p, "expected source list"))?;
            if a.iter()
                .any(|v| !SOURCES.iter().any(|s| v.as_str() == Some(s)))
            {
                return Err(ConfigError::new(&p, "unknown source"));
            }
            json!(
                SOURCES
                    .into_iter()
                    .filter(|s| a.iter().any(|v| v.as_str() == Some(s)))
                    .collect::<Vec<_>>()
            )
        }
        _ => return Err(ConfigError::new(&p, "unsupported field")),
    })
}
fn walk(patch: &mut SettingsPatch, n: &Nodes, path: &str) {
    let prefix = format!("{path}.");
    for f in fields()
        .into_iter()
        .filter(|f| f.path.starts_with(&prefix) && !f.path[prefix.len()..].contains('.'))
    {
        let name = f.path[prefix.len()..].to_owned();
        let fp = f.path.clone();
        leaf(patch, n, &name, &format!("Crosshair.{fp}"), |v, _| {
            Ok(SettingChange::Crosshair(CrosshairSetting(
                fp.clone(),
                parse_value(v, &f)?,
            )))
        });
    }
    for child in match path {
        "Mode.Circle" => vec!["Radius", "Color"],
        "Mode.CS2" => vec!["Crosshair", "Color"],
        "CooldownBar" => vec!["ReadyFlash"],
        _ => vec![],
    } {
        if let Some(v) = n.get(child) {
            match nodes(v, child, &format!("Crosshair.{path}.{child}")) {
                Ok(n) => walk(patch, &n, &format!("{path}.{child}")),
                Err(e) => patch.issues.push(e),
            }
        }
    }
}
pub(super) fn parse_crosshair(patch: &mut SettingsPatch, root: &Nodes) {
    if let Some(m) = root.get("Mode") {
        if let Some(active) = m["active"].as_str().filter(|s| MODES.contains(s)) {
            patch
                .changes
                .push(SettingChange::Crosshair(CrosshairSetting(
                    "Mode".into(),
                    json!(active),
                )));
        } else {
            patch
                .issues
                .push(ConfigError::new("Crosshair.Mode", "unknown mode"));
        }
        for mode in MODES {
            if let Some(v) = m.get("choices").and_then(|v| v.get(mode)) {
                match nodes(v, mode, &format!("Crosshair.Mode.{mode}")) {
                    Ok(n) => walk(patch, &n, &format!("Mode.{mode}")),
                    Err(e) => patch.issues.push(e),
                }
            }
        }
    }
    for name in ["CooldownBar", "HitFlash"] {
        if let Some(v) = root.get(name) {
            match nodes(v, name, &format!("Crosshair.{name}")) {
                Ok(n) => walk(patch, &n, name),
                Err(e) => patch.issues.push(e),
            }
        }
    }
}
