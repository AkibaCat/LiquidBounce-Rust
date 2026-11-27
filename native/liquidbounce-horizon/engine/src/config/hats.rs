// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::hats::{Hats, MODES, ModeSettings};
#[derive(Clone, Debug, PartialEq)]
pub enum HatsSetting {
    Mode(usize),
    Height(Vec<[f32; 2]>),
    Period(i32),
    Symmetric(bool),
    Field(usize, String, Value),
    Render(BreadcrumbsSetting),
}
impl HatsSetting {
    pub fn apply(self, s: &mut Hats) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::Height(v) => s.height = v,
            Self::Period(v) => s.period = v,
            Self::Symmetric(v) => s.symmetric = v,
            Self::Render(v) => v.apply(&mut s.render),
            Self::Field(i, path, v) => {
                let m = &mut s.modes[i];
                let f = || v.as_f64().unwrap() as f32;
                let b = || v.as_bool().unwrap();
                let n = || v.as_i64().unwrap() as i32;
                match path.as_str() {
                    "FollowRotation" => m.follow_rotation = b(),
                    "EquipmentOffset.ArmorOffset" => m.armor_offset = f(),
                    "ShowDamage" => m.show_damage = b(),
                    "FriendsOptions.ViewOnFriend" => m.view_on_friend = b(),
                    "FriendsOptions.Distance" => m.distance = n(),
                    "FirstPersonView" => m.first_person = b(),
                    "HatSettings.Peak" => m.peak = f(),
                    "RadiusSettings.OuterRadius" | "HatSettings.Radius" => m.radius = f(),
                    "HatSettings.Thickness" => m.thickness = f(),
                    "HatSettings.Sharpness" => m.sharpness = f(),
                    "HatSettings.PetalCount"
                    | "HatSettings.PointsCount"
                    | "HatSettings.OrbsCount" => m.count = n(),
                    "HatSettings.SpinSpeed" | "SpinSpeed" => m.spin = f(),
                    "HatSettings.Speed" => m.speed = f(),
                    "HatSettings.OrbsSize" => m.size = f(),
                    "Wave.Enabled" => m.wave = b(),
                    "Wave.WaveHeight" => m.wave_height = f(),
                    "Wave.WaveSpeed" => m.wave_speed = f(),
                    "Colors.SyncColors" => m.colors.sync = b(),
                    "Colors.FirstColor" => m.colors.first = n(),
                    "Colors.SecondColor" => m.colors.second = n(),
                    "Colors.SpinSpeed" => m.colors.spin = f(),
                    "color" | "ColorModulator" => m.color = n(),
                    "Image" => m.image = v.as_str().unwrap().to_owned(),
                    "Scale" => {
                        m.scale = [
                            v["x"].as_f64().unwrap() as f32,
                            v["y"].as_f64().unwrap() as f32,
                        ]
                    }
                    _ => {}
                }
            }
        }
    }
}
struct Spec {
    path: &'static str,
    kind: &'static str,
    value: Value,
    range: Option<(f32, f32, &'static str)>,
}
fn fields(i: usize, s: &ModeSettings) -> Vec<Spec> {
    let mut a = vec![];
    macro_rules! v {
        ($p:expr,$k:expr,$v:expr) => {
            a.push(Spec {
                path: $p,
                kind: $k,
                value: json!($v),
                range: None,
            })
        };
        ($p:expr,$k:expr,$v:expr,$lo:expr,$hi:expr,$suffix:expr) => {
            a.push(Spec {
                path: $p,
                kind: $k,
                value: json!($v),
                range: Some(($lo as f32, $hi as f32, $suffix)),
            })
        };
    }
    v!("FollowRotation", "BOOLEAN", s.follow_rotation);
    v!(
        "EquipmentOffset.ArmorOffset",
        "FLOAT",
        s.armor_offset,
        0,
        1,
        ""
    );
    v!("ShowDamage", "BOOLEAN", s.show_damage);
    v!("FriendsOptions.ViewOnFriend", "BOOLEAN", s.view_on_friend);
    v!(
        "FriendsOptions.Distance",
        "INT",
        s.distance,
        8,
        512,
        "blocks"
    );
    v!("FirstPersonView", "BOOLEAN", s.first_person);
    match i {
        0 => {
            v!("HatSettings.Peak", "FLOAT", s.peak, 0.01, 2, "");
            v!("RadiusSettings.OuterRadius", "FLOAT", s.radius, 0.1, 2, "");
        }
        1 | 3 | 4 => {
            v!("HatSettings.Radius", "FLOAT", s.radius, 0.1, 2, "");
            v!("HatSettings.Thickness", "FLOAT", s.thickness, 0.01, 1, "");
            if i != 1 {
                v!(
                    "HatSettings.Sharpness",
                    "FLOAT",
                    s.sharpness,
                    0.1,
                    if i == 3 { 0.9 } else { 0.7 },
                    ""
                );
                v!(
                    if i == 3 {
                        "HatSettings.PetalCount"
                    } else {
                        "HatSettings.PointsCount"
                    },
                    "INT",
                    s.count,
                    5,
                    15,
                    ""
                );
                v!("HatSettings.SpinSpeed", "FLOAT", s.spin, -10, 10, "");
            }
        }
        2 => {
            v!("color", "COLOR", s.color);
            v!("HatSettings.Radius", "FLOAT", s.radius, 0, 2, "");
            v!("HatSettings.Speed", "FLOAT", s.speed, 0.1, 10, "");
            v!("HatSettings.OrbsSize", "FLOAT", s.size, 0.01, 0.5, "");
            v!("HatSettings.OrbsCount", "INT", s.count, 1, 12, "");
            v!("HatSettings.SpinSpeed", "FLOAT", s.spin, -10, 10, "");
            v!("Wave.Enabled", "BOOLEAN", s.wave);
            v!("Wave.WaveHeight", "FLOAT", s.wave_height, 0.01, 1, "");
            v!("Wave.WaveSpeed", "FLOAT", s.wave_speed, 0.1, 10, "");
        }
        5 => {
            v!("Image", "FILE", s.image);
            v!("ColorModulator", "COLOR", s.color);
            v!("Scale", "VECTOR2_F", json!({"x":s.scale[0],"y":s.scale[1]}));
            v!("SpinSpeed", "FLOAT", s.spin, -10, 10, "");
        }
        _ => {}
    }
    if matches!(i, 0 | 1 | 3 | 4) {
        v!("Colors.SyncColors", "BOOLEAN", s.colors.sync);
        v!("Colors.FirstColor", "COLOR", s.colors.first);
        v!("Colors.SecondColor", "COLOR", s.colors.second);
        v!("Colors.SpinSpeed", "FLOAT", s.colors.spin, 0, 10, "");
    }
    a
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.hats.{}",
        path.split('.')
            .map(|s| {
                let mut c = s.chars();
                c.next().map_or(String::new(), |a| {
                    a.to_lowercase().collect::<String>() + c.as_str()
                })
            })
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn group_fields(t: &Tree, i: usize, specs: &[Spec], prefix: &str) -> Vec<Value> {
    let mut out = vec![];
    let mut index = 0;
    while index < specs.len() {
        let spec = &specs[index];
        let path = spec.path.strip_prefix(prefix).unwrap();
        if let Some((group, _)) = path.split_once('.') {
            let nested = format!("{prefix}{group}.");
            let end = (index..specs.len())
                .find(|&j| !specs[j].path.starts_with(&nested))
                .unwrap_or(specs.len());
            out.push(t.group(
                group,
                if group == "Wave" {
                    "TOGGLEABLE"
                } else {
                    "CONFIGURABLE"
                },
                group_fields(t, i, &specs[index..end], &nested),
            ));
            index = end;
        } else {
            let k = key(&format!("Mode.{}.{}", MODES[i], spec.path));
            let mut leaf = if let Some((lo, hi, suffix)) = spec.range {
                t.ranged(
                    path,
                    spec.kind,
                    spec.value.clone(),
                    if spec.kind == "INT" {
                        json!({"from":lo as i32,"to":hi as i32})
                    } else {
                        json!({"from":lo,"to":hi})
                    },
                    suffix,
                    &k,
                )
            } else {
                t.leaf(path, spec.kind, spec.value.clone(), &k, None)
            };
            if t.format == JsonFormat::Interop && spec.kind == "FILE" {
                leaf["dialogMode"] = json!("OPEN_FILE");
                leaf["supportedExtensions"] = json!(["png", "jpg", "jpeg"]);
            }
            out.push(leaf);
            index += 1;
        }
    }
    out
}
fn rename(v: &mut Value, describe: &dyn Fn(&str) -> String) {
    match v {
        Value::String(s) => {
            *s = s.replace(
                "liquidbounce.module.breadcrumbs",
                "liquidbounce.module.hats",
            )
        }
        Value::Array(a) => {
            for v in a {
                rename(v, describe)
            }
        }
        Value::Object(o) => {
            for v in o.values_mut() {
                rename(v, describe)
            }
            if let Some(k) = o.get("key").and_then(Value::as_str) {
                o.insert(
                    "description".into(),
                    json!(describe(&format!("{k}.description"))),
                );
            }
        }
        _ => {}
    }
}
pub fn serialize_hats(
    s: &Hats,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    for (i, specs) in s.modes.iter().enumerate().map(|(i, m)| (i, fields(i, m))) {
        for spec in specs {
            if spec.kind == "FLOAT" && !spec.value.is_number() {
                return Err(ConfigError::new(
                    &format!("Hats.Mode.{}.{}", MODES[i], spec.path),
                    "non-finite float",
                ));
            }
        }
    }
    for p in &s.height {
        float_json(p[0], "Hats.HeightOffset.Height.x")?;
        float_json(p[1], "Hats.HeightOffset.Height.y")?;
    }
    for v in s.modes[5].scale {
        float_json(v, "Hats.Mode.Image.Scale")?;
    }
    let t = Tree { format, describe };
    let mut out = t.common(ModuleKind::Hats, common);
    let mut curve = t.leaf(
        "Height",
        "CURVE",
        json!(
            s.height
                .iter()
                .map(|p| json!({"x":p[0],"y":p[1]}))
                .collect::<Vec<_>>()
        ),
        &key("HeightOffset.Height"),
        None,
    );
    if format == JsonFormat::Interop {
        curve["innerType"] = json!("VECTOR2_F");
        curve["xAxis"] = json!({"label":"Progress","range":{"from":0,"to":1}});
        curve["yAxis"] = json!({"label":"Offset","range":{"from":0,"to":2}});
        curve["tension"] = json!(0.4);
    }
    out.push(t.group(
        "HeightOffset",
        "CONFIGURABLE",
        vec![
            t.ranged(
                "Period",
                "INT",
                json!(s.period),
                json!({"from":10,"to":20000}),
                "ms",
                &key("HeightOffset.Period"),
            ),
            t.leaf(
                "Symmetric",
                "BOOLEAN",
                json!(s.symmetric),
                &key("HeightOffset.Symmetric"),
                None,
            ),
            curve,
        ],
    ));
    let choices = MODES
        .iter()
        .enumerate()
        .map(|(i, name)| {
            (
                name.to_string(),
                t.group(
                    name,
                    "CONFIGURABLE",
                    group_fields(&t, i, &fields(i, &s.modes[i]), ""),
                ),
            )
        })
        .collect();
    let mut mode = t.mode(MODES[s.mode], choices);
    if format == JsonFormat::Interop {
        mode["key"] = json!(key("Mode"));
        mode["description"] = json!(describe(&format!("{}.description", key("Mode"))));
    }
    out.push(mode);
    let other = breadcrumbs::serialize_breadcrumbs(&s.render, common, format, describe)?;
    if let Some(mut render) = other["value"]
        .as_array()
        .and_then(|a| a.iter().find(|v| v["name"] == "Render"))
        .cloned()
    {
        rename(&mut render, describe);
        out.push(render);
    }
    Ok(t.group("Hats", "TOGGLEABLE", out))
}
fn color(v: &Value, p: &str) -> Result<Value, ConfigError> {
    let n = match v {
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
    n.map(|v| json!(v))
        .ok_or_else(|| ConfigError::new(p, "invalid ARGB color"))
}
fn read_fields(patch: &mut SettingsPatch, i: usize, root: &Nodes, prefix: &str) {
    let defaults = ModeSettings::for_mode(i);
    let specs = fields(i, &defaults);
    for spec in specs.iter().filter(|s| s.path.starts_with(prefix)) {
        let parts = spec
            .path
            .strip_prefix(prefix)
            .unwrap()
            .split('.')
            .collect::<Vec<_>>();
        let mut current = root.clone();
        let mut missing = false;
        for group in &parts[..parts.len() - 1] {
            let Some(v) = current.get(*group) else {
                missing = true;
                break;
            };
            match nodes(v, group, group) {
                Ok(v) => current = v,
                Err(e) => {
                    patch.issues.push(e);
                    missing = true;
                    break;
                }
            }
        }
        if missing {
            continue;
        }
        let name = parts.last().unwrap();
        let path = format!("Hats.Mode.{}.{}", MODES[i], spec.path);
        leaf(patch, &current, name, &path, |v, p| {
            let value = match spec.kind {
                "BOOLEAN" => json!(parse_bool(v, p)?),
                "FLOAT" => json!(parse_float(v, p)?),
                "INT" => json!(parse_int(v, p)?),
                "COLOR" => color(v, p)?,
                "FILE" => json!(
                    v.as_str()
                        .ok_or_else(|| ConfigError::new(p, "expected file path"))?
                ),
                "VECTOR2_F" => json!({"x":parse_float(&v["x"],p)?,"y":parse_float(&v["y"],p)?}),
                _ => unreachable!(),
            };
            Ok(SettingChange::Hats(HatsSetting::Field(
                i,
                spec.path.into(),
                value,
            )))
        });
    }
}
pub(super) fn parse_hats(patch: &mut SettingsPatch, root: &Nodes) {
    let change = |s| SettingChange::Hats(s);
    if let Some(v) = root.get("HeightOffset") {
        match nodes(v, "HeightOffset", "Hats.HeightOffset") {
            Ok(n) => {
                leaf(patch, &n, "Period", "Hats.HeightOffset.Period", |v, p| {
                    parse_int(v, p).map(|v| change(HatsSetting::Period(v)))
                });
                leaf(
                    patch,
                    &n,
                    "Symmetric",
                    "Hats.HeightOffset.Symmetric",
                    |v, p| parse_bool(v, p).map(|v| change(HatsSetting::Symmetric(v))),
                );
                leaf(patch, &n, "Height", "Hats.HeightOffset.Height", |v, p| {
                    let a = v
                        .as_array()
                        .ok_or_else(|| ConfigError::new(p, "expected curve"))?;
                    let points = a
                        .iter()
                        .map(|v| Ok([parse_float(&v["x"], p)?, parse_float(&v["y"], p)?]))
                        .collect::<Result<Vec<_>, ConfigError>>()?;
                    if points.is_empty() {
                        return Err(ConfigError::new(p, "curve must contain points"));
                    }
                    Ok(change(HatsSetting::Height(points)))
                });
            }
            Err(e) => patch.issues.push(e),
        }
    }
    if let Some(v) = root.get("Mode") {
        match v
            .get("active")
            .and_then(scalar_string)
            .and_then(|tag| MODES.iter().position(|m| *m == tag))
        {
            Some(i) => patch.changes.push(change(HatsSetting::Mode(i))),
            None => patch
                .issues
                .push(ConfigError::new("Hats.Mode", "unknown mode")),
        }
        if let Some(c) = v.get("choices").and_then(Value::as_object) {
            for (i, name) in MODES.iter().enumerate() {
                if let Some(v) = c.get(*name) {
                    match nodes(v, name, name) {
                        Ok(n) => read_fields(patch, i, &n, ""),
                        Err(e) => patch.issues.push(e),
                    }
                }
            }
        } else {
            patch
                .issues
                .push(ConfigError::new("Hats.Mode", "expected choices object"));
        }
    }
    if let Some(v) = root.get("Render") {
        let n = Nodes::from_iter([("Render".to_owned(), *v)]);
        let mut inner = SettingsPatch {
            module: ModuleKind::Hats,
            changes: vec![],
            issues: vec![],
        };
        breadcrumbs::parse_breadcrumbs(&mut inner, &n);
        for c in inner.changes {
            if let SettingChange::Breadcrumbs(v) = c {
                patch.changes.push(change(HatsSetting::Render(v)));
            }
        }
        for mut e in inner.issues {
            e.path = e.path.replace("Breadcrumbs", "Hats");
            patch.issues.push(e);
        }
    }
}
