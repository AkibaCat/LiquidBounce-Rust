// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::auto_rod_runtime::{AutoRodRuntime, NativeSamples};
use liquidbounce_behaviors::{
    auto_bow::PRIORITIES, no_rotate_set::NoRotateSet, target_renderer::TargetRendererSetting,
};
pub const REQUIRES: [&str; 5] = [
    "Click",
    "MouseMovement",
    "Weapon",
    "VanillaName",
    "NotBreaking",
];
pub const IGNORES: [&str; 3] = ["OpenInventory", "UsingItem", "HoldingConsumable"];
pub const PARTS: [&str; 3] = ["Head", "Body", "Feet"];
#[derive(Clone, Debug, PartialEq)]
pub enum AutoRodSetting {
    Value(String, Value),
    Point(String, Value),
    Rotation(NoRotateSetSetting),
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
impl AutoRodSetting {
    pub fn apply(self, r: &mut AutoRodRuntime, items: &[String]) -> Result<(), ConfigError> {
        match self {
            Self::Rotation(v) => {
                let mut proxy = NoRotateSet {
                    rotations: r.rotations.clone(),
                    ..Default::default()
                };
                v.apply(&mut proxy);
                r.rotations = proxy.rotations;
            }
            Self::Rendering(v) => v.apply(&mut r.target_renderer),
            Self::Point(path, v) => {
                let p = format!("AutoRod.AimPoint.{path}");
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
                let p = format!("AutoRod.{path}");
                let s = &mut r.behavior.settings;
                match path.as_str() {
                    "GravityType" => s.projectile = choice(&v, &["Linear", "Projectile"], &p)? == 1,
                    "Range" => s.range = floats(&v, &p)?,
                    "ScanExtraRange" => {
                        let value = floats(&v, &p)?;
                        if value != s.scan_extra {
                            s.scan_extra = value;
                            r.behavior.scan_changed(&mut NativeSamples(&mut r.random));
                        }
                    }
                    "MaxEnemiesNearby" => s.max_enemies = parse_int(&v, &p)?,
                    "MinHealth" => s.min_health = parse_float(&v, &p)?,
                    "MinTargetHealth" => s.min_target_health = parse_float(&v, &p)?,
                    "Requires" => s.requires = multi(&v, REQUIRES, &p)?,
                    "Ignore" => s.ignore = multi(&v, IGNORES, &p)?,
                    "HoldingItemsForIgnore" => {
                        let values = match &v {
                            Value::Array(a) => a.clone(),
                            _ => vec![v],
                        };
                        let mut entries = vec![];
                        for v in values {
                            let raw = scalar_string(&v)
                                .ok_or_else(|| ConfigError::new(&p, "expected registry entry"))?;
                            let id = if raw.starts_with(':') {
                                format!("minecraft{raw}")
                            } else if raw.contains(':') {
                                raw
                            } else {
                                format!("minecraft:{raw}")
                            };
                            entries.push(if items.contains(&id) {
                                id
                            } else {
                                "minecraft:air".into()
                            });
                        }
                        entries.sort();
                        entries.dedup();
                        s.holding_ignore = entries;
                    }
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
                    "AimOffThreshold" => s.aim_threshold = parse_float(&v, &p)?,
                    "SwingMode" => {
                        s.hide_swing = choice(&v, &["DoNotHide", "HideForClient"], &p)? == 1
                    }
                    "HitTimeout" => s.hit_timeout = parse_int(&v, &p)?,
                    "PullOnOutOfRange" => s.pull_out_of_range = parse_bool(&v, &p)?,
                    "SlotResetDelay" => s.slot_reset = ints(&v, &p)?,
                    "Cooldown" => s.cooldown = ints(&v, &p)?,
                    _ => return Err(ConfigError::new(&p, "unknown AutoRod setting")),
                }
            }
        }
        Ok(())
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.autoRod.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "AutoRod.Rotations",
        )
        .replace(OLD, NEW);
    e
}
fn keys(v: &mut Value) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(OLD, NEW);
            }
            for v in o.values_mut() {
                keys(v);
            }
        }
        Value::Array(a) => {
            for v in a {
                keys(v)
            }
        }
        _ => {}
    }
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.autoRod.{}",
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
pub fn serialize_auto_rod(
    r: &AutoRodRuntime,
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
    let choice = |n: &str, v: &str, options: &[&str], p: &str| {
        let mut value = t.leaf(n, "CHOOSE", json!(v), &key(p), None);
        if f == JsonFormat::Interop {
            value["choices"] = json!(options);
        }
        value
    };
    let multi = |n: &str, v: Value, options: &[&str], p: &str| {
        let mut value = t.leaf(n, "MULTI_CHOOSE", v, &key(p), None);
        if f == JsonFormat::Interop {
            value["choices"] = json!(options);
            value["canBeNone"] = json!(true);
        }
        value
    };
    let mut values = t.common(ModuleKind::AutoRod, c);
    values.push(choice(
        "GravityType",
        if s.projectile { "Projectile" } else { "Linear" },
        &["Linear", "Projectile"],
        "GravityType",
    ));
    values.push(range(
        "Range",
        "FLOAT_RANGE",
        pair_f(s.range),
        2.,
        10.,
        "",
        "Range",
    ));
    values.push(range(
        "ScanExtraRange",
        "FLOAT_RANGE",
        pair_f(s.scan_extra),
        0.,
        5.,
        "",
        "ScanExtraRange",
    ));
    values.push(range(
        "MaxEnemiesNearby",
        "INT",
        json!(s.max_enemies),
        0.,
        10.,
        "",
        "MaxEnemiesNearby",
    ));
    values.push(range(
        "MinHealth",
        "FLOAT",
        json!(s.min_health),
        1.,
        20.,
        "",
        "MinHealth",
    ));
    values.push(range(
        "MinTargetHealth",
        "FLOAT",
        json!(s.min_target_health),
        1.,
        20.,
        "",
        "MinTargetHealth",
    ));
    values.push(multi(
        "Requires",
        selected(s.requires, REQUIRES),
        &REQUIRES,
        "Requires",
    ));
    values.push(multi(
        "Ignore",
        selected(s.ignore, IGNORES),
        &IGNORES,
        "Ignore",
    ));
    let mut items = t.leaf(
        "HoldingItemsForIgnore",
        "REGISTRY_LIST",
        json!(s.holding_ignore),
        &key("HoldingItemsForIgnore"),
        None,
    );
    if f == JsonFormat::Interop {
        items["registry"] = json!("item");
    }
    values.push(items);
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
    let proxy = NoRotateSet {
        rotations: r.rotations.clone(),
        ..Default::default()
    };
    let tree =
        serialize_no_rotate_set(&proxy, c, f, &|k| d(&k.replace(OLD, NEW))).map_err(remap)?;
    let mut rotations = tree["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Mode")
        .unwrap()["choices"]["ResetRotation"]["value"][0]
        .clone();
    keys(&mut rotations);
    values.push(rotations);
    values.push(range(
        "AimOffThreshold",
        "FLOAT",
        json!(s.aim_threshold),
        2.,
        10.,
        "",
        "AimOffThreshold",
    ));
    values.push(choice(
        "SwingMode",
        if s.hide_swing {
            "HideForClient"
        } else {
            "DoNotHide"
        },
        &["DoNotHide", "HideForClient"],
        "SwingMode",
    ));
    values.push(target_renderer::serialize_target_renderer(
        &r.target_renderer,
        f,
        d,
        "liquidbounce.module.autoRod.targetRendering",
    )?);
    values.push(range(
        "HitTimeout",
        "INT",
        json!(s.hit_timeout),
        5.,
        200.,
        "ticks",
        "HitTimeout",
    ));
    values.push(b(
        "PullOnOutOfRange",
        s.pull_out_of_range,
        "PullOnOutOfRange",
    ));
    values.push(range(
        "SlotResetDelay",
        "INT_RANGE",
        pair_i(s.slot_reset),
        0.,
        20.,
        "ticks",
        "SlotResetDelay",
    ));
    values.push(range(
        "Cooldown",
        "INT_RANGE",
        pair_i(s.cooldown),
        1.,
        50.,
        "ticks",
        "Cooldown",
    ));
    Ok(t.group("AutoRod", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, path: &str, v: &Value, point: bool) {
    let setting = if point {
        AutoRodSetting::Point(path.into(), v.clone())
    } else {
        AutoRodSetting::Value(path.into(), v.clone())
    };
    match setting.clone().apply(&mut AutoRodRuntime::default(), &[]) {
        Ok(()) => p.changes.push(SettingChange::AutoRod(setting)),
        Err(e) => p.issues.push(e),
    }
}
fn rotations(p: &mut SettingsPatch, v: &Value) {
    let tree = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[v]}}}]});
    let root = nodes(&tree, "NoRotateSet", "NoRotateSet").unwrap();
    let mut parsed = SettingsPatch {
        module: ModuleKind::NoRotateSet,
        changes: vec![],
        issues: vec![],
    };
    no_rotate_set::parse_no_rotate_set(&mut parsed, &root);
    for c in parsed.changes {
        if let SettingChange::NoRotateSet(s) = c {
            if !matches!(s, NoRotateSetSetting::Mode(_)) {
                p.changes
                    .push(SettingChange::AutoRod(AutoRodSetting::Rotation(s)));
            }
        }
    }
    p.issues.extend(parsed.issues.into_iter().map(remap));
}
fn point(p: &mut SettingsPatch, v: &Value) {
    fn walk(p: &mut SettingsPatch, v: &Value, prefix: &str) {
        let path = if prefix.is_empty() {
            "AutoRod.AimPoint".into()
        } else {
            format!("AutoRod.AimPoint.{prefix}")
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
                match value_of(v, &format!("AutoRod.AimPoint.{full}")) {
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
pub(super) fn parse_auto_rod(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in [
        "GravityType",
        "Range",
        "ScanExtraRange",
        "MaxEnemiesNearby",
        "MinHealth",
        "MinTargetHealth",
        "Requires",
        "Ignore",
        "HoldingItemsForIgnore",
        "AimOffThreshold",
        "SwingMode",
        "HitTimeout",
        "PullOnOutOfRange",
        "SlotResetDelay",
        "Cooldown",
    ] {
        if let Some(v) = root.get(name) {
            match value_of(v, &format!("AutoRod.{name}")) {
                Ok(v) => add(p, name, v, false),
                Err(e) => p.issues.push(e),
            }
        }
    }
    if let Some(v) = root.get("Target") {
        match nodes(v, "Target", "AutoRod.Target") {
            Ok(n) => {
                for name in ["FOV", "HurtTime", "Priority"] {
                    if let Some(v) = n.get(name) {
                        match value_of(v, &format!("AutoRod.Target.{name}")) {
                            Ok(v) => add(p, &format!("Target.{name}"), v, false),
                            Err(e) => p.issues.push(e),
                        }
                    }
                }
            }
            Err(e) => p.issues.push(e),
        }
    }
    if let Some(v) = root.get("AimPoint") {
        point(p, v);
    }
    if let Some(v) = root.get("Rotations") {
        rotations(p, v);
    }
    if let Some(v) = root.get("TargetRendering") {
        let (settings, issues) =
            target_renderer::parse_target_renderer(v, "AutoRod.TargetRendering");
        p.changes.extend(
            settings
                .into_iter()
                .map(|s| SettingChange::AutoRod(AutoRodSetting::Rendering(s))),
        );
        p.issues.extend(issues);
    }
}
