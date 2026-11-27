// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::blink_esp::{BlinkEsp, MODES};
#[derive(Clone, Debug, PartialEq)]
pub enum BlinkEspSetting {
    Field(String, Value),
    Render(BreadcrumbsSetting),
}
impl BlinkEspSetting {
    pub fn apply(self, s: &mut BlinkEsp) {
        match self {
            Self::Render(v) => v.apply(&mut s.render),
            Self::Field(p, v) => {
                let n = || v.as_i64().unwrap() as i32;
                match p.as_str() {
                    "Esp" => s.mode = v.as_u64().unwrap() as usize,
                    "Esp.Box.Color" => s.box_color = n(),
                    "Esp.Box.OutlineColor" => s.box_outline = n(),
                    "Esp.Model.OutlineColor" => s.model_outline = n(),
                    "Esp.Model.LightPercent" => s.light_percent = n(),
                    "Esp.Wireframe.Color" => s.wire_color = n(),
                    "Esp.Wireframe.OutlineColor" => s.wire_outline = n(),
                    "Esp.Ring.Color" => s.ring_color = n(),
                    "Esp.Ring.OutlineColor" => s.ring_outline = n(),
                    "Esp.Ring.Radius" => s.radius = v.as_f64().unwrap() as f32,
                    "Line" => s.line = n(),
                    _ => {}
                }
            }
        }
    }
}
#[derive(Default, Debug)]
pub struct BlinkEspPatch {
    pub changes: Vec<BlinkEspSetting>,
    pub issues: Vec<ConfigError>,
}
fn field(out: &mut BlinkEspPatch, node: &Value, path: &str, kind: &str) {
    let parsed = node
        .get("value")
        .ok_or_else(|| ConfigError::new(path, "missing value"))
        .and_then(|v| match kind {
            "COLOR" => match v {
                Value::String(s) => {
                    let h = s.strip_prefix('#').unwrap_or(s);
                    if !matches!(h.len(), 6 | 8) {
                        Err(ConfigError::new(path, "expected 6/8 hex digits"))
                    } else {
                        i64::from_str_radix(h, 16)
                            .map(|v| {
                                json!(
                                    v as i32
                                        | if h.len() == 6 {
                                            0xff000000u32 as i32
                                        } else {
                                            0
                                        }
                                )
                            })
                            .map_err(|_| ConfigError::new(path, "invalid hex"))
                    }
                }
                _ => parse_int(v, path).map(|v| json!(v)),
            },
            "INT" => parse_int(v, path).map(|v| json!(v)),
            _ => parse_float(v, path).map(|v| json!(v)),
        });
    match parsed {
        Ok(v) => out.changes.push(BlinkEspSetting::Field(path.into(), v)),
        Err(e) => out.issues.push(e),
    }
}
pub fn blink_esp_key(path: &str) -> String {
    format!(
        "liquidbounce.settings.blinkManager{}",
        if path.is_empty() {
            String::new()
        } else {
            format!(
                ".{}",
                path.split('.')
                    .map(|p| p[..1].to_ascii_lowercase() + &p[1..])
                    .collect::<Vec<_>>()
                    .join(".")
            )
        }
    )
}
fn metadata(v: &mut Value, path: &str, format: JsonFormat, describe: &dyn Fn(&str) -> String) {
    if format == JsonFormat::Interop {
        let key = blink_esp_key(path);
        v["key"] = json!(key);
        v["description"] = json!(describe(&format!("{key}.description")));
    }
}
fn render_keys(v: &mut Value, describe: &dyn Fn(&str) -> String) {
    match v {
        Value::String(s) => {
            *s = s.replace(
                "liquidbounce.module.breadcrumbs",
                "liquidbounce.settings.blinkManager",
            )
        }
        Value::Array(a) => {
            for v in a {
                render_keys(v, describe)
            }
        }
        Value::Object(o) => {
            for v in o.values_mut() {
                render_keys(v, describe)
            }
            if let Some(key) = o.get("key").and_then(Value::as_str) {
                o.insert(
                    "description".into(),
                    json!(describe(&format!("{key}.description"))),
                );
            }
        }
        _ => {}
    }
}
pub fn serialize_global_blink(
    s: &BlinkEsp,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Value {
    let t = Tree { format, describe };
    let mut choices = Map::new();
    for (mode, fields) in [
        (
            "Box",
            vec![
                ("Color", "COLOR", json!(s.box_color)),
                ("OutlineColor", "COLOR", json!(s.box_outline)),
            ],
        ),
        (
            "Model",
            vec![
                ("OutlineColor", "COLOR", json!(s.model_outline)),
                ("LightPercent", "INT", json!(s.light_percent)),
            ],
        ),
        (
            "Wireframe",
            vec![
                ("Color", "COLOR", json!(s.wire_color)),
                ("OutlineColor", "COLOR", json!(s.wire_outline)),
            ],
        ),
        (
            "Ring",
            vec![
                ("Radius", "FLOAT", json!(s.radius)),
                ("Color", "COLOR", json!(s.ring_color)),
                ("OutlineColor", "COLOR", json!(s.ring_outline)),
            ],
        ),
        ("None", vec![]),
    ] {
        let mut values = vec![];
        for (name, kind, value) in fields {
            let path = format!("Esp.{mode}.{name}");
            let mut v = t.leaf(name, kind, value, &blink_esp_key(&path), None);
            if format == JsonFormat::Interop {
                if name == "Radius" {
                    v["range"] = json!({"from":0.1,"to":2.});
                    v["suffix"] = json!("");
                }
                if name == "LightPercent" {
                    v["range"] = json!({"from":0,"to":100});
                    v["suffix"] = json!("%");
                }
            }
            values.push(v);
        }
        let mut group = t.group(mode, "CONFIGURABLE", values);
        metadata(&mut group, &format!("Esp.{mode}"), format, describe);
        choices.insert(mode.into(), group);
    }
    let mut esp = t.mode(MODES[s.mode], choices);
    esp["name"] = json!("Esp");
    metadata(&mut esp, "Esp", format, describe);
    let common = CommonSettings {
        enabled: true,
        hidden: false,
        key_bind: json!({}),
    };
    let tree = breadcrumbs::serialize_breadcrumbs(&s.render, &common, format, describe)
        .expect("finite Blink shader values");
    let mut render = tree["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Render")
        .unwrap()
        .clone();
    render_keys(&mut render, describe);
    let mut group = t.group(
        "BlinkManager",
        "CONFIGURABLE",
        vec![
            esp,
            t.leaf("Line", "COLOR", json!(s.line), &blink_esp_key("Line"), None),
            render,
        ],
    );
    metadata(&mut group, "", format, describe);
    group
}
pub fn parse_global_blink(value: &Value) -> Result<BlinkEspPatch, ConfigError> {
    let root = nodes(value, "Settings", "Settings")?;
    let mut out = BlinkEspPatch::default();
    let Some(v) = root.get("BlinkManager") else {
        return Ok(out);
    };
    let n = match nodes(v, "BlinkManager", "Settings.BlinkManager") {
        Ok(v) => v,
        Err(e) => {
            out.issues.push(e);
            return Ok(out);
        }
    };
    if let Some(v) = n.get("Line") {
        field(&mut out, v, "Line", "COLOR");
    }
    if let Some(v) = n.get("Esp") {
        let selected = v["active"]
            .as_str()
            .and_then(|s| MODES.iter().position(|m| *m == s));
        // All inactive choices also persist, matching ModeValueGroupSerializer.
        for mode in MODES {
            if let Some(choice) = v.get("choices").and_then(|v| v.get(mode)) {
                match nodes(choice, mode, &format!("Settings.BlinkManager.Esp.{mode}")) {
                    Ok(n) => {
                        for (name, node) in n {
                            let kind = if name == "Radius" {
                                "FLOAT"
                            } else if name == "LightPercent" {
                                "INT"
                            } else if matches!(name.as_str(), "Color" | "OutlineColor") {
                                "COLOR"
                            } else {
                                continue;
                            };
                            field(&mut out, node, &format!("Esp.{mode}.{name}"), kind)
                        }
                    }
                    Err(e) => out.issues.push(e),
                }
            }
        }
        match selected {
            Some(i) => out
                .changes
                .push(BlinkEspSetting::Field("Esp".into(), json!(i))),
            None => out.issues.push(ConfigError::new(
                "Settings.BlinkManager.Esp.active",
                "missing or unknown case-sensitive mode",
            )),
        }
    }
    if let Some(v) = n.get("Render") {
        let mut inner = SettingsPatch {
            module: ModuleKind::Breadcrumbs,
            changes: vec![],
            issues: vec![],
        };
        breadcrumbs::parse_breadcrumbs(&mut inner, &Nodes::from_iter([("Render".into(), *v)]));
        for v in inner.changes {
            if let SettingChange::Breadcrumbs(v) = v {
                out.changes.push(BlinkEspSetting::Render(v));
            }
        }
        for mut e in inner.issues {
            e.path = e.path.replace("Breadcrumbs", "Settings.BlinkManager");
            out.issues.push(e);
        }
    }
    Ok(out)
}
/// Returns each changed leaf/mode independently for the existing valueChanged event.
pub fn blink_changed_values(before: &Value, after: &Value) -> Vec<Value> {
    let mut out = vec![];
    if before == after {
        return out;
    }
    if after["valueType"] == "CHOICE" || after["type"] == "CHOICE" {
        if before["active"] != after["active"] {
            out.push(after.clone());
        }
    }
    if let Some(values) = after["value"].as_array() {
        for v in values {
            let old = before["value"]
                .as_array()
                .and_then(|a| a.iter().find(|x| x["name"] == v["name"]))
                .unwrap_or(&Value::Null);
            out.extend(blink_changed_values(old, v));
        }
    } else if after.get("value").is_some() && before["value"] != after["value"] {
        out.push(after.clone());
    }
    if let Some(choices) = after["choices"].as_object() {
        for (name, v) in choices {
            out.extend(blink_changed_values(&before["choices"][name], v));
        }
    }
    out
}
