// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::wings::Wings;
#[derive(Clone, Debug, PartialEq)]
pub enum WingsSetting {
    Field(String, Value),
    Render(BreadcrumbsSetting),
}
impl WingsSetting {
    pub fn apply(self, s: &mut Wings) {
        match self {
            Self::Render(v) => v.apply(&mut s.render),
            Self::Field(path, v) => {
                let f = || v.as_f64().unwrap() as f32;
                let b = || v.as_bool().unwrap();
                let n = || v.as_i64().unwrap() as i32;
                match path.as_str() {
                    "Position.Height" => s.height = f(),
                    "Position.BackOffset" => s.back_offset = f(),
                    "Position.EquipmentOffset" => s.equipment_offset = f(),
                    "ShowDamage" => s.show_damage = b(),
                    "ShowInFirstPerson" => s.first_person = b(),
                    "FriendsOptions.ViewOnFriend" => s.view_on_friend = b(),
                    "FriendsOptions.Distance" => s.distance = n(),
                    "Length" => s.length = f(),
                    "Width" => s.width = f(),
                    "FadeStartRatio" => s.fade_start = f(),
                    "LinesCount" => s.count = n(),
                    "Angles" => {
                        s.angles = [
                            v["from"].as_i64().unwrap() as i32,
                            v["to"].as_i64().unwrap() as i32,
                        ]
                    }
                    "Color" => s.color = n(),
                    _ => {}
                }
            }
        }
    }
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.wings.{}",
        path.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn render_key(v: &mut Value, describe: &dyn Fn(&str) -> String) {
    match v {
        Value::String(s) => {
            *s = s.replace(
                "liquidbounce.module.breadcrumbs",
                "liquidbounce.module.wings.mode.lines",
            )
        }
        Value::Array(a) => {
            for v in a {
                render_key(v, describe)
            }
        }
        Value::Object(o) => {
            for v in o.values_mut() {
                render_key(v, describe)
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
pub fn serialize_wings(
    s: &Wings,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut out = t.common(ModuleKind::Wings, common);
    let f = |name: &str, path: &str, v: f32, lo: f32, hi: f32| -> Result<Value, ConfigError> {
        Ok(t.ranged(
            name,
            "FLOAT",
            float_json(v, path)?,
            json!({"from":lo,"to":hi}),
            "",
            &key(path),
        ))
    };
    out.push(t.group(
        "Position",
        "CONFIGURABLE",
        vec![
            f("Height", "Position.Height", s.height, -1., 1.)?,
            f("BackOffset", "Position.BackOffset", s.back_offset, 0., 0.5)?,
            f(
                "EquipmentOffset",
                "Position.EquipmentOffset",
                s.equipment_offset,
                0.,
                1.,
            )?,
        ],
    ));
    let p = "Mode.Lines";
    let mut mode = vec![
        t.leaf(
            "ShowDamage",
            "BOOLEAN",
            json!(s.show_damage),
            &key(&format!("{p}.ShowDamage")),
            None,
        ),
        t.leaf(
            "ShowInFirstPerson",
            "BOOLEAN",
            json!(s.first_person),
            &key(&format!("{p}.ShowInFirstPerson")),
            None,
        ),
        t.group(
            "FriendsOptions",
            "CONFIGURABLE",
            vec![
                t.leaf(
                    "ViewOnFriend",
                    "BOOLEAN",
                    json!(s.view_on_friend),
                    &key(&format!("{p}.FriendsOptions.ViewOnFriend")),
                    None,
                ),
                t.ranged(
                    "Distance",
                    "INT",
                    json!(s.distance),
                    json!({"from":8,"to":512}),
                    "blocks",
                    &key(&format!("{p}.FriendsOptions.Distance")),
                ),
            ],
        ),
    ];
    for (name, value, lo, hi) in [
        ("Length", s.length, 0.1, 2.),
        ("Width", s.width, 0.01, 1.),
        ("FadeStartRatio", s.fade_start, 0.1, 1.),
    ] {
        mode.push(f(name, &format!("{p}.{name}"), value, lo, hi)?);
    }
    mode.push(t.ranged(
        "LinesCount",
        "INT",
        json!(s.count),
        json!({"from":1,"to":12}),
        "",
        &key(&format!("{p}.LinesCount")),
    ));
    mode.push(t.ranged(
        "Angles",
        "INT_RANGE",
        json!({"from":s.angles[0],"to":s.angles[1]}),
        json!({"from":1,"to":90}),
        "",
        &key(&format!("{p}.Angles")),
    ));
    mode.push(t.leaf(
        "Color",
        "COLOR",
        json!(s.color),
        &key(&format!("{p}.Color")),
        None,
    ));
    let tree = breadcrumbs::serialize_breadcrumbs(&s.render, common, format, describe)?;
    let mut render = tree["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Render")
        .unwrap()
        .clone();
    render_key(&mut render, describe);
    mode.push(render);
    let mut choice = t.mode(
        "Lines",
        Map::from_iter([("Lines".into(), t.group("Lines", "CONFIGURABLE", mode))]),
    );
    if format == JsonFormat::Interop {
        choice["key"] = json!(key("Mode"));
        choice["description"] = json!(describe(&format!("{}.description", key("Mode"))));
    }
    out.push(choice);
    Ok(t.group("Wings", "TOGGLEABLE", out))
}
fn read_field(patch: &mut SettingsPatch, n: &Nodes, name: &str, path: &str, kind: &str) {
    leaf(patch, n, name, &format!("Wings.{path}"), |v, p| {
        let value = match kind {
            "BOOLEAN" => json!(parse_bool(v, p)?),
            "INT" => json!(parse_int(v, p)?),
            "FLOAT" => json!(parse_float(v, p)?),
            "INT_RANGE" => {
                let a = parse_int(&v["from"], p)?;
                let b = parse_int(&v["to"], p)?;
                if a > b {
                    return Err(ConfigError::new(p, "reversed range"));
                }
                json!({"from":a,"to":b})
            }
            "COLOR" => {
                let number = if let Some(s) = v.as_str() {
                    let hex = s.strip_prefix('#').unwrap_or(s);
                    if !matches!(hex.len(), 6 | 8) {
                        return Err(ConfigError::new(p, "expected 6/8 hex digits"));
                    }
                    i64::from_str_radix(hex, 16).ok().map(|x| {
                        x as i32
                            | if hex.len() == 6 {
                                0xff000000u32 as i32
                            } else {
                                0
                            }
                    })
                } else {
                    v.as_number().and_then(|n| decimal_int(&n.to_string()))
                };
                json!(number.ok_or_else(|| ConfigError::new(p, "invalid ARGB color"))?)
            }
            _ => unreachable!(),
        };
        Ok(SettingChange::Wings(WingsSetting::Field(
            path.to_owned(),
            value,
        )))
    })
}
pub(super) fn parse_wings(patch: &mut SettingsPatch, root: &Nodes) {
    if let Some(v) = root.get("Position") {
        match nodes(v, "Position", "Wings.Position") {
            Ok(n) => {
                for name in ["Height", "BackOffset", "EquipmentOffset"] {
                    read_field(patch, &n, name, &format!("Position.{name}"), "FLOAT")
                }
            }
            Err(e) => patch.issues.push(e),
        }
    }
    let Some(mode) = root.get("Mode") else { return };
    if mode["active"].as_str() != Some("Lines") {
        patch
            .issues
            .push(ConfigError::new("Wings.Mode", "unknown mode"));
    }
    let Some(v) = mode.get("choices").and_then(|v| v.get("Lines")) else {
        patch
            .issues
            .push(ConfigError::new("Wings.Mode.Lines", "choice not found"));
        return;
    };
    let n = match nodes(v, "Lines", "Wings.Mode.Lines") {
        Ok(n) => n,
        Err(e) => {
            patch.issues.push(e);
            return;
        }
    };
    for (name, kind) in [
        ("ShowDamage", "BOOLEAN"),
        ("ShowInFirstPerson", "BOOLEAN"),
        ("Length", "FLOAT"),
        ("Width", "FLOAT"),
        ("FadeStartRatio", "FLOAT"),
        ("LinesCount", "INT"),
        ("Angles", "INT_RANGE"),
        ("Color", "COLOR"),
    ] {
        read_field(patch, &n, name, name, kind)
    }
    if let Some(v) = n.get("FriendsOptions") {
        match nodes(v, "FriendsOptions", "Wings.Mode.Lines.FriendsOptions") {
            Ok(n) => {
                read_field(
                    patch,
                    &n,
                    "ViewOnFriend",
                    "FriendsOptions.ViewOnFriend",
                    "BOOLEAN",
                );
                read_field(patch, &n, "Distance", "FriendsOptions.Distance", "INT")
            }
            Err(e) => patch.issues.push(e),
        }
    }
    if let Some(v) = n.get("Render") {
        let mut inner = SettingsPatch {
            module: ModuleKind::Wings,
            changes: vec![],
            issues: vec![],
        };
        breadcrumbs::parse_breadcrumbs(&mut inner, &Nodes::from_iter([("Render".into(), *v)]));
        for c in inner.changes {
            if let SettingChange::Breadcrumbs(v) = c {
                patch
                    .changes
                    .push(SettingChange::Wings(WingsSetting::Render(v)))
            }
        }
        for mut e in inner.issues {
            e.path = e.path.replace("Breadcrumbs", "Wings.Mode.Lines");
            patch.issues.push(e)
        }
    }
}
