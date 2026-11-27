// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::auto_walk_runtime::AutoWalkSettings;
use liquidbounce_behaviors::no_rotate_set::NoRotateSet;
#[derive(Clone, Debug, PartialEq)]
pub enum AutoWalkSetting {
    Mode(String),
    Destination([i32; 3]),
    Target(String),
    Distance(f32),
    Boolean(&'static str, bool),
    Integer(&'static str, i32),
    Color(&'static str, u32),
    Rotation(NoRotateSetSetting),
}
impl AutoWalkSetting {
    pub fn apply(self, s: &mut AutoWalkSettings) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::Destination(v) => s.destination = v,
            Self::Target(v) => s.target = v,
            Self::Distance(v) => s.distance = v,
            Self::Boolean(k, v) => match k {
                "AllowBreak" => s.allow_break = v,
                "AllowPlace" => s.allow_place = v,
                "AllowParkour" => s.allow_parkour = v,
                "RiskyJumps" => s.risky_jumps = v,
                "AllowFly" => s.allow_fly = v,
                "AllowElytra" => s.allow_elytra = v,
                "ElytraRockets" => s.elytra_rockets = v,
                "AllowBoat" => s.allow_boat = v,
                "AllowEat" => s.allow_eat = v,
                "Render.Enabled" => s.render.enabled = v,
                "CanBeCovered" => s.render.can_be_covered = v,
                "Ghost" => s.render.ghost = v,
                _ => {}
            },
            Self::Integer(k, v) => match k {
                "MaxFall" => s.max_fall = v,
                "PathRange" => s.path_range = v,
                "FadeOut" => s.render.fade_out = v,
                _ => {}
            },
            Self::Color(k, v) => match k {
                "Route" => s.render.route = v,
                "Search" => s.render.search = v,
                "Destination" => s.render.destination = v,
                "Break" => s.render.breaks = v,
                "Place" => s.render.places = v,
                "Target" => s.render.target = v,
                "Invalid" => s.render.invalid = v,
                _ => {}
            },
            Self::Rotation(v) => {
                let mut proxy = NoRotateSet {
                    rotations: std::mem::take(&mut s.rotations),
                    ..Default::default()
                };
                v.apply(&mut proxy);
                s.rotations = proxy.rotations;
            }
        }
    }
}
const OLD_KEY: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const ROT_KEY: &str = "liquidbounce.module.autoWalk.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "AutoWalk.Rotations",
        )
        .replace(OLD_KEY, ROT_KEY);
    e
}
fn rotation_tree(
    s: &AutoWalkSettings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let describe = |key: &str| d(&key.replace(OLD_KEY, ROT_KEY));
    let proxy = NoRotateSet {
        rotations: s.rotations.clone(),
        ..Default::default()
    };
    let mut tree = serialize_no_rotate_set(&proxy, c, f, &describe).map_err(remap)?;
    let mut rotations = tree["value"]
        .as_array_mut()
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|v| v.first_mut())
        .map(Value::take)
        .ok_or_else(|| {
            ConfigError::new("AutoWalk.Rotations", "shared Rotations codec shape changed")
        })?;
    fn keys(v: &mut Value) {
        match v {
            Value::Object(o) => {
                if let Some(Value::String(k)) = o.get_mut("key") {
                    *k = k.replace(OLD_KEY, ROT_KEY);
                }
                for v in o.values_mut() {
                    keys(v);
                }
            }
            Value::Array(a) => {
                for v in a {
                    keys(v);
                }
            }
            _ => {}
        }
    }
    keys(&mut rotations);
    Ok(rotations)
}
pub fn serialize_auto_walk(
    s: &AutoWalkSettings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::AutoWalk, c);
    let key = |s: &str| format!("liquidbounce.module.autoWalk.{s}");
    let mut choices = Map::new();
    for name in ["Forward", "Direction"] {
        choices.insert(name.into(), t.group(name, "CONFIGURABLE", vec![]));
    }
    let mut destination = t.leaf(
        "Destination",
        "VECTOR3_I",
        json!({"x":s.destination[0],"y":s.destination[1],"z":s.destination[2]}),
        &key("mode.goto.destination"),
        None,
    );
    if f == JsonFormat::Interop {
        destination["useLocateButton"] = json!(true);
    }
    choices.insert(
        "Goto".into(),
        t.group("Goto", "CONFIGURABLE", vec![destination]),
    );
    choices.insert(
        "Follow".into(),
        t.group(
            "Follow",
            "CONFIGURABLE",
            vec![
                t.leaf(
                    "Target",
                    "TEXT",
                    json!(s.target),
                    &key("mode.follow.target"),
                    None,
                ),
                t.ranged(
                    "Distance",
                    "FLOAT",
                    float_json(s.distance, "AutoWalk.Mode.Follow.Distance")?,
                    json!({"from":1.,"to":10.}),
                    "blocks",
                    &key("mode.follow.distance"),
                ),
            ],
        ),
    );
    values.push(t.mode(&s.mode, choices));
    for (name, k, v) in [
        ("AllowBreak", "allowBreak", s.allow_break),
        ("AllowPlace", "allowPlace", s.allow_place),
        ("AllowParkour", "allowParkour", s.allow_parkour),
        ("RiskyJumps", "riskyJumps", s.risky_jumps),
    ] {
        values.push(t.leaf(name, "BOOLEAN", json!(v), &key(k), None));
    }
    values.push(t.ranged(
        "MaxFall",
        "INT",
        json!(s.max_fall),
        json!({"from":0,"to":20}),
        "blocks",
        &key("maxFall"),
    ));
    values.push(t.ranged(
        "PathRange",
        "INT",
        json!(s.path_range),
        json!({"from":16,"to":128}),
        "blocks",
        &key("pathRange"),
    ));
    for (name, k, v) in [
        ("AllowFly", "allowFly", s.allow_fly),
        ("AllowElytra", "allowElytra", s.allow_elytra),
        ("ElytraRockets", "elytraRockets", s.elytra_rockets),
        ("AllowBoat", "allowBoat", s.allow_boat),
        ("AllowEat", "allowEat", s.allow_eat),
    ] {
        values.push(t.leaf(name, "BOOLEAN", json!(v), &key(k), None));
    }
    values.push(rotation_tree(s, c, f, d)?);
    let r = &s.render;
    let mut render = vec![
        t.leaf(
            "Enabled",
            "BOOLEAN",
            json!(r.enabled),
            &key("render.enabled"),
            None,
        ),
        t.leaf(
            "CanBeCovered",
            "BOOLEAN",
            json!(r.can_be_covered),
            &key("render.canBeCovered"),
            None,
        ),
    ];
    for (name, k, color) in [
        ("Route", "route", r.route),
        ("Search", "search", r.search),
        ("Destination", "destination", r.destination),
        ("Break", "break", r.breaks),
        ("Place", "place", r.places),
        ("Target", "target", r.target),
        ("Invalid", "invalid", r.invalid),
    ] {
        render.push(t.leaf(
            name,
            "COLOR",
            json!(color as i32),
            &key(&format!("render.{k}")),
            None,
        ));
    }
    render.push(t.leaf(
        "Ghost",
        "BOOLEAN",
        json!(r.ghost),
        &key("render.ghost"),
        None,
    ));
    render.push(t.ranged(
        "FadeOut",
        "INT",
        json!(r.fade_out),
        json!({"from":0,"to":3000}),
        "ms",
        &key("render.fadeOut"),
    ));
    values.push(t.group("Render", "TOGGLEABLE", render));
    Ok(t.group("AutoWalk", "TOGGLEABLE", values))
}
fn ch(v: AutoWalkSetting) -> SettingChange {
    SettingChange::AutoWalk(v)
}
fn parse_rotation(p: &mut SettingsPatch, v: &Value) {
    let proxy = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[v]}}}]});
    let root = nodes(&proxy, "NoRotateSet", "NoRotateSet").unwrap();
    let mut parsed = SettingsPatch {
        module: ModuleKind::NoRotateSet,
        changes: Vec::new(),
        issues: Vec::new(),
    };
    no_rotate_set::parse_no_rotate_set(&mut parsed, &root);
    for c in parsed.changes {
        if let SettingChange::NoRotateSet(v) = c
            && !matches!(v, NoRotateSetSetting::Mode(_))
        {
            p.changes.push(ch(AutoWalkSetting::Rotation(v)));
        }
    }
    p.issues.extend(parsed.issues.into_iter().map(remap));
}
pub(super) fn parse_auto_walk(p: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(mode) = root.get("Mode") {
        match mode.get("active").and_then(scalar_string) {
            Some(v) if ["Forward", "Direction", "Goto", "Follow"].contains(&v.as_str()) => {
                p.changes.push(ch(AutoWalkSetting::Mode(v)))
            }
            _ => p.issues.push(ConfigError::new(
                "AutoWalk.Mode.active",
                "missing or unknown case-sensitive mode",
            )),
        }
        if let Some(choices) = mode["choices"].as_object() {
            for name in ["Forward", "Direction", "Goto", "Follow"] {
                if let Some(v) = choices.get(name) {
                    match nodes(v, name, &format!("AutoWalk.Mode.{name}")) {
                        Ok(n) if name == "Goto" => leaf(
                            p,
                            &n,
                            "Destination",
                            "AutoWalk.Mode.Goto.Destination",
                            |v, path| {
                                let o = v.as_object().ok_or_else(|| {
                                    ConfigError::new(path, "expected vector object")
                                })?;
                                let mut at = [0; 3];
                                for (i, k) in ["x", "y", "z"].into_iter().enumerate() {
                                    if let Some(v) = o.get(k) {
                                        at[i] = parse_int(v, path)?;
                                    }
                                }
                                Ok(ch(AutoWalkSetting::Destination(at)))
                            },
                        ),
                        Ok(n) if name == "Follow" => {
                            leaf(p, &n, "Target", "AutoWalk.Mode.Follow.Target", |v, path| {
                                scalar_string(v)
                                    .map(|v| ch(AutoWalkSetting::Target(v)))
                                    .ok_or_else(|| ConfigError::new(path, "expected text"))
                            });
                            leaf(
                                p,
                                &n,
                                "Distance",
                                "AutoWalk.Mode.Follow.Distance",
                                |v, path| {
                                    parse_float(v, path).map(|v| ch(AutoWalkSetting::Distance(v)))
                                },
                            );
                        }
                        Ok(_) => {}
                        Err(e) => p.issues.push(e),
                    }
                } else {
                    p.issues.push(ConfigError::new(
                        &format!("AutoWalk.Mode.{name}"),
                        "choice not found",
                    ));
                }
            }
        } else {
            p.issues.push(ConfigError::new(
                "AutoWalk.Mode.choices",
                "expected choices object",
            ));
        }
    }
    for name in [
        "AllowBreak",
        "AllowPlace",
        "AllowParkour",
        "RiskyJumps",
        "AllowFly",
        "AllowElytra",
        "ElytraRockets",
        "AllowBoat",
        "AllowEat",
    ] {
        leaf(p, root, name, &format!("AutoWalk.{name}"), |v, path| {
            parse_bool(v, path).map(|v| ch(AutoWalkSetting::Boolean(name, v)))
        });
    }
    for name in ["MaxFall", "PathRange"] {
        leaf(p, root, name, &format!("AutoWalk.{name}"), |v, path| {
            parse_int(v, path).map(|v| ch(AutoWalkSetting::Integer(name, v)))
        });
    }
    if let Some(v) = root.get("Rotations") {
        parse_rotation(p, v);
    }
    if let Some(v) = root.get("Render") {
        match nodes(v, "Render", "AutoWalk.Render") {
            Ok(n) => {
                leaf(p, &n, "Enabled", "AutoWalk.Render.Enabled", |v, path| {
                    parse_bool(v, path).map(|v| ch(AutoWalkSetting::Boolean("Render.Enabled", v)))
                });
                for name in ["CanBeCovered", "Ghost"] {
                    leaf(
                        p,
                        &n,
                        name,
                        &format!("AutoWalk.Render.{name}"),
                        |v, path| {
                            parse_bool(v, path).map(|v| ch(AutoWalkSetting::Boolean(name, v)))
                        },
                    );
                }
                for name in [
                    "Route",
                    "Search",
                    "Destination",
                    "Break",
                    "Place",
                    "Target",
                    "Invalid",
                ] {
                    leaf(
                        p,
                        &n,
                        name,
                        &format!("AutoWalk.Render.{name}"),
                        |v, path| {
                            let color = if v.is_number() {
                                v.as_i64()
                                    .map(|n| n as u32)
                                    .or_else(|| v.as_u64().map(|n| n as u32))
                                    .ok_or_else(|| {
                                        ConfigError::new(path, "expected integer color")
                                    })?
                            } else if let Some(v) = v.as_str() {
                                let text = v.strip_prefix("#").unwrap_or(v);
                                if text.len() != 6 && text.len() != 8 {
                                    return Err(ConfigError::new(
                                        path,
                                        "expected 6 or 8 hex digits",
                                    ));
                                }
                                let n = u32::from_str_radix(text, 16)
                                    .map_err(|e| ConfigError::new(path, e.to_string()))?;
                                if text.len() == 6 { n | 0xff000000 } else { n }
                            } else {
                                return Err(ConfigError::new(path, "expected number or hex color"));
                            };
                            Ok(ch(AutoWalkSetting::Color(name, color)))
                        },
                    );
                }
                leaf(p, &n, "FadeOut", "AutoWalk.Render.FadeOut", |v, path| {
                    parse_int(v, path).map(|v| ch(AutoWalkSetting::Integer("FadeOut", v)))
                });
            }
            Err(e) => p.issues.push(e),
        }
    }
}
