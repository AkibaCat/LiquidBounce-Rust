// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::tracers::{ColorMode, Tracers};
#[derive(Clone, Debug, PartialEq)]
pub enum TracersSetting {
    Mode(ColorMode),
    Saturation(f32),
    Brightness(f32),
    Hue(Vec<[f32; 2]>),
    Alpha(f32),
    HealthAlpha(i32),
    Color(i32),
    LineWidth(f32),
    MaximumDistance(f32),
    Render(BreadcrumbsSetting),
}
impl TracersSetting {
    pub fn apply(self, s: &mut Tracers) {
        match self {
            Self::Mode(v) => s.color_mode = v,
            Self::Saturation(v) => s.saturation = v,
            Self::Brightness(v) => s.brightness = v,
            Self::Hue(v) => s.hue = v,
            Self::Alpha(v) => s.alpha = v,
            Self::HealthAlpha(v) => s.health_alpha = v,
            Self::Color(v) => s.color = v,
            Self::LineWidth(v) => s.line_width = v,
            Self::MaximumDistance(v) => s.maximum_distance = v,
            Self::Render(v) => v.apply(&mut s.render),
        }
    }
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.tracers.{}",
        path.split('.')
            .map(|s| {
                let mut c = s.chars();
                c.next().map_or(String::new(), |f| {
                    f.to_lowercase().collect::<String>() + c.as_str()
                })
            })
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn rename(v: &mut Value, describe: &dyn Fn(&str) -> String) {
    match v {
        Value::Array(a) => {
            for v in a {
                rename(v, describe)
            }
        }
        Value::Object(m) => {
            for v in m.values_mut() {
                rename(v, describe)
            }
            if let Some(k) = m.get("key").and_then(Value::as_str) {
                m.insert(
                    "description".into(),
                    json!(describe(&format!("{k}.description"))),
                );
            }
        }
        Value::String(s) => {
            *s = s.replace(
                "liquidbounce.module.breadcrumbs",
                "liquidbounce.module.tracers",
            )
        }
        _ => {}
    }
}
pub fn serialize_tracers(
    s: &Tracers,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Tracers, common);
    let mut choices = Map::new();
    let mut hue = t.leaf(
        "Hue",
        "CURVE",
        json!(
            s.hue
                .iter()
                .map(|p| json!({"x":p[0],"y":p[1]}))
                .collect::<Vec<_>>()
        ),
        &key("ColorMode.Distance.Hue"),
        None,
    );
    if format == JsonFormat::Interop {
        hue["innerType"] = json!("VECTOR2_F");
        hue["xAxis"] = json!({"label":"Distance","range":{"from":0.0,"to":200.0}});
        hue["yAxis"] = json!({"label":"Hue","range":{"from":0.0,"to":360.0}});
        hue["tension"] = json!(0.4);
    }
    let mut distance = vec![];
    for (name, v) in [("Saturation", s.saturation), ("Brightness", s.brightness)] {
        distance.push(t.ranged(
            name,
            "FLOAT",
            float_json(v, name)?,
            json!({"from":0,"to":1}),
            "",
            &key(&format!("ColorMode.Distance.{name}")),
        ));
    }
    distance.push(hue);
    distance.push(t.ranged(
        "Alpha",
        "FLOAT",
        float_json(s.alpha, "Alpha")?,
        json!({"from":0,"to":1}),
        "",
        &key("ColorMode.Distance.Alpha"),
    ));
    choices.insert(
        "Distance".into(),
        t.group("Distance", "CONFIGURABLE", distance),
    );
    choices.insert(
        "Health".into(),
        t.group(
            "Health",
            "CONFIGURABLE",
            vec![t.ranged(
                "Alpha",
                "INT",
                json!(s.health_alpha),
                json!({"from":0,"to":255}),
                "",
                &key("ColorMode.Health.Alpha"),
            )],
        ),
    );
    choices.insert(
        "Static".into(),
        t.group(
            "Static",
            "CONFIGURABLE",
            vec![t.leaf(
                "Color",
                "COLOR",
                json!(s.color),
                &key("ColorMode.Static.Color"),
                None,
            )],
        ),
    );
    choices.insert("Rainbow".into(), t.group("Rainbow", "CONFIGURABLE", vec![]));
    let mut mode = t.mode(s.color_mode.tag(), choices);
    mode["name"] = json!("ColorMode");
    if format == JsonFormat::Interop {
        mode["key"] = json!(key("ColorMode"));
        mode["description"] = json!(describe(&format!("{}.description", key("ColorMode"))));
    }
    values.push(mode);
    for (name, v, to) in [
        ("LineWidth", s.line_width, 16),
        ("MaximumDistance", s.maximum_distance, 512),
    ] {
        values.push(t.ranged(
            name,
            "FLOAT",
            float_json(v, name)?,
            json!({"from":1,"to":to}),
            "",
            &key(name),
        ));
    }
    let tree = serialize_breadcrumbs(&s.render, common, format, describe)?;
    let mut render = tree["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Render")
        .unwrap()
        .clone();
    rename(&mut render, describe);
    values.push(render);
    Ok(t.group("Tracers", "TOGGLEABLE", values))
}
fn change(v: TracersSetting) -> SettingChange {
    SettingChange::Tracers(v)
}
pub(super) fn parse_tracers(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(v) = root.get("ColorMode") {
        match v
            .get("active")
            .and_then(scalar_string)
            .and_then(|s| ColorMode::ALL.into_iter().find(|m| m.tag() == s))
        {
            Some(v) => patch.changes.push(change(TracersSetting::Mode(v))),
            None => patch.issues.push(ConfigError::new(
                "Tracers.ColorMode.active",
                "unknown case-sensitive mode",
            )),
        }
        if let Some(choices) = v.get("choices").and_then(Value::as_object) {
            for name in ["Distance", "Health", "Static", "Rainbow"] {
                let Some(v) = choices.get(name) else {
                    patch
                        .issues
                        .push(ConfigError::new(name, "choice not found"));
                    continue;
                };
                let n = match nodes(v, name, name) {
                    Ok(n) => n,
                    Err(e) => {
                        patch.issues.push(e);
                        continue;
                    }
                };
                match name {
                    "Distance" => {
                        for (name, set) in [
                            (
                                "Saturation",
                                TracersSetting::Saturation as fn(f32) -> TracersSetting,
                            ),
                            ("Brightness", TracersSetting::Brightness),
                            ("Alpha", TracersSetting::Alpha),
                        ] {
                            leaf(patch, &n, name, name, |v, p| {
                                parse_float(v, p).map(|v| change(set(v)))
                            });
                        }
                        leaf(
                            patch,
                            &n,
                            "Hue",
                            "Tracers.ColorMode.Distance.Hue",
                            |v, p| {
                                let a = v
                                    .as_array()
                                    .ok_or_else(|| ConfigError::new(p, "expected curve points"))?;
                                let mut points = vec![];
                                for point in a {
                                    points.push([
                                        parse_float(&point["x"], p)?,
                                        parse_float(&point["y"], p)?,
                                    ]);
                                }
                                if points.is_empty() {
                                    return Err(ConfigError::new(p, "curve must contain points"));
                                }
                                Ok(change(TracersSetting::Hue(points)))
                            },
                        );
                    }
                    "Health" => leaf(
                        patch,
                        &n,
                        "Alpha",
                        "Tracers.ColorMode.Health.Alpha",
                        |v, p| parse_int(v, p).map(|v| change(TracersSetting::HealthAlpha(v))),
                    ),
                    "Static" => leaf(
                        patch,
                        &n,
                        "Color",
                        "Tracers.ColorMode.Static.Color",
                        |v, p| {
                            let v = match v {
                                Value::Number(n) => decimal_int(&n.to_string()),
                                Value::String(s) => {
                                    let h = s.trim_start_matches('#');
                                    i64::from_str_radix(h, 16).ok().map(|v| {
                                        if h.len() == 6 {
                                            v as i32 | 0xff000000u32 as i32
                                        } else {
                                            v as i32
                                        }
                                    })
                                }
                                _ => None,
                            };
                            v.map(|v| change(TracersSetting::Color(v)))
                                .ok_or_else(|| ConfigError::new(p, "invalid ARGB color"))
                        },
                    ),
                    _ => {}
                }
            }
        } else {
            patch.issues.push(ConfigError::new(
                "Tracers.ColorMode",
                "expected choices object",
            ));
        }
    }
    for (name, set) in [
        (
            "LineWidth",
            TracersSetting::LineWidth as fn(f32) -> TracersSetting,
        ),
        ("MaximumDistance", TracersSetting::MaximumDistance),
    ] {
        leaf(patch, root, name, name, |v, p| {
            parse_float(v, p).map(|v| change(set(v)))
        });
    }
    if let Some(v) = root.get("Render") {
        let root = Nodes::from_iter([("Render".to_owned(), *v)]);
        let mut inner = SettingsPatch {
            module: ModuleKind::Tracers,
            changes: Vec::new(),
            issues: Vec::new(),
        };
        breadcrumbs::parse_breadcrumbs(&mut inner, &root);
        for c in inner.changes {
            if let SettingChange::Breadcrumbs(v) = c {
                patch.changes.push(change(TracersSetting::Render(v)));
            }
        }
        for mut e in inner.issues {
            e.path = e.path.replace("Breadcrumbs", "Tracers");
            patch.issues.push(e);
        }
    }
}
