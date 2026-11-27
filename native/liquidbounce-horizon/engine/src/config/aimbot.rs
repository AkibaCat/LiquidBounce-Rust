// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::{aimbot_runtime::AimbotRuntime, auto_rod_runtime::NativeSamples};
use liquidbounce_behaviors::{
    aimbot::Smooth, auto_bow::PRIORITIES, no_rotate_set::FloatRange,
    target_renderer::TargetRendererSetting,
};
pub const REQUIRES: [&str; 6] = [
    "Click",
    "MouseMovement",
    "Weapon",
    "EmptyHand",
    "VanillaName",
    "NotBreaking",
];
pub const IGNORES: [&str; 2] = ["Screen", "Container"];
pub const AXIS: [&str; 2] = ["Horizontal", "Vertical"];
pub const PARTS: [&str; 3] = ["Head", "Body", "Feet"];
#[derive(Clone, Debug, PartialEq)]
pub enum AimbotSetting {
    Value(String, Value),
    Point(String, Value),
    Rendering(TargetRendererSetting),
}
fn choice(v: &Value, options: &[&str], p: &str) -> Result<usize, ConfigError> {
    let s = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected choice"))?;
    options
        .iter()
        .position(|o| o.eq_ignore_ascii_case(&s))
        .ok_or_else(|| ConfigError::new(p, "unknown choice"))
}
fn multi<const N: usize>(v: &Value, options: [&str; N], p: &str) -> Result<[bool; N], ConfigError> {
    let mut selected = [false; N];
    if let Some(a) = v.as_array() {
        for v in a {
            selected[choice(v, &options, p)?] = true;
        }
    } else if !v.is_null() {
        selected[choice(v, &options, p)?] = true;
    }
    Ok(selected)
}
fn floats(v: &Value, p: &str) -> Result<[f32; 2], ConfigError> {
    let r = parse_float_pair(v, p)?;
    if r[0] > r[1] {
        Err(ConfigError::new(p, "empty float range"))
    } else {
        Ok(r)
    }
}
fn ints(v: &Value, p: &str) -> Result<[i32; 2], ConfigError> {
    let r = parse_int_pair(v, p)?;
    if r[0] > r[1] {
        Err(ConfigError::new(p, "empty integer range"))
    } else {
        Ok(r)
    }
}
impl AimbotSetting {
    pub fn apply(self, r: &mut AimbotRuntime) -> Result<(), ConfigError> {
        match self {
            Self::Rendering(v) => v.apply(&mut r.target_renderer),
            Self::Point(path, v) => {
                let p = format!("Aimbot.AimPoint.{path}");
                let s = &mut r.point_tracker.settings;
                match path.as_str() {
                    "ExemptBoxParts" => s.exempt_parts = multi(&v, PARTS, &p)?,
                    "ExemptBestHitVector.Enabled" => s.exempt_best = parse_bool(&v, &p)?,
                    "ExemptBestHitVector.Vertical" => s.vertical = parse_float(&v, &p)?,
                    "ExemptBestHitVector.Horizontal" => s.horizontal = parse_float(&v, &p)?,
                    "Gaussian.Enabled" => s.gaussian = parse_bool(&v, &p)?,
                    "Gaussian.YawOffset" => s.yaw_offset = floats(&v, &p)?,
                    "Gaussian.PitchOffset" => s.pitch_offset = floats(&v, &p)?,
                    "Gaussian.Chance" => s.chance = parse_float(&v, &p)?,
                    "Gaussian.Speed" => s.speed = floats(&v, &p)?,
                    "Gaussian.Tolerance" => s.tolerance = parse_float(&v, &p)?,
                    "Gaussian.Dynamic.Enabled" => s.dynamic = parse_bool(&v, &p)?,
                    "Gaussian.Dynamic.HurtTime" => s.dynamic_hurt = parse_int(&v, &p)?,
                    "Gaussian.Dynamic.YawFactor" => s.dynamic_yaw = parse_float(&v, &p)?,
                    "Gaussian.Dynamic.PitchFactor" => s.dynamic_pitch = parse_float(&v, &p)?,
                    "Gaussian.Dynamic.Speed" => s.dynamic_speed = floats(&v, &p)?,
                    "Gaussian.Dynamic.Tolerance" => s.dynamic_tolerance = parse_float(&v, &p)?,
                    "Lazy.Enabled" => s.lazy = parse_bool(&v, &p)?,
                    "Lazy.Threshold" => {
                        let value = floats(&v, &p)?;
                        if value != s.threshold {
                            s.threshold = value;
                            r.point_tracker
                                .threshold_changed(&mut NativeSamples(&mut r.random));
                        }
                    }
                    "Delay.Enabled" => s.delay = parse_bool(&v, &p)?,
                    "Delay.Delay" => {
                        let value = ints(&v, &p)?;
                        if value != s.delay_ticks {
                            s.delay_ticks = value;
                            r.point_tracker
                                .delay_changed(&mut NativeSamples(&mut r.random));
                        }
                    }
                    _ => return Err(ConfigError::new(&p, "unknown AimPoint setting")),
                }
            }
            Self::Value(path, v) => {
                let p = format!("Aimbot.{path}");
                let s = &mut r.behavior.settings;
                match path.as_str() {
                    "Range" => s.range = parse_float(&v, &p)?,
                    "WallRange" => {
                        let wall = parse_float(&v, &p)?;
                        s.wall_range = if wall.is_nan() || s.range.is_nan() {
                            f32::NAN
                        } else {
                            wall.min(s.range)
                        };
                    }
                    "LazyRotation" => s.lazy = parse_bool(&v, &p)?,
                    "Crystals.Enabled" => s.crystals = parse_bool(&v, &p)?,
                    "Crystals.Only" => s.crystals_only = parse_bool(&v, &p)?,
                    "Crystals.OnlyWhileUsing" => s.crystals_using = parse_bool(&v, &p)?,
                    "Requires" => s.requires = multi(&v, REQUIRES, &p)?,
                    "Axis" => s.axis = multi(&v, AXIS, &p)?,
                    "Ignore" => s.ignore = multi(&v, IGNORES, &p)?,
                    "Target.FOV" => s.fov = parse_float(&v, &p)?,
                    "Target.HurtTime" => s.hurt_time = parse_int(&v, &p)?,
                    "Target.Priority" => {
                        let mut priorities = vec![];
                        let mut add = |v: &Value| -> Result<(), ConfigError> {
                            let name = scalar_string(v)
                                .ok_or_else(|| ConfigError::new(&p, "expected target priority"))?;
                            let q = PRIORITIES
                                .into_iter()
                                .find(|q| q.name().eq_ignore_ascii_case(&name))
                                .ok_or_else(|| ConfigError::new(&p, "unknown target priority"))?;
                            if !priorities.contains(&q) {
                                priorities.push(q);
                            }
                            Ok(())
                        };
                        if let Some(a) = v.as_array() {
                            for v in a {
                                add(v)?;
                            }
                        } else if !v.is_null() {
                            add(&v)?;
                        }
                        if priorities.is_empty() {
                            return Err(ConfigError::new(&p, "priority cannot be empty"));
                        }
                        s.priority = priorities;
                    }
                    "AngleSmooth" => {
                        s.smooth = Smooth::ALL
                            .into_iter()
                            .find(|m| scalar_string(&v).as_deref() == Some(m.tag()))
                            .ok_or_else(|| ConfigError::new(&p, "unknown case-sensitive mode"))?
                    }
                    "AngleSmooth.Interpolation.HorizontalSpeed" => {
                        s.interpolation.horizontal = ints(&v, &p)?
                    }
                    "AngleSmooth.Interpolation.VerticalSpeed" => {
                        s.interpolation.vertical = ints(&v, &p)?
                    }
                    "AngleSmooth.Interpolation.DirectionChangeFactor" => {
                        s.interpolation.direction_change = ints(&v, &p)?
                    }
                    "AngleSmooth.Interpolation.Midpoint" => {
                        s.interpolation.midpoint = parse_float(&v, &p)?
                    }
                    "AngleSmooth.Linear.HorizontalTurnSpeed" => {
                        let v = floats(&v, &p)?;
                        s.basic.linear.horizontal_turn_speed = FloatRange(v[0], v[1]);
                    }
                    "AngleSmooth.Linear.VerticalTurnSpeed" => {
                        let v = floats(&v, &p)?;
                        s.basic.linear.vertical_turn_speed = FloatRange(v[0], v[1]);
                    }
                    "AngleSmooth.Sigmoid.HorizontalTurnSpeed" => {
                        let v = floats(&v, &p)?;
                        s.basic.sigmoid.speeds.horizontal_turn_speed = FloatRange(v[0], v[1]);
                    }
                    "AngleSmooth.Sigmoid.VerticalTurnSpeed" => {
                        let v = floats(&v, &p)?;
                        s.basic.sigmoid.speeds.vertical_turn_speed = FloatRange(v[0], v[1]);
                    }
                    "AngleSmooth.Sigmoid.Steepness" => {
                        s.basic.sigmoid.steepness = parse_float(&v, &p)?
                    }
                    "AngleSmooth.Sigmoid.Midpoint" => {
                        s.basic.sigmoid.midpoint = parse_float(&v, &p)?
                    }
                    _ => return Err(ConfigError::new(&p, "unknown Aimbot setting")),
                }
            }
        }
        Ok(())
    }
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.aimbot.{}",
        path.split('.')
            .map(|p| if p == "FOV" {
                "fov".into()
            } else {
                p[..1].to_ascii_lowercase() + &p[1..]
            })
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn pair_f(r: [f32; 2]) -> Value {
    json!({"from":r[0],"to":r[1]})
}
fn pair_i(r: [i32; 2]) -> Value {
    json!({"from":r[0],"to":r[1]})
}
fn selected<const N: usize>(values: [bool; N], choices: [&str; N]) -> Value {
    json!(
        choices
            .into_iter()
            .enumerate()
            .filter(|(i, _)| values[*i])
            .map(|(_, s)| s)
            .collect::<Vec<_>>()
    )
}
pub fn serialize_aimbot(
    r: &AimbotRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = &r.behavior.settings;
    let a = &r.point_tracker.settings;
    let b = |n: &str, v: bool, p: &str| t.leaf(n, "BOOLEAN", json!(v), &key(p), None);
    let range = |n: &str, k: &str, v: Value, from: f64, to: f64, suffix: &str, p: &str| {
        t.ranged(n, k, v, json!({"from":from,"to":to}), suffix, &key(p))
    };
    let multi = |n: &str, v: Value, options: &[&str], p: &str| {
        let mut value = t.leaf(n, "MULTI_CHOOSE", v, &key(p), None);
        if f == JsonFormat::Interop {
            value["choices"] = json!(options);
            value["canBeNone"] = json!(true);
        }
        value
    };
    let mut values = t.common(ModuleKind::Aimbot, c);
    values.push(range("Range", "FLOAT", json!(s.range), 1., 8., "", "Range"));
    values.push(range(
        "WallRange",
        "FLOAT",
        json!(s.wall_range),
        0.,
        8.,
        "",
        "WallRange",
    ));
    let mut priority = multi(
        "Priority",
        json!(s.priority.iter().map(|p| p.name()).collect::<Vec<_>>()),
        &PRIORITIES.map(|p| p.name()),
        "Target.Priority",
    );
    if f == JsonFormat::Interop {
        priority["canBeNone"] = json!(false);
        priority["isOrderSensitive"] = json!(true);
    }
    values.push(t.group(
        "Target",
        "CONFIGURABLE",
        vec![
            range("FOV", "FLOAT", json!(s.fov), 0., 180., "", "Target.FOV"),
            range(
                "HurtTime",
                "INT",
                json!(s.hurt_time),
                0.,
                10.,
                "",
                "Target.HurtTime",
            ),
            priority,
        ],
    ));
    values.push(target_renderer::serialize_target_renderer(
        &r.target_renderer,
        f,
        d,
        "liquidbounce.module.aimbot.targetRendering",
    )?);
    let dynamic = t.group(
        "Dynamic",
        "TOGGLEABLE",
        vec![
            b("Enabled", a.dynamic, "AimPoint.Gaussian.Dynamic.Enabled"),
            range(
                "HurtTime",
                "INT",
                json!(a.dynamic_hurt),
                0.,
                10.,
                "",
                "AimPoint.Gaussian.Dynamic.HurtTime",
            ),
            range(
                "YawFactor",
                "FLOAT",
                json!(a.dynamic_yaw),
                0.,
                10.,
                "x",
                "AimPoint.Gaussian.Dynamic.YawFactor",
            ),
            range(
                "PitchFactor",
                "FLOAT",
                json!(a.dynamic_pitch),
                0.,
                10.,
                "x",
                "AimPoint.Gaussian.Dynamic.PitchFactor",
            ),
            range(
                "Speed",
                "FLOAT_RANGE",
                pair_f(a.dynamic_speed),
                0.01,
                1.,
                "",
                "AimPoint.Gaussian.Dynamic.Speed",
            ),
            range(
                "Tolerance",
                "FLOAT",
                json!(a.dynamic_tolerance),
                0.01,
                0.1,
                "",
                "AimPoint.Gaussian.Dynamic.Tolerance",
            ),
        ],
    );
    values.push(t.group(
        "AimPoint",
        "CONFIGURABLE",
        vec![
            multi(
                "ExemptBoxParts",
                selected(a.exempt_parts, PARTS),
                &PARTS,
                "AimPoint.ExemptBoxParts",
            ),
            t.group(
                "ExemptBestHitVector",
                "TOGGLEABLE",
                vec![
                    b(
                        "Enabled",
                        a.exempt_best,
                        "AimPoint.ExemptBestHitVector.Enabled",
                    ),
                    range(
                        "Vertical",
                        "FLOAT",
                        json!(a.vertical),
                        0.,
                        1.,
                        "",
                        "AimPoint.ExemptBestHitVector.Vertical",
                    ),
                    range(
                        "Horizontal",
                        "FLOAT",
                        json!(a.horizontal),
                        0.,
                        1.,
                        "",
                        "AimPoint.ExemptBestHitVector.Horizontal",
                    ),
                ],
            ),
            t.group(
                "Gaussian",
                "TOGGLEABLE",
                vec![
                    b("Enabled", a.gaussian, "AimPoint.Gaussian.Enabled"),
                    range(
                        "YawOffset",
                        "FLOAT_RANGE",
                        pair_f(a.yaw_offset),
                        0.,
                        1.,
                        "",
                        "AimPoint.Gaussian.YawOffset",
                    ),
                    range(
                        "PitchOffset",
                        "FLOAT_RANGE",
                        pair_f(a.pitch_offset),
                        0.,
                        1.,
                        "",
                        "AimPoint.Gaussian.PitchOffset",
                    ),
                    range(
                        "Chance",
                        "FLOAT",
                        json!(a.chance),
                        0.,
                        100.,
                        "%",
                        "AimPoint.Gaussian.Chance",
                    ),
                    range(
                        "Speed",
                        "FLOAT_RANGE",
                        pair_f(a.speed),
                        0.01,
                        1.,
                        "",
                        "AimPoint.Gaussian.Speed",
                    ),
                    range(
                        "Tolerance",
                        "FLOAT",
                        json!(a.tolerance),
                        0.01,
                        0.1,
                        "",
                        "AimPoint.Gaussian.Tolerance",
                    ),
                    dynamic,
                ],
            ),
            t.group(
                "Lazy",
                "TOGGLEABLE",
                vec![
                    b("Enabled", a.lazy, "AimPoint.Lazy.Enabled"),
                    range(
                        "Threshold",
                        "FLOAT_RANGE",
                        pair_f(a.threshold),
                        0.01,
                        0.4,
                        "m",
                        "AimPoint.Lazy.Threshold",
                    ),
                ],
            ),
            t.group(
                "Delay",
                "TOGGLEABLE",
                vec![
                    b("Enabled", a.delay, "AimPoint.Delay.Enabled"),
                    range(
                        "Delay",
                        "INT_RANGE",
                        pair_i(a.delay_ticks),
                        0.,
                        5.,
                        "ticks",
                        "AimPoint.Delay.Delay",
                    ),
                ],
            ),
        ],
    ));

    values.push(b("LazyRotation", s.lazy, "LazyRotation"));
    values.push(t.group(
        "Crystals",
        "TOGGLEABLE",
        vec![
            b("Enabled", s.crystals, "Crystals.Enabled"),
            b("Only", s.crystals_only, "Crystals.Only"),
            b(
                "OnlyWhileUsing",
                s.crystals_using,
                "Crystals.OnlyWhileUsing",
            ),
        ],
    ));
    values.push(multi(
        "Requires",
        selected(s.requires, REQUIRES),
        &REQUIRES,
        "Requires",
    ));
    let mut choices = Map::new();
    let i = &s.interpolation;
    choices.insert(
        "Interpolation".into(),
        t.group(
            "Interpolation",
            "CONFIGURABLE",
            vec![
                range(
                    "HorizontalSpeed",
                    "INT_RANGE",
                    pair_i(i.horizontal),
                    1.,
                    100.,
                    "%",
                    "AngleSmooth.Interpolation.HorizontalSpeed",
                ),
                range(
                    "VerticalSpeed",
                    "INT_RANGE",
                    pair_i(i.vertical),
                    1.,
                    100.,
                    "%",
                    "AngleSmooth.Interpolation.VerticalSpeed",
                ),
                range(
                    "DirectionChangeFactor",
                    "INT_RANGE",
                    pair_i(i.direction_change),
                    0.,
                    100.,
                    "%",
                    "AngleSmooth.Interpolation.DirectionChangeFactor",
                ),
                range(
                    "Midpoint",
                    "FLOAT",
                    json!(i.midpoint),
                    0.,
                    1.,
                    "",
                    "AngleSmooth.Interpolation.Midpoint",
                ),
            ],
        ),
    );
    for (name, settings) in [
        ("Sigmoid", s.basic.sigmoid.speeds),
        ("Linear", s.basic.linear),
    ] {
        let mut leaves = vec![];
        for (n, v) in [
            ("HorizontalTurnSpeed", settings.horizontal_turn_speed),
            ("VerticalTurnSpeed", settings.vertical_turn_speed),
        ] {
            leaves.push(range(
                n,
                "FLOAT_RANGE",
                pair_f([v.0, v.1]),
                0.,
                180.,
                "",
                &format!("AngleSmooth.{name}.{n}"),
            ));
        }
        if name == "Sigmoid" {
            leaves.push(range(
                "Steepness",
                "FLOAT",
                json!(s.basic.sigmoid.steepness),
                0.,
                20.,
                "",
                "AngleSmooth.Sigmoid.Steepness",
            ));
            leaves.push(range(
                "Midpoint",
                "FLOAT",
                json!(s.basic.sigmoid.midpoint),
                0.,
                1.,
                "",
                "AngleSmooth.Sigmoid.Midpoint",
            ));
        }
        choices.insert(name.into(), t.group(name, "CONFIGURABLE", leaves));
    }
    let mut smooth = t.group("AngleSmooth", "CHOICE", vec![]);
    smooth["active"] = json!(s.smooth.tag());
    smooth["choices"] = Value::Object(choices);
    values.push(smooth);
    values.push(multi("Axis", selected(s.axis, AXIS), &AXIS, "Axis"));
    values.push(multi(
        "Ignore",
        selected(s.ignore, IGNORES),
        &IGNORES,
        "Ignore",
    ));
    Ok(t.group("Aimbot", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, path: &str, v: &Value, point: bool) {
    let setting = if point {
        AimbotSetting::Point(path.into(), v.clone())
    } else {
        AimbotSetting::Value(path.into(), v.clone())
    };
    match setting.clone().apply(&mut AimbotRuntime::default()) {
        Ok(()) => p.changes.push(SettingChange::Aimbot(setting)),
        Err(e) => p.issues.push(e),
    }
}
fn point(p: &mut SettingsPatch, v: &Value) {
    fn walk(p: &mut SettingsPatch, v: &Value, prefix: &str) {
        let path = if prefix.is_empty() {
            "Aimbot.AimPoint".into()
        } else {
            format!("Aimbot.AimPoint.{prefix}")
        };
        let n = match nodes(
            v,
            if prefix.is_empty() {
                "AimPoint"
            } else {
                prefix.rsplit('.').next().unwrap()
            },
            &path,
        ) {
            Ok(n) => n,
            Err(e) => {
                p.issues.push(e);
                return;
            }
        };
        let leaves: &[&str] = match prefix {
            "" => &["ExemptBoxParts"],
            "ExemptBestHitVector" => &["Enabled", "Vertical", "Horizontal"],
            "Gaussian" => &[
                "Enabled",
                "YawOffset",
                "PitchOffset",
                "Chance",
                "Speed",
                "Tolerance",
            ],
            "Gaussian.Dynamic" => &[
                "Enabled",
                "HurtTime",
                "YawFactor",
                "PitchFactor",
                "Speed",
                "Tolerance",
            ],
            "Lazy" => &["Enabled", "Threshold"],
            "Delay" => &["Enabled", "Delay"],
            _ => &[],
        };
        for &name in leaves {
            if let Some(v) = n.get(name) {
                let full = if prefix.is_empty() {
                    name.to_string()
                } else {
                    format!("{prefix}.{name}")
                };
                match value_of(v, &format!("Aimbot.AimPoint.{full}")) {
                    Ok(v) => add(p, &full, v, true),
                    Err(e) => p.issues.push(e),
                }
            }
        }
        let children: &[&str] = match prefix {
            "" => &["ExemptBestHitVector", "Gaussian", "Lazy", "Delay"],
            "Gaussian" => &["Dynamic"],
            _ => &[],
        };
        for &child in children {
            if let Some(v) = n.get(child) {
                walk(
                    p,
                    v,
                    &if prefix.is_empty() {
                        child.to_string()
                    } else {
                        format!("{prefix}.{child}")
                    },
                );
            }
        }
    }
    walk(p, v, "");
}
pub(super) fn parse_aimbot(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in [
        "Range",
        "WallRange",
        "LazyRotation",
        "Requires",
        "Axis",
        "Ignore",
    ] {
        if let Some(v) = root.get(name) {
            match value_of(v, &format!("Aimbot.{name}")) {
                Ok(v) => add(p, name, v, false),
                Err(e) => p.issues.push(e),
            }
        }
    }
    for (group, leaves) in [
        ("Target", &["FOV", "HurtTime", "Priority"][..]),
        ("Crystals", &["Enabled", "Only", "OnlyWhileUsing"][..]),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("Aimbot.{group}")) {
                Ok(n) => {
                    for &name in leaves {
                        if let Some(v) = n.get(name) {
                            let path = format!("{group}.{name}");
                            match value_of(v, &format!("Aimbot.{path}")) {
                                Ok(v) => add(p, &path, v, false),
                                Err(e) => p.issues.push(e),
                            }
                        }
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
    if let Some(v) = root.get("AimPoint") {
        point(p, v);
    }
    if let Some(v) = root.get("TargetRendering") {
        let (settings, issues) =
            target_renderer::parse_target_renderer(v, "Aimbot.TargetRendering");
        p.changes.extend(
            settings
                .into_iter()
                .map(|v| SettingChange::Aimbot(AimbotSetting::Rendering(v))),
        );
        p.issues.extend(issues);
    }
    if let Some(v) = root.get("AngleSmooth") {
        if let Some(active) = v.get("active") {
            add(p, "AngleSmooth", active, false)
        } else {
            p.issues.push(ConfigError::new(
                "Aimbot.AngleSmooth.active",
                "missing active mode",
            ));
        }
        match nodes(v, "AngleSmooth", "Aimbot.AngleSmooth") {
            Ok(_) => {}
            Err(e) => {
                p.issues.push(e);
                return;
            }
        }
        let Some(choices) = v.get("choices").and_then(Value::as_object) else {
            p.issues.push(ConfigError::new(
                "Aimbot.AngleSmooth.choices",
                "expected choices object",
            ));
            return;
        };
        for (mode, leaves) in [
            (
                "Interpolation",
                &[
                    "HorizontalSpeed",
                    "VerticalSpeed",
                    "DirectionChangeFactor",
                    "Midpoint",
                ][..],
            ),
            (
                "Sigmoid",
                &[
                    "HorizontalTurnSpeed",
                    "VerticalTurnSpeed",
                    "Steepness",
                    "Midpoint",
                ][..],
            ),
            ("Linear", &["HorizontalTurnSpeed", "VerticalTurnSpeed"][..]),
        ] {
            let path = format!("Aimbot.AngleSmooth.{mode}");
            let Some(v) = choices.get(mode) else {
                p.issues.push(ConfigError::new(&path, "choice not found"));
                continue;
            };
            match nodes(v, mode, &path) {
                Ok(n) => {
                    for &name in leaves {
                        if let Some(v) = n.get(name) {
                            let full = format!("AngleSmooth.{mode}.{name}");
                            match value_of(v, &format!("Aimbot.{full}")) {
                                Ok(v) => add(p, &full, v, false),
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

#[cfg(test)]
mod aimbot_config_tests {
    use super::*;
    #[test]
    fn source_order_three_modes_and_inactive_roundtrip() {
        let mut r = AimbotRuntime::default();
        r.behavior.settings.smooth = Smooth::Sigmoid;
        r.behavior.settings.interpolation.horizontal = [63, 71];
        r.behavior.settings.basic.linear.vertical_turn_speed = FloatRange(13., 17.);
        r.behavior.settings.crystals = true;
        r.behavior.settings.crystals_only = true;
        r.behavior.settings.axis = [false, true];
        r.point_tracker.settings.delay = true;
        r.point_tracker.settings.delay_ticks = [1, 3];
        let value = serialize_aimbot(
            &r,
            &CommonSettings {
                enabled: false,
                hidden: false,
                key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Toggle"}),
            },
            JsonFormat::Interop,
            &str::to_owned,
        )
        .unwrap();
        let values = value["value"].as_array().unwrap();
        let names = values
            .iter()
            .filter_map(|v| v["name"].as_str())
            .collect::<Vec<_>>();
        for (before, after) in [
            ("Range", "WallRange"),
            ("Target", "TargetRendering"),
            ("TargetRendering", "AimPoint"),
            ("AimPoint", "LazyRotation"),
            ("Crystals", "Requires"),
            ("Requires", "AngleSmooth"),
            ("AngleSmooth", "Axis"),
            ("Axis", "Ignore"),
        ] {
            assert!(
                names.iter().position(|n| *n == before) < names.iter().position(|n| *n == after)
            );
        }
        let smooth = values.iter().find(|v| v["name"] == "AngleSmooth").unwrap();
        assert_eq!(smooth["active"], "Sigmoid");
        assert_eq!(smooth["choices"].as_object().unwrap().len(), 3);
        assert!(smooth["choices"].get("Acceleration").is_none());
        let patch = parse_settings(ModuleKind::Aimbot, &value).unwrap();
        assert!(patch.issues.is_empty(), "{:?}", patch.issues);
        let mut restored = AimbotRuntime::default();
        for change in patch.changes {
            if let SettingChange::Aimbot(change) = change {
                change.apply(&mut restored).unwrap();
            }
        }
        assert_eq!(r.behavior.settings, restored.behavior.settings);
        assert_eq!(r.point_tracker.settings, restored.point_tracker.settings);
        assert_eq!(ModuleKind::from_name("AimAssist"), Some(ModuleKind::Aimbot));
        assert_eq!(ModuleKind::from_name("AutoAim"), Some(ModuleKind::Aimbot));
    }
    #[test]
    fn wall_range_changes_only_on_its_own_assignment_and_bad_mode_keeps_other_leaves() {
        let mut r = AimbotRuntime::default();
        for (path, value) in [
            ("Range", json!(8.)),
            ("WallRange", json!(7.)),
            ("Range", json!(2.)),
        ] {
            AimbotSetting::Value(path.into(), value)
                .apply(&mut r)
                .unwrap();
        }
        assert_eq!(r.behavior.settings.wall_range, 7.);
        AimbotSetting::Value("WallRange".into(), json!(20.))
            .apply(&mut r)
            .unwrap();
        assert_eq!(r.behavior.settings.wall_range, 2.);
        let value = json!({"name":"Aimbot","value":[{"name":"Range","value":6.5},{"name":"AngleSmooth","active":"linear","value":[],"choices":{"Linear":{"name":"Linear","value":[{"name":"HorizontalTurnSpeed","value":{"from":2.,"to":3.}}]}}}]});
        let patch = parse_settings(ModuleKind::Aimbot, &value).unwrap();
        assert!(!patch.issues.is_empty());
        for change in patch.changes {
            if let SettingChange::Aimbot(change) = change {
                change.apply(&mut r).unwrap();
            }
        }
        assert_eq!(r.behavior.settings.range, 6.5);
        assert_eq!(r.behavior.settings.smooth, Smooth::Interpolation);
        assert_eq!(
            r.behavior.settings.basic.linear.horizontal_turn_speed,
            FloatRange(2., 3.)
        );
    }
}
