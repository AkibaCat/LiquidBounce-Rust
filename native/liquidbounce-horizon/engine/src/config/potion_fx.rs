// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete PotionFX choices, including inactive Simple/Image and Shader styles.
use super::*;
pub use liquidbounce_behaviors::potion_fx::PotionFXSetting;
use liquidbounce_behaviors::potion_fx::{PotionFX, schema};
use liquidbounce_behaviors::target_renderer::Node;
fn key(prefix: &str, path: &str) -> String {
    if path.is_empty() {
        return prefix.into();
    }
    format!(
        "{prefix}.{}",
        path.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn join(p: &str, n: &str) -> String {
    if p.is_empty() {
        n.into()
    } else {
        format!("{p}.{n}")
    }
}
pub fn serialize_potion_fx(
    s: &PotionFX,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    fn build(
        n: &Node,
        p: &str,
        s: &PotionFX,
        t: &Tree,
        prefix: &str,
    ) -> Result<Value, ConfigError> {
        let k = key(prefix, p);
        let mut v = match n.kind {
            "CONFIGURABLE" | "TOGGLEABLE" => t.group(
                n.name,
                n.kind,
                n.children
                    .iter()
                    .map(|c| build(c, &join(p, c.name), s, t, prefix))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            "CHOICE" => {
                let choices = n
                    .choices
                    .iter()
                    .map(|c| Ok((c.name.into(), build(c, &join(p, c.name), s, t, prefix)?)))
                    .collect::<Result<Map<_, _>, ConfigError>>()?;
                let mut v = t.mode(s.text(p), choices);
                v["name"] = json!(n.name);
                v
            }
            _ => {
                let value = match n.kind {
                    "FLOAT" => float_json(s.number(p) as f32,p)?,
                    "INT_RANGE"=>{let v=s.value(p);json!({"from":v["from"],"to":v["to"]})},
                    "FLOAT_RANGE"=>{let v=s.value(p);json!({"from":float_json(v["from"].as_f64().unwrap()as f32,p)?,"to":float_json(v["to"].as_f64().unwrap()as f32,p)?})},
                    "VECTOR2_F" => json!({"x":float_json(s.value(p)["x"].as_f64().unwrap_or(0.) as f32,p)?,"y":float_json(s.value(p)["y"].as_f64().unwrap_or(0.) as f32,p)?}),
                    "CURVE" => Value::Array(s.value(p).as_array().into_iter().flatten().map(|v|Ok(json!({"x":float_json(v["x"].as_f64().unwrap_or(0.) as f32,p)?,"y":float_json(v["y"].as_f64().unwrap_or(0.) as f32,p)?}))).collect::<Result<Vec<_>,ConfigError>>()?),
                    _ => s.value(p).clone()
                };
                if value.is_null() {
                    return Err(ConfigError::new(p, "missing PotionFX setting"));
                }
                if let Some([a, b]) = n.range {
                    t.ranged(
                        n.name,
                        n.kind,
                        value,
                        if matches!(n.kind, "INT" | "INT_RANGE") {
                            json!({"from":a as i32,"to":b as i32})
                        } else {
                            json!({"from":float_json(a,p)?,"to":float_json(b,p)?})
                        },
                        n.suffix,
                        &k,
                    )
                } else {
                    t.leaf(n.name, n.kind, value, &k, None)
                }
            }
        };
        if t.format == JsonFormat::Interop {
            v["key"] = json!(k);
            v["description"] = json!((t.describe)(&format!("{k}.description")));
            if n.kind == "CHOOSE" {
                v["choices"] = json!(n.choices.iter().map(|c| c.name).collect::<Vec<_>>());
            }
            if n.kind == "CURVE" {
                v["innerType"] = json!("VECTOR2_F");
                v["xAxis"] = json!({"label":"Progress","range":{"from":0,"to":1}});
                v["yAxis"] = json!({"label":"Degrees","range":{"from":-180,"to":180}});
                v["tension"] = json!(0.4);
            }
            if n.kind == "FILE" {
                v["supportedExtensions"] = json!(["png", "jpg", "jpeg"]);
                v["dialogMode"] = json!("OPEN_FILE");
            }
            if n.kind == "MUTABLE_LIST" {
                v["innerType"] = json!("TEXT");
            }
        }
        Ok(v)
    }
    let t = Tree { format, describe };
    let result = build(&schema(), "", s, &t, "liquidbounce.module.potionFX")?;
    let mut children = t.common(ModuleKind::PotionFX, common);
    children.extend(result["value"].as_array().into_iter().flatten().cloned());
    Ok(t.group("PotionFX", "TOGGLEABLE", children))
}
pub fn parse_potion_fx_value(
    value: &Value,
    path: &str,
) -> (Vec<PotionFXSetting>, Vec<ConfigError>) {
    fn parse_color(v: &Value, p: &str) -> Result<Value, ConfigError> {
        if let Some(s) = v.as_str() {
            let h = s.strip_prefix('#').unwrap_or(s);
            if matches!(h.len(), 6 | 8) {
                return i64::from_str_radix(h, 16)
                    .map(|x| {
                        json!(
                            x as i32
                                | if h.len() == 6 {
                                    0xff000000u32 as i32
                                } else {
                                    0
                                }
                        )
                    })
                    .map_err(|_| ConfigError::new(p, "invalid ARGB color"));
            }
        }
        parse_int(v, p).map(|v| json!(v))
    }
    fn pair(v: &Value, p: &str) -> Result<Value, ConfigError> {
        let members = if let Some(a) = v.as_array().filter(|a| a.len() == 2) {
            [&a[0], &a[1]]
        } else {
            [
                v.get("x")
                    .ok_or_else(|| ConfigError::new(p, "missing vector x"))?,
                v.get("y")
                    .ok_or_else(|| ConfigError::new(p, "missing vector y"))?,
            ]
        };
        Ok(json!({"x":parse_float(members[0],p)?,"y":parse_float(members[1],p)?}))
    }
    fn walk(
        n: &Node,
        input: &Value,
        p: &str,
        root: &str,
        out: &mut Vec<PotionFXSetting>,
        errors: &mut Vec<ConfigError>,
    ) {
        let full = if p.is_empty() {
            root.into()
        } else {
            format!("{root}.{p}")
        };
        if matches!(n.kind, "CONFIGURABLE" | "TOGGLEABLE") {
            match nodes(input, n.name, &full) {
                Ok(values) => {
                    for c in &n.children {
                        if let Some(v) = values.get(c.name) {
                            walk(c, v, &join(p, c.name), root, out, errors);
                        }
                    }
                }
                Err(e) => errors.push(e),
            }
            return;
        }
        if n.kind == "CHOICE" {
            let tag = input
                .get("active")
                .or_else(|| input.get("value"))
                .and_then(Value::as_str);
            match tag {
                Some(tag) if n.choices.iter().any(|c| c.name == tag) => {
                    out.push(PotionFXSetting(p.into(), json!(tag)))
                }
                Some(_) => errors.push(ConfigError::new(&full, "unknown PotionFX choice")),
                None => errors.push(ConfigError::new(&full, "missing active choice")),
            };
            if let Some(choices) = input.get("choices").and_then(Value::as_object) {
                for c in &n.choices {
                    if let Some(v) = choices.get(c.name) {
                        walk(c, v, &join(p, c.name), root, out, errors);
                    }
                }
            }
            return;
        }
        let result = (|| {
            let v = value_of(input, &full)?;
            Ok(match n.kind {
                "BOOLEAN" => json!(parse_bool(v, &full)?),
                "FLOAT" => {
                    let f = parse_float(v, &full)?;
                    float_json(f, &full)?
                }
                "INT" => json!(parse_int(v, &full)?),
                "INT_RANGE" | "FLOAT_RANGE" => {
                    let a = v
                        .get("from")
                        .ok_or_else(|| ConfigError::new(&full, "missing range from"))?;
                    let b = v
                        .get("to")
                        .ok_or_else(|| ConfigError::new(&full, "missing range to"))?;
                    let (a, b) = if n.kind == "INT_RANGE" {
                        (parse_int(a, &full)? as f32, parse_int(b, &full)? as f32)
                    } else {
                        (parse_float(a, &full)?, parse_float(b, &full)?)
                    };
                    let [lo, hi] = n.range.unwrap();
                    if !a.is_finite() || !b.is_finite() || a < lo || a > b || b > hi {
                        return Err(ConfigError::new(
                            &full,
                            "expected ordered range within setting bounds",
                        ));
                    }
                    if n.kind == "INT_RANGE" {
                        json!({"from":a as i32,"to":b as i32})
                    } else {
                        json!({"from":float_json(a,&full)?,"to":float_json(b,&full)?})
                    }
                }
                "COLOR" => parse_color(v, &full)?,
                "FILE" => json!(
                    v.as_str()
                        .ok_or_else(|| ConfigError::new(&full, "expected file path"))?
                ),
                "CHOOSE" => {
                    let text = v
                        .as_str()
                        .ok_or_else(|| ConfigError::new(&full, "expected preset name"))?;
                    if !n.choices.iter().any(|c| c.name == text) {
                        return Err(ConfigError::new(&full, "unknown PotionFX preset"));
                    }
                    json!(text)
                }
                "VECTOR2_F" => pair(v, &full)?,
                "CURVE" => {
                    let a = v
                        .as_array()
                        .ok_or_else(|| ConfigError::new(&full, "expected curve points"))?;
                    let mut points = Vec::with_capacity(a.len());
                    for point in a {
                        points.push(pair(point, &full)?);
                    }
                    json!(points)
                }
                "MUTABLE_LIST" => {
                    let a = v
                        .as_array()
                        .ok_or_else(|| ConfigError::new(&full, "expected text array"))?;
                    let mut lines = vec![];
                    for v in a {
                        lines.push(
                            v.as_str()
                                .ok_or_else(|| ConfigError::new(&full, "expected text line"))?,
                        );
                    }
                    json!(lines)
                }
                _ => return Err(ConfigError::new(&full, "unknown PotionFX value type")),
            })
        })();
        match result {
            Ok(v) => {
                if matches!(n.kind, "INT" | "FLOAT") {
                    let f = v.as_f64().unwrap() as f32;
                    if let Some([a, b]) = n.range {
                        if f < a || f > b {
                            errors.push(ConfigError::new(&full, "value outside setting range"));
                            return;
                        }
                    }
                }
                out.push(PotionFXSetting(p.into(), v));
            }
            Err(e) => errors.push(e),
        }
    }
    let (mut changes, mut errors) = (vec![], vec![]);
    walk(&schema(), value, "", path, &mut changes, &mut errors);
    (changes, errors)
}

pub(super) fn parse_potion_fx(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let value = json!({"name":"PotionFX","value":root.values().copied().collect::<Vec<_>>()});
    let (changes, errors) = parse_potion_fx_value(&value, "PotionFX");
    p.changes
        .extend(changes.into_iter().map(SettingChange::PotionFX));
    p.issues.extend(errors);
}
