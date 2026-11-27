// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    radar::{PointerMode, Radar},
    tracers::ColorMode,
};
#[derive(Clone, Debug, PartialEq)]
pub struct RadarSetting(pub String, pub Value);
fn choice<T: Copy>(
    v: &Value,
    all: &[T],
    tag: impl Fn(T) -> &'static str,
    p: &str,
) -> Result<T, ConfigError> {
    let s = scalar_string(v);
    all.iter()
        .copied()
        .find(|v| Some(tag(*v)) == s.as_deref())
        .ok_or_else(|| ConfigError::new(p, "unknown case-sensitive choice"))
}
fn points(v: &Value, p: &str) -> Result<Vec<[f32; 2]>, ConfigError> {
    let a = v
        .as_array()
        .ok_or_else(|| ConfigError::new(p, "expected curve points"))?;
    if a.is_empty() {
        return Err(ConfigError::new(p, "curve must contain points"));
    }
    a.iter()
        .map(|v| Ok([parse_float(&v["x"], p)?, parse_float(&v["y"], p)?]))
        .collect()
}
fn color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    match v {
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
    }
    .ok_or_else(|| ConfigError::new(p, "invalid ARGB color"))
}
impl RadarSetting {
    pub fn apply(&self, s: &mut Radar) -> Result<(), ConfigError> {
        let p = format!("Radar.{}", self.0);
        let v = &self.1;
        match self.0.as_str() {
            "OnlyPlayers" => s.only_players = parse_bool(v, &p)?,
            "HideTeammates" => s.hide_teammates = parse_bool(v, &p)?,
            "Show" => {
                let a = v
                    .as_array()
                    .ok_or_else(|| ConfigError::new(&p, "expected visibility list"))?;
                let mut mask = 0;
                for v in a {
                    match scalar_string(v).as_deref() {
                        Some("InView") => mask |= 1,
                        Some("Offscreen") => mask |= 2,
                        _ => return Err(ConfigError::new(&p, "unknown visibility")),
                    }
                }
                if mask == 0 {
                    return Err(ConfigError::new(&p, "select at least one visibility"));
                }
                s.show = mask;
            }
            "EdgeInset" => s.edge_inset = parse_float(v, &p)?,
            "PointerMode.active" => {
                s.pointer_mode = choice(v, &PointerMode::ALL, PointerMode::tag, &p)?
            }
            "PointerMode.Triangle.Width" => s.width = parse_float(v, &p)?,
            "PointerMode.Triangle.Height" => s.height = parse_float(v, &p)?,
            "PointerMode.Triangle.TailConcaveSize" => s.set_concavity(parse_float(v, &p)?),
            "PointerMode.Image1.Size" => s.image_sizes[0] = parse_float(v, &p)?,
            "PointerMode.Image2.Size" => s.image_sizes[1] = parse_float(v, &p)?,
            "ColorMode.active" => s.color_mode = choice(v, &ColorMode::ALL, ColorMode::tag, &p)?,
            "ColorMode.Distance.Saturation" => s.saturation = parse_float(v, &p)?,
            "ColorMode.Distance.Brightness" => s.brightness = parse_float(v, &p)?,
            "ColorMode.Distance.Hue" => s.hue = points(v, &p)?,
            "ColorMode.Health.Alpha" => s.health_alpha = parse_int(v, &p)?,
            "ColorMode.Static.Color" => s.color = color(v, &p)?,
            "Alpha" => s.alpha = points(v, &p)?,
            _ => return Err(ConfigError::new(&p, "unknown Radar setting")),
        }
        Ok(())
    }
}
fn key(p: &str) -> String {
    format!(
        "liquidbounce.module.radar.{}",
        p.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn curve(t: &Tree<'_>, name: &str, p: &str, points: &[[f32; 2]], axis: &str, to: f32) -> Value {
    let mut v = t.leaf(
        name,
        "CURVE",
        json!(
            points
                .iter()
                .map(|p| json!({"x":p[0],"y":p[1]}))
                .collect::<Vec<_>>()
        ),
        &key(p),
        None,
    );
    if t.format == JsonFormat::Interop {
        v["innerType"] = json!("VECTOR2_F");
        v["xAxis"] = json!({"label":"Distance","range":{"from":0.,"to":200.}});
        v["yAxis"] = json!({"label":axis,"range":{"from":0.,"to":to}});
        v["tension"] = json!(0.4);
    }
    v
}
fn mode(t: &Tree<'_>, name: &str, active: &str, choices: Map<String, Value>) -> Value {
    let mut v = t.mode(active, choices);
    v["name"] = json!(name);
    if t.format == JsonFormat::Interop {
        v["key"] = json!(key(name));
        v["description"] = json!((t.describe)(&format!("{}.description", key(name))));
    }
    v
}
pub fn serialize_radar(
    s: &Radar,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::Radar, c);
    for (n, v) in [
        ("OnlyPlayers", s.only_players),
        ("HideTeammates", s.hide_teammates),
    ] {
        values.push(t.leaf(n, "BOOLEAN", json!(v), &key(n), None));
    }
    let mut show = t.multi("Show", s.show, &["InView", "Offscreen"], &key("Show"));
    if f == JsonFormat::Interop {
        show["canBeNone"] = json!(false);
    }
    values.push(show);
    values.push(t.ranged(
        "EdgeInset",
        "FLOAT",
        float_json(s.edge_inset, "Radar.EdgeInset")?,
        json!({"from":0.,"to":0.9}),
        "",
        &key("EdgeInset"),
    ));
    let mut pointers = Map::new();
    let mut triangle = vec![];
    for (n, v, from, to) in [
        ("Width", s.width, 1., 100.),
        ("Height", s.height, 1., 100.),
        ("TailConcaveSize", s.tail_concave_size, 0., 50.),
    ] {
        triangle.push(t.ranged(
            n,
            "FLOAT",
            float_json(v, n)?,
            json!({"from":from,"to":to}),
            "",
            &key(&format!("PointerMode.Triangle.{n}")),
        ));
    }
    pointers.insert(
        "Triangle".into(),
        t.group("Triangle", "CONFIGURABLE", triangle),
    );
    for (i, n) in ["Image1", "Image2"].into_iter().enumerate() {
        pointers.insert(
            n.into(),
            t.group(
                n,
                "CONFIGURABLE",
                vec![t.ranged(
                    "Size",
                    "FLOAT",
                    float_json(s.image_sizes[i], n)?,
                    json!({"from":1.,"to":100.}),
                    "",
                    &key(&format!("PointerMode.{n}.Size")),
                )],
            ),
        );
    }
    values.push(mode(&t, "PointerMode", s.pointer_mode.tag(), pointers));
    let mut colors = Map::new();
    let mut distance = vec![];
    for (n, v) in [("Saturation", s.saturation), ("Brightness", s.brightness)] {
        distance.push(t.ranged(
            n,
            "FLOAT",
            float_json(v, n)?,
            json!({"from":0.,"to":1.}),
            "",
            &key(&format!("ColorMode.Distance.{n}")),
        ));
    }
    distance.push(curve(
        &t,
        "Hue",
        "ColorMode.Distance.Hue",
        &s.hue,
        "Hue",
        360.,
    ));
    colors.insert(
        "Distance".into(),
        t.group("Distance", "CONFIGURABLE", distance),
    );
    colors.insert(
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
    colors.insert(
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
    colors.insert("Rainbow".into(), t.group("Rainbow", "CONFIGURABLE", vec![]));
    values.push(mode(&t, "ColorMode", s.color_mode.tag(), colors));
    values.push(curve(&t, "Alpha", "Alpha", &s.alpha, "Alpha", 1.));
    Ok(t.group("Radar", "TOGGLEABLE", values))
}
pub(super) fn parse_radar(p: &mut SettingsPatch, root: &Nodes<'_>) {
    fn accept(p: &mut SettingsPatch, path: String, v: Value) {
        let setting = RadarSetting(path, v);
        let mut s = Radar::default();
        match setting.apply(&mut s) {
            Ok(()) => p.changes.push(SettingChange::Radar(setting)),
            Err(e) => p.issues.push(e),
        }
    }
    for n in ["OnlyPlayers", "HideTeammates", "Show", "EdgeInset", "Alpha"] {
        if let Some(v) = root.get(n).and_then(|v| v.get("value")) {
            accept(p, n.into(), v.clone());
        }
    }
    for n in ["PointerMode", "ColorMode"] {
        let Some(v) = root.get(n) else { continue };
        if let Some(a) = v.get("active") {
            accept(p, format!("{n}.active"), a.clone());
        }
        if let Some(choices) = v.get("choices").and_then(Value::as_object) {
            let branches: &[(&str, &[&str])] = if n == "PointerMode" {
                &[
                    ("Triangle", &["Width", "Height", "TailConcaveSize"]),
                    ("Image1", &["Size"]),
                    ("Image2", &["Size"]),
                ]
            } else {
                &[
                    ("Distance", &["Saturation", "Brightness", "Hue"]),
                    ("Health", &["Alpha"]),
                    ("Static", &["Color"]),
                    ("Rainbow", &[]),
                ]
            };
            for (b, fields) in branches {
                let Some(v) = choices.get(*b) else { continue };
                match nodes(v, b, b) {
                    Ok(items) => {
                        for field in *fields {
                            if let Some(v) = items.get(*field).and_then(|v| v.get("value")) {
                                accept(p, format!("{n}.{b}.{field}"), v.clone());
                            }
                        }
                    }
                    Err(e) => p.issues.push(e),
                }
            }
        } else {
            p.issues.push(ConfigError::new(
                &format!("Radar.{n}"),
                "expected choices object",
            ));
        }
    }
}
