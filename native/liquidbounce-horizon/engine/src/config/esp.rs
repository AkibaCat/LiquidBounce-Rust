// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    breadcrumbs::{STYLES, specs},
    esp::{Esp, MODES, OBJECTS},
    storage_esp::StorageEsp,
};
#[derive(Clone, Debug, PartialEq)]
pub enum EspSetting {
    Color(TracersSetting),
    Value(String, Value),
}
impl EspSetting {
    pub fn apply(self, s: &mut Esp) {
        let (p, v) = match self {
            Self::Value(p, v) => (p, v),
            Self::Color(v) => {
                v.apply(&mut s.color);
                return;
            }
        };
        let f = || v.as_f64().unwrap() as f32;
        let b = || v.as_bool().unwrap();
        match p.as_str() {
            "Mode" => {
                let mode = MODES.iter().position(|n| v == *n).unwrap();
                if s.mode != mode {
                    s.prepared.clear();
                }
                s.mode = mode;
            }
            "Mode.Box.Expand" => s.expand[0] = f(),
            "Mode.Box.Outline" => s.outline = b(),
            "Mode.Box.MergeIntersecting" => s.merge_intersecting = b(),
            "Mode.2D.Expand" => s.expand[1] = f(),
            "Mode.2D.Fill" => s.fill = b(),
            "Mode.2D.Outline.Enabled" => s.outline_2d = b(),
            "Mode.2D.Outline.Thickness" => s.outline_thickness = f(),
            "Mode.2D.Corner.Enabled" => s.corner = b(),
            "Mode.2D.Corner.Gap" => s.gap = f(),
            "Mode.2D.Border.Enabled" => s.border = b(),
            "Mode.2D.Border.Thickness" => s.border_thickness = f(),
            "Mode.2D.HealthBar.Enabled" => s.health_bar = b(),
            "Mode.2D.HealthBar.Spacing" => s.health_spacing = f(),
            "Mode.Legacy2D.Scale" => s.scale = f(),
            "Mode.Legacy2D.YOffset" => s.y_offset = f(),
            "Mode.Legacy2D.BackgroundAlpha" => s.background_alpha = v.as_i64().unwrap() as i32,
            "Mode.Glow.Style" => {
                s.style = std::iter::once("Vanilla")
                    .chain(STYLES)
                    .position(|n| v == n)
                    .unwrap()
            }
            "Invisible" => s.invisible = v.as_i64().unwrap() as i32,
            "Friends" => s.friends = v.as_i64().unwrap() as i32,
            "MaximumDistance" => s.maximum_distance = f(),
            "Objects" => s.objects = v.as_u64().unwrap() as u8,
            "ObjectColor" => s.object_color = v.as_i64().unwrap() as i32,
            _ => {
                for (i, name) in STYLES.iter().enumerate() {
                    if let Some(field) = p.strip_prefix(&format!("Mode.Glow.Style.{name}.")) {
                        if specs(i).any(|p| p.name == field && p.color) {
                            s.styles[i].accent = v.as_i64().unwrap() as i32;
                        } else {
                            s.styles[i].values.insert(field.into(), f());
                        }
                        return;
                    }
                }
            }
        }
    }
}
fn key(p: &str) -> String {
    format!(
        "liquidbounce.module.esp.{}",
        p.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn meta(v: &mut Value, p: &str, t: &Tree) {
    if t.format == JsonFormat::Interop {
        v["key"] = json!(key(p));
        v["description"] = json!((t.describe)(&format!("{}.description", key(p))));
    }
}
fn renamed(v: &mut Value, describe: &dyn Fn(&str) -> String) {
    match v {
        Value::Array(a) => {
            for v in a {
                renamed(v, describe)
            }
        }
        Value::Object(o) => {
            for v in o.values_mut() {
                renamed(v, describe);
            }
            if let Some(k) = o.get("key").and_then(Value::as_str) {
                o.insert(
                    "description".into(),
                    json!(describe(&format!("{k}.description"))),
                );
            }
        }
        Value::String(s) => {
            *s = s
                .replace("liquidbounce.module.tracers", "liquidbounce.module.esp")
                .replace("liquidbounce.module.storageESP", "liquidbounce.module.esp")
        }
        _ => {}
    }
}
fn choice(t: &Tree, name: &str, p: &str, active: &str, choices: Map<String, Value>) -> Value {
    let mut v = t.mode(active, choices);
    v["name"] = json!(name);
    meta(&mut v, p, t);
    v
}
fn bool_field(t: &Tree, name: &str, p: &str, b: bool) -> Value {
    t.leaf(name, "BOOLEAN", json!(b), &key(p), None)
}
fn float_field(
    t: &Tree,
    name: &str,
    p: &str,
    v: f32,
    range: [f32; 2],
    suffix: &str,
) -> Result<Value, ConfigError> {
    Ok(t.ranged(
        name,
        "FLOAT",
        float_json(v, p)?,
        json!({"from":range[0],"to":range[1]}),
        suffix,
        &key(p),
    ))
}
fn toggle(t: &Tree, name: &str, p: &str, on: bool, mut fields: Vec<Value>) -> Value {
    fields.insert(0, bool_field(t, "Enabled", &format!("{p}.Enabled"), on));
    let mut v = t.group(name, "TOGGLEABLE", fields);
    meta(&mut v, p, t);
    v
}
fn named<'a>(v: &'a Value, name: &str) -> &'a Value {
    v["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == name)
        .unwrap()
}
pub fn serialize_esp(
    s: &Esp,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Esp, common);
    let mut style = named(
        &serialize_storage_esp(
            &StorageEsp {
                style: s.style,
                styles: s.styles.clone(),
                ..Default::default()
            },
            common,
            format,
            describe,
        )?,
        "Mode",
    )["choices"]["Glow"]
        .clone();
    renamed(&mut style, describe);
    let modes = Map::from_iter([
        (
            "Box".into(),
            t.group(
                "Box",
                "CONFIGURABLE",
                vec![
                    float_field(&t, "Expand", "Mode.Box.Expand", s.expand[0], [0., 0.5], "")?,
                    bool_field(&t, "Outline", "Mode.Box.Outline", s.outline),
                    bool_field(
                        &t,
                        "MergeIntersecting",
                        "Mode.Box.MergeIntersecting",
                        s.merge_intersecting,
                    ),
                ],
            ),
        ),
        (
            "2D".into(),
            t.group(
                "2D",
                "CONFIGURABLE",
                vec![
                    float_field(&t, "Expand", "Mode.2D.Expand", s.expand[1], [0., 0.5], "")?,
                    bool_field(&t, "Fill", "Mode.2D.Fill", s.fill),
                    toggle(
                        &t,
                        "Outline",
                        "Mode.2D.Outline",
                        s.outline_2d,
                        vec![float_field(
                            &t,
                            "Thickness",
                            "Mode.2D.Outline.Thickness",
                            s.outline_thickness,
                            [1., 9.],
                            "px",
                        )?],
                    ),
                    toggle(
                        &t,
                        "Corner",
                        "Mode.2D.Corner",
                        s.corner,
                        vec![float_field(
                            &t,
                            "Gap",
                            "Mode.2D.Corner.Gap",
                            s.gap,
                            [1., 100.],
                            "%",
                        )?],
                    ),
                    toggle(
                        &t,
                        "Border",
                        "Mode.2D.Border",
                        s.border,
                        vec![float_field(
                            &t,
                            "Thickness",
                            "Mode.2D.Border.Thickness",
                            s.border_thickness,
                            [1., 9.],
                            "px",
                        )?],
                    ),
                    toggle(
                        &t,
                        "HealthBar",
                        "Mode.2D.HealthBar",
                        s.health_bar,
                        vec![float_field(
                            &t,
                            "Spacing",
                            "Mode.2D.HealthBar.Spacing",
                            s.health_spacing,
                            [0., 32.],
                            "px",
                        )?],
                    ),
                ],
            ),
        ),
        (
            "Legacy2D".into(),
            t.group(
                "Legacy2D",
                "CONFIGURABLE",
                vec![
                    float_field(&t, "Scale", "Mode.Legacy2D.Scale", s.scale, [0.02, 0.3], "")?,
                    float_field(
                        &t,
                        "YOffset",
                        "Mode.Legacy2D.YOffset",
                        s.y_offset,
                        [-1., 1.],
                        "",
                    )?,
                    t.ranged(
                        "BackgroundAlpha",
                        "INT",
                        json!(s.background_alpha),
                        json!({"from":0,"to":255}),
                        "",
                        &key("Mode.Legacy2D.BackgroundAlpha"),
                    ),
                ],
            ),
        ),
        ("Glow".into(), style),
    ]);
    values.push(choice(&t, "Mode", "Mode", MODES[s.mode], modes));
    let tree = serialize_tracers(&s.color, common, format, describe)?;
    let mut colors = named(&tree, "ColorMode").clone();
    renamed(&mut colors, describe);
    values.push(colors);
    for (name, c) in [("Invisible", s.invisible), ("Friends", s.friends)] {
        values.push(t.leaf(name, "COLOR", json!(c), &key(name), None));
    }
    values.push(float_field(
        &t,
        "MaximumDistance",
        "MaximumDistance",
        s.maximum_distance,
        [1., 512.],
        "",
    )?);
    values.push(t.multi("Objects", s.objects as u32, &OBJECTS, &key("Objects")));
    values.push(t.leaf(
        "ObjectColor",
        "COLOR",
        json!(s.object_color),
        &key("ObjectColor"),
        None,
    ));
    Ok(t.group("ESP", "TOGGLEABLE", values))
}
fn color(v: &Value, p: &str) -> Result<Value, ConfigError> {
    if let Some(s) = v.as_str() {
        let h = s.strip_prefix('#').unwrap_or(s);
        if !matches!(h.len(), 6 | 8) {
            return Err(ConfigError::new(p, "expected 6/8 hex digits"));
        }
        i64::from_str_radix(h, 16)
            .map(|c| {
                json!(
                    c as i32
                        | if h.len() == 6 {
                            0xff000000u32 as i32
                        } else {
                            0
                        }
                )
            })
            .map_err(|_| ConfigError::new(p, "invalid ARGB color"))
    } else {
        parse_int(v, p).map(|v| json!(v))
    }
}
fn leaf_setting(patch: &mut SettingsPatch, n: &Nodes, name: &str, path: &str, kind: &str) {
    leaf(patch, n, name, &format!("ESP.{path}"), |v, p| {
        let value = match kind {
            "BOOLEAN" => json!(parse_bool(v, p)?),
            "FLOAT" => float_json(parse_float(v, p)?, p)?,
            "INT" => json!(parse_int(v, p)?),
            _ => color(v, p)?,
        };
        Ok(SettingChange::Esp(EspSetting::Value(path.into(), value)))
    });
}
fn mode(patch: &mut SettingsPatch, value: &Value, path: &str, choices: &[&str]) {
    if let Some(v) = value["active"].as_str().filter(|v| choices.contains(v)) {
        patch
            .changes
            .push(SettingChange::Esp(EspSetting::Value(path.into(), json!(v))));
    } else {
        patch.issues.push(ConfigError::new(
            &format!("ESP.{path}"),
            "unknown case-sensitive mode",
        ));
    }
}
pub(super) fn parse_esp(patch: &mut SettingsPatch, root: &Nodes) {
    if let Some(v) = root.get("ColorMode") {
        let mut n = Nodes::new();
        n.insert("ColorMode".into(), *v);
        let mut colors = SettingsPatch {
            module: ModuleKind::Tracers,
            changes: vec![],
            issues: vec![],
        };
        super::tracers::parse_tracers(&mut colors, &n);
        for change in colors.changes {
            if let SettingChange::Tracers(v) = change {
                patch.changes.push(SettingChange::Esp(EspSetting::Color(v)));
            }
        }
        patch.issues.extend(colors.issues);
    }
    if let Some(m) = root.get("Mode") {
        mode(patch, m, "Mode", &MODES);
        for name in MODES {
            let Some(v) = m["choices"].get(name) else {
                continue;
            };
            let n = match nodes(v, name, &format!("ESP.Mode.{name}")) {
                Ok(n) => n,
                Err(e) => {
                    patch.issues.push(e);
                    continue;
                }
            };
            match name {
                "Box" => {
                    for (field, kind) in [
                        ("Expand", "FLOAT"),
                        ("Outline", "BOOLEAN"),
                        ("MergeIntersecting", "BOOLEAN"),
                    ] {
                        leaf_setting(patch, &n, field, &format!("Mode.Box.{field}"), kind);
                    }
                }
                "2D" => {
                    for (field, kind) in [("Expand", "FLOAT"), ("Fill", "BOOLEAN")] {
                        leaf_setting(patch, &n, field, &format!("Mode.2D.{field}"), kind);
                    }
                    for (group, field) in [
                        ("Outline", "Thickness"),
                        ("Corner", "Gap"),
                        ("Border", "Thickness"),
                        ("HealthBar", "Spacing"),
                    ] {
                        if let Some(v) = n.get(group) {
                            match nodes(v, group, &format!("ESP.Mode.2D.{group}")) {
                                Ok(n) => {
                                    leaf_setting(
                                        patch,
                                        &n,
                                        "Enabled",
                                        &format!("Mode.2D.{group}.Enabled"),
                                        "BOOLEAN",
                                    );
                                    leaf_setting(
                                        patch,
                                        &n,
                                        field,
                                        &format!("Mode.2D.{group}.{field}"),
                                        "FLOAT",
                                    );
                                }
                                Err(e) => patch.issues.push(e),
                            }
                        }
                    }
                }
                "Legacy2D" => {
                    for (field, kind) in [
                        ("Scale", "FLOAT"),
                        ("YOffset", "FLOAT"),
                        ("BackgroundAlpha", "INT"),
                    ] {
                        leaf_setting(patch, &n, field, &format!("Mode.Legacy2D.{field}"), kind);
                    }
                }
                "Glow" => {
                    if let Some(style) = n.get("Style") {
                        mode(
                            patch,
                            style,
                            "Mode.Glow.Style",
                            &std::iter::once("Vanilla").chain(STYLES).collect::<Vec<_>>(),
                        );
                        for (i, name) in STYLES.iter().enumerate() {
                            if let Some(v) = style["choices"].get(*name) {
                                match nodes(v, name, &format!("ESP.Mode.Glow.Style.{name}")) {
                                    Ok(n) => {
                                        for spec in specs(i) {
                                            leaf_setting(
                                                patch,
                                                &n,
                                                spec.name,
                                                &format!("Mode.Glow.Style.{name}.{}", spec.name),
                                                if spec.color { "COLOR" } else { "FLOAT" },
                                            );
                                        }
                                    }
                                    Err(e) => patch.issues.push(e),
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    for (name, kind) in [
        ("Invisible", "COLOR"),
        ("Friends", "COLOR"),
        ("MaximumDistance", "FLOAT"),
        ("ObjectColor", "COLOR"),
    ] {
        leaf_setting(patch, root, name, name, kind);
    }
    leaf(patch, root, "Objects", "ESP.Objects", |v, p| {
        let list = v
            .as_array()
            .ok_or_else(|| ConfigError::new(p, "expected object choices"))?;
        let mut mask = 0u8;
        for value in list {
            let name =
                scalar_string(value).ok_or_else(|| ConfigError::new(p, "expected object name"))?;
            let index = OBJECTS
                .iter()
                .position(|n| *n == name)
                .ok_or_else(|| ConfigError::new(p, "unknown case-sensitive object"))?;
            mask |= 1 << index;
        }
        Ok(SettingChange::Esp(EspSetting::Value(
            "Objects".into(),
            json!(mask),
        )))
    });
}
