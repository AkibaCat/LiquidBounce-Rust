// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::auto_walk_runtime::AutoWalkSettings;
use crate::edge_assistance_runtime::{ParkourSettings, SafeWalkSettings};
#[derive(Clone, Debug, PartialEq)]
pub enum ParkourSetting {
    Assist(bool),
    MaxAngle(f32),
    Rotation(NoRotateSetSetting),
    Render(bool),
    Covered(bool),
    Color(&'static str, u32),
}
impl ParkourSetting {
    pub fn apply(self, s: &mut ParkourSettings) {
        match self {
            Self::Assist(v) => s.assist = v,
            Self::MaxAngle(v) => s.max_angle = v,
            Self::Rotation(v) => {
                let mut proxy = AutoWalkSettings {
                    rotations: std::mem::take(&mut s.rotations),
                    ..Default::default()
                };
                AutoWalkSetting::Rotation(v).apply(&mut proxy);
                s.rotations = proxy.rotations;
            }
            Self::Render(v) => s.render.enabled = v,
            Self::Covered(v) => s.render.can_be_covered = v,
            Self::Color(k, v) => match k {
                "Arc" => s.render.arc = v,
                "TakeOff" => s.render.takeoff = v,
                "Landing" => s.render.landing = v,
                "Aim" => s.render.aim = v,
                _ => {}
            },
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum SafeWalkSetting {
    Mode(String),
    Distance([f32; 2]),
    Keep([i32; 2]),
    OnEdgeMode(String),
    Sneak([i32; 2]),
    Jump(bool),
}
impl SafeWalkSetting {
    pub fn apply(self, s: &mut SafeWalkSettings) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::Distance(v) => s.distance = v,
            Self::Keep(v) => s.keep = v,
            Self::OnEdgeMode(v) => s.on_edge_mode = v,
            Self::Sneak(v) => s.sneak = v,
            Self::Jump(v) => s.jump = v,
        }
    }
}
fn replace_keys(v: &mut Value, old: &str, new: &str) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(old, new);
            }
            for v in o.values_mut() {
                replace_keys(v, old, new);
            }
        }
        Value::Array(a) => {
            for v in a {
                replace_keys(v, old, new);
            }
        }
        _ => {}
    }
}
pub fn serialize_parkour(
    s: &ParkourSettings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::Parkour, c);
    let key = |s: &str| format!("liquidbounce.module.parkour.assist.{s}");
    let describe = |k: &str| {
        d(&k.replace(
            "liquidbounce.module.autoWalk.rotations",
            "liquidbounce.module.parkour.assist.rotations",
        ))
    };
    let proxy = AutoWalkSettings {
        rotations: s.rotations.clone(),
        ..Default::default()
    };
    let mut tree = serialize_auto_walk(&proxy, c, f, &describe)?;
    let mut rotations = tree["value"]
        .as_array_mut()
        .and_then(|a| a.iter_mut().find(|v| v["name"] == "Rotations"))
        .map(Value::take)
        .ok_or_else(|| {
            ConfigError::new("Parkour.Assist.Rotations", "shared rotation tree changed")
        })?;
    replace_keys(
        &mut rotations,
        "liquidbounce.module.autoWalk.rotations",
        "liquidbounce.module.parkour.assist.rotations",
    );
    let mut assist = vec![
        t.leaf("Enabled", "BOOLEAN", json!(s.assist), &key("enabled"), None),
        t.ranged(
            "MaxAngle",
            "FLOAT",
            float_json(s.max_angle, "Parkour.Assist.MaxAngle")?,
            json!({"from":5.,"to":90.}),
            "°",
            &key("maxAngle"),
        ),
        rotations,
    ];
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
    for (name, k, v) in [
        ("Arc", "arc", r.arc),
        ("TakeOff", "takeOff", r.takeoff),
        ("Landing", "landing", r.landing),
        ("Aim", "aim", r.aim),
    ] {
        render.push(t.leaf(
            name,
            "COLOR",
            json!(v as i32),
            &key(&format!("render.{k}")),
            None,
        ));
    }
    assist.push(t.group("Render", "TOGGLEABLE", render));
    values.push(t.group("Assist", "TOGGLEABLE", assist));
    Ok(t.group("Parkour", "TOGGLEABLE", values))
}
pub fn serialize_safe_walk(
    s: &SafeWalkSettings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::SafeWalk, c);
    let key = |s: &str| format!("liquidbounce.module.safeWalk.mode.onEdge.{s}");
    let mut choices = Map::new();
    for name in ["None", "Safe"] {
        choices.insert(name.into(), t.group(name, "CONFIGURABLE", vec![]));
    }
    let mut choice = t.leaf("Mode", "CHOOSE", json!(s.on_edge_mode), &key("mode"), None);
    if f == JsonFormat::Interop {
        choice["choices"] = json!(["Stop", "Invert", "Center"]);
    }
    let edge = vec![
        t.ranged(
            "Distance",
            "FLOAT_RANGE",
            float_range(
                s.distance[0],
                s.distance[1],
                "SafeWalk.Mode.OnEdge.Distance",
            )?,
            json!({"from":0.05,"to":0.5}),
            "",
            &key("distance"),
        ),
        t.ranged(
            "Keep",
            "INT_RANGE",
            json!({"from":s.keep[0],"to":s.keep[1]}),
            json!({"from":1,"to":20}),
            "ticks",
            &key("keep"),
        ),
        choice,
        t.ranged(
            "Sneak",
            "INT_RANGE",
            json!({"from":s.sneak[0],"to":s.sneak[1]}),
            json!({"from":0,"to":20}),
            "ticks",
            &key("sneak"),
        ),
        t.leaf("Jump", "BOOLEAN", json!(s.jump), &key("jump"), None),
    ];
    choices.insert("OnEdge".into(), t.group("OnEdge", "CONFIGURABLE", edge));
    values.push(t.mode(&s.mode, choices));
    Ok(t.group("SafeWalk", "TOGGLEABLE", values))
}
fn color(v: &Value, path: &str) -> Result<u32, ConfigError> {
    if let Some(n) = v.as_i64() {
        return Ok(n as u32);
    }
    if let Some(n) = v.as_u64() {
        return Ok(n as u32);
    }
    if let Some(s) = v.as_str() {
        let s = s.strip_prefix('#').unwrap_or(s);
        if s.len() != 6 && s.len() != 8 {
            return Err(ConfigError::new(path, "expected 6 or 8 hex digits"));
        }
        let n = u32::from_str_radix(s, 16).map_err(|e| ConfigError::new(path, e.to_string()))?;
        return Ok(if s.len() == 6 { n | 0xff000000 } else { n });
    }
    Err(ConfigError::new(path, "expected integer or hex color"))
}
pub(super) fn parse_parkour(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(a) = root.get("Assist") else {
        return;
    };
    let a = match nodes(a, "Assist", "Parkour.Assist") {
        Ok(v) => v,
        Err(e) => {
            p.issues.push(e);
            return;
        }
    };
    leaf(p, &a, "Enabled", "Parkour.Assist.Enabled", |v, path| {
        parse_bool(v, path).map(|v| SettingChange::Parkour(ParkourSetting::Assist(v)))
    });
    leaf(p, &a, "MaxAngle", "Parkour.Assist.MaxAngle", |v, path| {
        parse_float(v, path).map(|v| SettingChange::Parkour(ParkourSetting::MaxAngle(v)))
    });
    if let Some(r) = a.get("Rotations") {
        let proxy = json!({"name":"AutoWalk","value":[r]});
        let n = nodes(&proxy, "AutoWalk", "AutoWalk").expect("owned proxy");
        let mut parsed = SettingsPatch {
            module: ModuleKind::AutoWalk,
            changes: Vec::new(),
            issues: Vec::new(),
        };
        auto_walk::parse_auto_walk(&mut parsed, &n);
        for c in parsed.changes {
            if let SettingChange::AutoWalk(AutoWalkSetting::Rotation(v)) = c {
                p.changes
                    .push(SettingChange::Parkour(ParkourSetting::Rotation(v)));
            }
        }
        p.issues.extend(parsed.issues.into_iter().map(|mut e| {
            e.path = e
                .path
                .replace("AutoWalk.Rotations", "Parkour.Assist.Rotations");
            e
        }));
    }
    if let Some(r) = a.get("Render") {
        match nodes(r, "Render", "Parkour.Assist.Render") {
            Ok(r) => {
                leaf(
                    p,
                    &r,
                    "Enabled",
                    "Parkour.Assist.Render.Enabled",
                    |v, path| {
                        parse_bool(v, path)
                            .map(|v| SettingChange::Parkour(ParkourSetting::Render(v)))
                    },
                );
                leaf(
                    p,
                    &r,
                    "CanBeCovered",
                    "Parkour.Assist.Render.CanBeCovered",
                    |v, path| {
                        parse_bool(v, path)
                            .map(|v| SettingChange::Parkour(ParkourSetting::Covered(v)))
                    },
                );
                for name in ["Arc", "TakeOff", "Landing", "Aim"] {
                    leaf(
                        p,
                        &r,
                        name,
                        &format!("Parkour.Assist.Render.{name}"),
                        |v, path| {
                            color(v, path)
                                .map(|v| SettingChange::Parkour(ParkourSetting::Color(name, v)))
                        },
                    );
                }
            }
            Err(e) => p.issues.push(e),
        }
    }
}
pub(super) fn parse_safe_walk(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    match mode.get("active").and_then(scalar_string) {
        Some(v) if ["None", "Safe", "OnEdge"].contains(&v.as_str()) => p
            .changes
            .push(SettingChange::SafeWalk(SafeWalkSetting::Mode(v))),
        _ => p.issues.push(ConfigError::new(
            "SafeWalk.Mode.active",
            "missing or unknown case-sensitive mode",
        )),
    }
    let Some(choices) = mode["choices"].as_object() else {
        p.issues.push(ConfigError::new(
            "SafeWalk.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for name in ["None", "Safe", "OnEdge"] {
        let path = format!("SafeWalk.Mode.{name}");
        let Some(v) = choices.get(name) else {
            p.issues.push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        match nodes(v, name, &path) {
            Ok(n) if name == "OnEdge" => {
                leaf(
                    p,
                    &n,
                    "Distance",
                    "SafeWalk.Mode.OnEdge.Distance",
                    |v, path| {
                        let a = parse_float(&v["from"], path)?;
                        let b = parse_float(&v["to"], path)?;
                        if a > b {
                            return Err(ConfigError::new(path, "range must be ascending"));
                        }
                        Ok(SettingChange::SafeWalk(SafeWalkSetting::Distance([a, b])))
                    },
                );
                for (name, create) in [
                    (
                        "Keep",
                        SafeWalkSetting::Keep as fn([i32; 2]) -> SafeWalkSetting,
                    ),
                    (
                        "Sneak",
                        SafeWalkSetting::Sneak as fn([i32; 2]) -> SafeWalkSetting,
                    ),
                ] {
                    leaf(
                        p,
                        &n,
                        name,
                        &format!("SafeWalk.Mode.OnEdge.{name}"),
                        |v, path| {
                            let a = parse_int(&v["from"], path)?;
                            let b = parse_int(&v["to"], path)?;
                            if a > b {
                                return Err(ConfigError::new(path, "range must be ascending"));
                            }
                            Ok(SettingChange::SafeWalk(create([a, b])))
                        },
                    );
                }
                leaf(
                    p,
                    &n,
                    "Mode",
                    "SafeWalk.Mode.OnEdge.Mode",
                    |v, path| match scalar_string(v) {
                        Some(v) if ["Stop", "Invert", "Center"].contains(&v.as_str()) => {
                            Ok(SettingChange::SafeWalk(SafeWalkSetting::OnEdgeMode(v)))
                        }
                        _ => Err(ConfigError::new(path, "unknown case-sensitive edge mode")),
                    },
                );
                leaf(p, &n, "Jump", "SafeWalk.Mode.OnEdge.Jump", |v, path| {
                    parse_bool(v, path).map(|v| SettingChange::SafeWalk(SafeWalkSetting::Jump(v)))
                });
            }
            Ok(_) => {}
            Err(e) => p.issues.push(e),
        }
    }
}
