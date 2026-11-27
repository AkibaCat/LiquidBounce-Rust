// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{air_place::Settings, camera_clip::ModifierKey};
#[derive(Clone, Debug, PartialEq)]
pub struct AirPlaceSetting(pub String, pub Value);
fn parse_color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    let fail = || ConfigError::new(p, "expected ARGB number or hex color");
    match v {
        Value::Number(n) => decimal_int(&n.to_string()).ok_or_else(fail),
        Value::String(s) => {
            let h = s.strip_prefix('#').unwrap_or(s);
            if !matches!(h.len(), 6 | 8) {
                return Err(fail());
            }
            let color = i64::from_str_radix(h, 16).map_err(|_| fail())? as i32;
            Ok(if h.len() == 6 {
                color | 0xff000000u32 as i32
            } else {
                color
            })
        }
        _ => Err(fail()),
    }
}
impl AirPlaceSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        let v = &self.1;
        match p {
            "Preview.Enabled" => s.preview = parse_bool(v, p)?,
            "Preview.OutlineOnly" => s.outline_only = parse_bool(v, p)?,
            "Preview.Color" => s.color = parse_color(v, p)?,
            "Preview.OutlineColor" => s.outline_color = parse_color(v, p)?,
            "PlaceInLiquid" => s.place_in_liquid = parse_bool(v, p)?,
            "CustomRange.Enabled" => s.custom_range = parse_bool(v, p)?,
            "CustomRange.Range" => s.range = parse_float(v, p)?,
            "CustomRange.ScrollAdjust.Enabled" => s.scroll_adjust = parse_bool(v, p)?,
            "CustomRange.ScrollAdjust.Sensitivity" => s.sensitivity = parse_float(v, p)?,
            "CustomRange.ScrollAdjust.Modifier" => {
                s.modifier = v
                    .as_str()
                    .and_then(ModifierKey::from_name)
                    .ok_or_else(|| ConfigError::new(p, "expected InputConstants key string"))?
            }
            _ => return Err(ConfigError::new(p, "unknown AirPlace setting")),
        };
        Ok(())
    }
}
pub fn serialize_air_place(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let k = |p: &str| format!("liquidbounce.module.airPlace.{p}");
    let mut values = t.common(ModuleKind::AirPlace, c);
    values.push(t.leaf(
        "PlaceInLiquid",
        "BOOLEAN",
        json!(s.place_in_liquid),
        &k("placeInLiquid"),
        None,
    ));
    values.push(t.group(
        "Preview",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.preview),
                &k("preview.enabled"),
                None,
            ),
            t.leaf(
                "OutlineOnly",
                "BOOLEAN",
                json!(s.outline_only),
                &k("preview.outlineOnly"),
                None,
            ),
            t.leaf("Color", "COLOR", json!(s.color), &k("preview.color"), None),
            t.leaf(
                "OutlineColor",
                "COLOR",
                json!(s.outline_color),
                &k("preview.outlineColor"),
                None,
            ),
        ],
    ));
    values.push(t.group(
        "CustomRange",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.custom_range),
                &k("customRange.enabled"),
                None,
            ),
            t.ranged(
                "Range",
                "FLOAT",
                float_json(s.range, "AirPlace.CustomRange.Range")?,
                json!({"from":1.,"to":4.5}),
                "m",
                &k("customRange.range"),
            ),
            t.group(
                "ScrollAdjust",
                "TOGGLEABLE",
                vec![
                    t.leaf(
                        "Enabled",
                        "BOOLEAN",
                        json!(s.scroll_adjust),
                        &k("customRange.scrollAdjust.enabled"),
                        None,
                    ),
                    t.leaf(
                        "Modifier",
                        "KEY",
                        json!(s.modifier.name),
                        &k("customRange.scrollAdjust.modifier"),
                        None,
                    ),
                    t.ranged(
                        "Sensitivity",
                        "FLOAT",
                        float_json(
                            s.sensitivity,
                            "AirPlace.CustomRange.ScrollAdjust.Sensitivity",
                        )?,
                        json!({"from":0.1,"to":1.}),
                        "",
                        &k("customRange.scrollAdjust.sensitivity"),
                    ),
                ],
            ),
        ],
    ));
    Ok(t.group("AirPlace", "TOGGLEABLE", values))
}
pub(super) fn parse_air_place(p: &mut SettingsPatch, root: &Nodes<'_>) {
    fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
        leaf(p, n, name, &format!("AirPlace.{path}"), |v, _| {
            let s = AirPlaceSetting(path.into(), v.clone());
            s.apply(&mut Settings::default())?;
            Ok(SettingChange::AirPlace(s))
        });
    }
    add(p, root, "PlaceInLiquid", "PlaceInLiquid");
    for (group, names) in [
        (
            "Preview",
            &["Enabled", "OutlineOnly", "Color", "OutlineColor"][..],
        ),
        ("CustomRange", &["Enabled", "Range"][..]),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("AirPlace.{group}")) {
                Ok(n) => {
                    for name in names {
                        add(p, &n, name, &format!("{group}.{name}"));
                    }
                    if group == "CustomRange" {
                        if let Some(v) = n.get("ScrollAdjust") {
                            match nodes(v, "ScrollAdjust", "AirPlace.CustomRange.ScrollAdjust") {
                                Ok(n) => {
                                    for name in ["Enabled", "Modifier", "Sensitivity"] {
                                        add(
                                            p,
                                            &n,
                                            name,
                                            &format!("CustomRange.ScrollAdjust.{name}"),
                                        );
                                    }
                                }
                                Err(e) => p.issues.push(e),
                            }
                        }
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
}
