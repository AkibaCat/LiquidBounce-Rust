// SPDX-License-Identifier: GPL-3.0-or-later
// AimPoint tree and noncombat rotations derived verbatim from the pinned adapter.
use super::*;
use crate::{auto_rod_runtime::NativeSamples, auto_shoot_runtime::AutoShootRuntime};
use liquidbounce_behaviors::{
    auto_bow::PRIORITIES, clicking::Technique, no_rotate_set::NoRotateSet,
    target_renderer::TargetRendererSetting,
};
pub const PARTS: [&str; 3] = ["Head", "Body", "Feet"];
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.autoShoot.rotations";
#[derive(Clone, Debug, PartialEq)]
pub enum AutoShootSetting {
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
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "AutoShoot.Rotations",
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
        "liquidbounce.module.autoShoot.{}",
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
impl AutoShootSetting {
    pub fn apply(self, r: &mut AutoShootRuntime, items: &[String]) -> Result<(), ConfigError> {
        match self {
            Self::Rotation(v) => {
                let mut proxy = NoRotateSet {
                    rotations: std::mem::take(&mut r.rotations),
                    ..Default::default()
                };
                v.apply(&mut proxy);
                r.rotations = proxy.rotations;
            }
            Self::Rendering(v) => v.apply(&mut r.target_renderer),
            Self::Point(path, v) => {
                let p = format!("AutoShoot.AimPoint.{path}");
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
                let p = format!("AutoShoot.{path}");
                let s = &mut r.behavior.settings;
                match path.as_str() {
                    "ThrowableType" => {
                        s.custom = choice(&v, &["EggAndSnowball", "Custom", "Anything"], &p)? != 0
                    }
                    "ThrowableType.Custom.Filter" => {
                        s.whitelist = choice(&v, &["Whitelist", "Blacklist"], &p)? == 0
                    }
                    "ThrowableType.Custom.Items" => {
                        let values = if let Some(a) = v.as_array() {
                            a.clone()
                        } else {
                            vec![v.clone()]
                        };
                        let mut parsed = std::collections::BTreeSet::new();
                        for value in values {
                            let name = scalar_string(&value)
                                .ok_or_else(|| ConfigError::new(&p, "expected item identifier"))?;
                            let id = match name.find(':') {
                                None => format!("minecraft:{name}"),
                                Some(0) => format!("minecraft{name}"),
                                _ => name,
                            };
                            parsed.insert(if items.contains(&id) {
                                id
                            } else {
                                "minecraft:air".into()
                            });
                        }
                        s.items = parsed;
                    }
                    "GravityType" => {
                        s.gravity = choice(&v, &["Auto", "Linear", "Projectile"], &p)? as u8
                    }
                    "Clicker.Technique" => {
                        r.clicker.settings.technique =
                            if choice(&v, &["Human", "Constant"], &p)? == 0 {
                                Technique::Human
                            } else {
                                Technique::Constant
                            }
                    }
                    "Clicker.CPS" => r.clicker.settings.cps = ints(&v, &p)?,
                    "Clicker.MaxPerTick" => r.clicker.settings.max_per_tick = parse_int(&v, &p)?,
                    "Target.Range" => s.range = floats(&v, &p)?,
                    "Target.FOV" => s.fov = parse_float(&v, &p)?,
                    "Target.HurtTime" => s.hurt_time = parse_int(&v, &p)?,
                    "Target.Priority" => {
                        let values = if let Some(a) = v.as_array() {
                            a.clone()
                        } else {
                            vec![v.clone()]
                        };
                        let mut parsed = vec![];
                        for value in values {
                            let index = choice(&value, &PRIORITIES.map(|v| v.name()), &p)?;
                            if !parsed.contains(&PRIORITIES[index]) {
                                parsed.push(PRIORITIES[index]);
                            }
                        }
                        if parsed.is_empty() {
                            return Err(ConfigError::new(&p, "priority cannot be empty"));
                        }
                        s.priority = parsed;
                    }
                    "AimOffThreshold" => s.aim_threshold = parse_float(&v, &p)?,
                    "SwingMode" => {
                        s.hide_swing = choice(&v, &["DoNotHide", "HideForClient"], &p)? == 1
                    }
                    "SelectSlotAutomatically" => s.select_slot = parse_bool(&v, &p)?,
                    "TicksUntilSlotReset" => s.slot_reset = parse_int(&v, &p)?,
                    "ConsiderInventory" => s.consider_inventory = parse_bool(&v, &p)?,
                    "RequiresKillAura" => s.requires_aura = parse_bool(&v, &p)?,
                    "NotDuringCombat" => s.not_during_combat = parse_bool(&v, &p)?,
                    "ConstantLag" => s.constant_lag = parse_bool(&v, &p)?,
                    _ => return Err(ConfigError::new(&p, "unknown AutoShoot setting")),
                }
            }
        }
        Ok(())
    }
}
pub fn serialize_auto_shoot(
    r: &AutoShootRuntime,
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
    let choose = |n: &str, v: &str, options: &[&str], p: &str| {
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
    let mut values = t.common(ModuleKind::AutoShoot, c);
    let mut items = s.items.iter().collect::<Vec<_>>();
    items.sort_by(|a, b| {
        let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
        let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
        ap.cmp(bp).then(an.cmp(bn))
    });
    let custom = t.group(
        "Custom",
        "CONFIGURABLE",
        vec![
            choose(
                "Filter",
                if s.whitelist {
                    "Whitelist"
                } else {
                    "Blacklist"
                },
                &["Whitelist", "Blacklist"],
                "throwableType.custom.filter",
            ),
            t.leaf(
                "Items",
                "ITEM",
                json!(items),
                &key("throwableType.custom.items"),
                None,
            ),
        ],
    );
    let mut throwable = t.group("ThrowableType", "CHOICE", vec![]);
    throwable["active"] = json!(if s.custom { "Custom" } else { "EggAndSnowball" });
    throwable["choices"] =
        json!({"EggAndSnowball":t.group("EggAndSnowball","CONFIGURABLE",vec![]),"Custom":custom});
    values.push(throwable);
    values.push(choose(
        "GravityType",
        ["Auto", "Linear", "Projectile"][usize::from(s.gravity.min(2))],
        &["Auto", "Linear", "Projectile"],
        "GravityType",
    ));
    values.push(crosshair_clicker::click_tree(
        &t,
        r.clicker.settings,
        "liquidbounce.module.autoShoot.clicker",
    )?);
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
            range(
                "Range",
                "FLOAT_RANGE",
                pair_f(s.range),
                1.,
                256.,
                "m",
                "Target.Range",
            ),
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
        0.5,
        10.,
        "",
        "AimOffThreshold",
    ));
    values.push(choose(
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
        "liquidbounce.module.autoShoot.targetRendering",
    )?);
    values.push(b(
        "SelectSlotAutomatically",
        s.select_slot,
        "SelectSlotAutomatically",
    ));
    values.push(range(
        "TicksUntilSlotReset",
        "INT",
        json!(s.slot_reset),
        0.,
        20.,
        "ticks",
        "TicksUntilSlotReset",
    ));
    for (n, v) in [
        ("ConsiderInventory", s.consider_inventory),
        ("RequiresKillAura", s.requires_aura),
        ("NotDuringCombat", s.not_during_combat),
        ("ConstantLag", s.constant_lag),
    ] {
        values.push(b(n, v, n));
    }
    Ok(t.group("AutoShoot", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, path: &str, v: &Value, is_point: bool) {
    let s = if is_point {
        AutoShootSetting::Point(path.into(), v.clone())
    } else {
        AutoShootSetting::Value(path.into(), v.clone())
    };
    match s.clone().apply(&mut AutoShootRuntime::default(), &[]) {
        Ok(()) => p.changes.push(SettingChange::AutoShoot(s)),
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
                    .push(SettingChange::AutoShoot(AutoShootSetting::Rotation(s)));
            }
        }
    }
    p.issues.extend(parsed.issues.into_iter().map(remap));
}
fn point(p: &mut SettingsPatch, v: &Value) {
    fn walk(p: &mut SettingsPatch, v: &Value, prefix: &str) {
        let path = if prefix.is_empty() {
            "AutoShoot.AimPoint".into()
        } else {
            format!("AutoShoot.AimPoint.{prefix}")
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
                match value_of(v, &format!("AutoShoot.AimPoint.{full}")) {
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
pub(super) fn parse_auto_shoot(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for n in [
        "GravityType",
        "AimOffThreshold",
        "SwingMode",
        "SelectSlotAutomatically",
        "TicksUntilSlotReset",
        "ConsiderInventory",
        "RequiresKillAura",
        "NotDuringCombat",
        "ConstantLag",
    ] {
        leaf(p, root, n, &format!("AutoShoot.{n}"), |v, _| {
            Ok(SettingChange::AutoShoot(AutoShootSetting::Value(
                n.into(),
                v.clone(),
            )))
        });
    }
    if let Some(v) = root.get("ThrowableType") {
        if let Some(active) = v.get("active") {
            add(p, "ThrowableType", active, false);
        }
        if let Some(custom) = v["choices"]
            .get("Custom")
            .or_else(|| v["choices"].get("Anything"))
        {
            match nodes(
                custom,
                custom["name"].as_str().unwrap_or("Custom"),
                "AutoShoot.ThrowableType.Custom",
            ) {
                Ok(n) => {
                    for name in ["Filter", "Items"] {
                        if let Some(v) = n.get(name) {
                            match value_of(v, name) {
                                Ok(v) => add(p, &format!("ThrowableType.Custom.{name}"), v, false),
                                Err(e) => p.issues.push(e),
                            }
                        }
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
    for (group, names) in [
        ("Clicker", &["Technique", "CPS", "MaxPerTick"][..]),
        ("Target", &["Range", "FOV", "HurtTime", "Priority"][..]),
    ] {
        if let Some(v) = root.get(group).or_else(|| {
            (group == "Clicker")
                .then(|| root.get("ClickScheduler"))
                .flatten()
        }) {
            match nodes(
                v,
                v["name"].as_str().unwrap_or(group),
                &format!("AutoShoot.{group}"),
            ) {
                Ok(n) => {
                    for name in names {
                        if let Some(v) = n.get(*name) {
                            match value_of(v, name) {
                                Ok(v) => add(p, &format!("{group}.{name}"), v, false),
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
    if let Some(v) = root.get("Rotations") {
        rotations(p, v);
    }
    if let Some(v) = root.get("TargetRendering") {
        let (values, errors) =
            target_renderer::parse_target_renderer(v, "AutoShoot.TargetRendering");
        p.changes.extend(
            values
                .into_iter()
                .map(|v| SettingChange::AutoShoot(AutoShootSetting::Rendering(v))),
        );
        p.issues.extend(errors);
    }
}
