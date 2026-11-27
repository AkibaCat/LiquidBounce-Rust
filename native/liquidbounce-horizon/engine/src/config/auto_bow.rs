// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::auto_bow_runtime::AutoBowRuntime;
use liquidbounce_behaviors::{
    auto_bow::PRIORITIES,
    no_rotate_set::NoRotateSet,
    old_regen::{PACKET_TYPES, packet_type},
    target_renderer::TargetRendererSetting,
};
#[derive(Clone, Debug, PartialEq)]
pub enum AutoBowSetting {
    Value(String, Value),
    Rotation(NoRotateSetSetting),
    Rendering(TargetRendererSetting),
}
impl AutoBowSetting {
    pub fn apply(self, r: &mut AutoBowRuntime) -> Result<(), ConfigError> {
        match self {
            Self::Rotation(v) => {
                let mut p = NoRotateSet {
                    rotations: r.rotations.clone(),
                    ..Default::default()
                };
                v.apply(&mut p);
                r.rotations = p.rotations;
            }
            Self::Rendering(v) => v.apply(&mut r.target_renderer),
            Self::Value(path, v) => {
                let p = format!("AutoBow.{path}");
                let s = &mut r.behavior.settings;
                match path.as_str() {
                    "AutoShoot.Enabled" => {
                        s.auto_shoot = parse_bool(&v, &p)?;
                        if !s.auto_shoot {
                            r.behavior.force_uncharged = false;
                        }
                    }
                    "AutoShoot.Charged" => s.charged = parse_int(&v, &p)?,
                    "AutoShoot.ChargedRandom" => {
                        let range = parse_float_pair(&v, &p)?;
                        if range[0] > range[1] {
                            return Err(ConfigError::new(&p, "empty charged range"));
                        }
                        s.charged_random = range;
                    }
                    "AutoShoot.DelayBetweenShots" => s.delay = parse_float(&v, &p)?,
                    "AutoShoot.AimThreshold" => s.aim_threshold = parse_float(&v, &p)?,
                    "AutoShoot.RequiresHypotheticalHit" => s.requires_hit = parse_bool(&v, &p)?,
                    "AutoShoot.UsePrechargedCrossbow" => s.precharged = parse_bool(&v, &p)?,
                    "BowAimbot.Enabled" => s.aimbot = parse_bool(&v, &p)?,
                    "BowAimbot.ThroughWalls" => s.through_walls = parse_bool(&v, &p)?,
                    "BowAimbot.Target.FOV" => s.fov = parse_float(&v, &p)?,
                    "BowAimbot.Target.HurtTime" => s.hurt_time = parse_int(&v, &p)?,
                    "BowAimbot.Target.Priority" => {
                        let mut priorities = vec![];
                        let mut add = |v: &Value| -> Result<(), ConfigError> {
                            let name = scalar_string(v)
                                .ok_or_else(|| ConfigError::new(&p, "expected target priority"))?;
                            let choice = PRIORITIES
                                .into_iter()
                                .find(|q| q.name().eq_ignore_ascii_case(&name))
                                .ok_or_else(|| ConfigError::new(&p, "unknown target priority"))?;
                            if !priorities.contains(&choice) {
                                priorities.push(choice);
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
                    "FastCharge.Enabled" => s.fast_charge = parse_bool(&v, &p)?,
                    "FastCharge.Speed" => s.speed = parse_int(&v, &p)?,
                    "FastCharge.NotInTheAir" => s.ground_only = parse_bool(&v, &p)?,
                    "FastCharge.NotDuringMove" => s.stationary_only = parse_bool(&v, &p)?,
                    "FastCharge.NotDuringRegeneration" => s.no_regeneration = parse_bool(&v, &p)?,
                    "FastCharge.PacketType" => {
                        s.packet = packet_type(
                            &scalar_string(&v)
                                .ok_or_else(|| ConfigError::new(&p, "expected packet type"))?,
                        )
                        .ok_or_else(|| ConfigError::new(&p, "unknown packet type"))?
                    }
                    _ => return Err(ConfigError::new(&p, "unknown AutoBow setting")),
                }
            }
        }
        Ok(())
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.autoBow.bowAimbot.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "AutoBow.BowAimbot.Rotations",
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
pub fn serialize_auto_bow(
    r: &AutoBowRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = &r.behavior.settings;
    let mut values = t.common(ModuleKind::AutoBow, c);
    let key = |p: &str| format!("liquidbounce.module.autoBow.{p}");
    let boolean = |name: &str, value: bool, path: &str| {
        t.leaf(name, "BOOLEAN", json!(value), &key(path), None)
    };
    let range =
        |name: &str, kind: &str, value: Value, from: f64, to: f64, suffix: &str, path: &str| {
            t.ranged(
                name,
                kind,
                value,
                json!({"from":from,"to":to}),
                suffix,
                &key(path),
            )
        };
    values.push(t.group(
        "AutoShoot",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.auto_shoot, "autoShoot.enabled"),
            range(
                "Charged",
                "INT",
                json!(s.charged),
                3.,
                20.,
                "ticks",
                "autoShoot.charged",
            ),
            range(
                "ChargedRandom",
                "FLOAT_RANGE",
                json!({"from":s.charged_random[0],"to":s.charged_random[1]}),
                -10.,
                10.,
                "ticks",
                "autoShoot.chargedRandom",
            ),
            range(
                "DelayBetweenShots",
                "FLOAT",
                json!(s.delay),
                0.,
                5.,
                "s",
                "autoShoot.delayBetweenShots",
            ),
            range(
                "AimThreshold",
                "FLOAT",
                json!(s.aim_threshold),
                1.,
                4.,
                "°",
                "autoShoot.aimThreshold",
            ),
            boolean(
                "RequiresHypotheticalHit",
                s.requires_hit,
                "autoShoot.requiresHypotheticalHit",
            ),
            boolean(
                "UsePrechargedCrossbow",
                s.precharged,
                "autoShoot.usePrechargedCrossbow",
            ),
        ],
    ));
    let mut priority = t.leaf(
        "Priority",
        "MULTI_CHOOSE",
        json!(s.priority.iter().map(|p| p.name()).collect::<Vec<_>>()),
        &key("bowAimbot.target.priority"),
        None,
    );
    if f == JsonFormat::Interop {
        priority["choices"] = json!(PRIORITIES.map(|p| p.name()));
        priority["canBeNone"] = json!(false);
        priority["isOrderSensitive"] = json!(true);
    }
    let target = t.group(
        "Target",
        "CONFIGURABLE",
        vec![
            range(
                "FOV",
                "FLOAT",
                json!(s.fov),
                0.,
                180.,
                "",
                "bowAimbot.target.fov",
            ),
            range(
                "HurtTime",
                "INT",
                json!(s.hurt_time),
                0.,
                10.,
                "",
                "bowAimbot.target.hurtTime",
            ),
            priority,
        ],
    );
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
    let rendering = target_renderer::serialize_target_renderer(
        &r.target_renderer,
        f,
        d,
        &key("bowAimbot.targetRendering"),
    )?;
    values.push(t.group(
        "BowAimbot",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.aimbot, "bowAimbot.enabled"),
            boolean("ThroughWalls", s.through_walls, "bowAimbot.throughWalls"),
            target,
            rotations,
            rendering,
        ],
    ));
    let mut packet = t.leaf(
        "PacketType",
        "CHOOSE",
        json!(PACKET_TYPES[usize::from(s.packet)]),
        &key("fastCharge.packetType"),
        None,
    );
    if f == JsonFormat::Interop {
        packet["choices"] = json!(PACKET_TYPES);
    }
    values.push(t.group(
        "FastCharge",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.fast_charge, "fastCharge.enabled"),
            range(
                "Speed",
                "INT",
                json!(s.speed),
                3.,
                20.,
                "",
                "fastCharge.speed",
            ),
            boolean("NotInTheAir", s.ground_only, "fastCharge.notInTheAir"),
            boolean(
                "NotDuringMove",
                s.stationary_only,
                "fastCharge.notDuringMove",
            ),
            boolean(
                "NotDuringRegeneration",
                s.no_regeneration,
                "fastCharge.notDuringRegeneration",
            ),
            packet,
        ],
    ));
    Ok(t.group("AutoBow", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, path: &str, value: &Value) {
    let setting = AutoBowSetting::Value(path.into(), value.clone());
    match setting.clone().apply(&mut AutoBowRuntime::default()) {
        Ok(()) => p.changes.push(SettingChange::AutoBow(setting)),
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
                    .push(SettingChange::AutoBow(AutoBowSetting::Rotation(s)));
            }
        }
    }
    p.issues.extend(parsed.issues.into_iter().map(remap));
}
pub(super) fn parse_auto_bow(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for (group, names) in [
        (
            "AutoShoot",
            vec![
                "Enabled",
                "Charged",
                "ChargedRandom",
                "DelayBetweenShots",
                "AimThreshold",
                "RequiresHypotheticalHit",
                "UsePrechargedCrossbow",
            ],
        ),
        ("BowAimbot", vec!["Enabled", "ThroughWalls"]),
        (
            "FastCharge",
            vec![
                "Enabled",
                "Speed",
                "NotInTheAir",
                "NotDuringMove",
                "NotDuringRegeneration",
                "PacketType",
            ],
        ),
    ] {
        let Some(v) = root.get(group) else {
            continue;
        };
        let n = match nodes(v, group, &format!("AutoBow.{group}")) {
            Ok(n) => n,
            Err(e) => {
                p.issues.push(e);
                continue;
            }
        };
        for name in names {
            if let Some(v) = n.get(name) {
                let path = format!("AutoBow.{group}.{name}");
                match value_of(v, &path) {
                    Ok(v) => add(p, &format!("{group}.{name}"), v),
                    Err(e) => p.issues.push(e),
                }
            }
        }
        if group == "BowAimbot" {
            if let Some(v) = n.get("Rotations") {
                rotations(p, v);
            }
            if let Some(v) = n.get("Target") {
                match nodes(v, "Target", "AutoBow.BowAimbot.Target") {
                    Ok(n) => {
                        for name in ["FOV", "HurtTime", "Priority"] {
                            if let Some(v) = n.get(name) {
                                let path = format!("AutoBow.BowAimbot.Target.{name}");
                                match value_of(v, &path) {
                                    Ok(v) => add(p, &format!("BowAimbot.Target.{name}"), v),
                                    Err(e) => p.issues.push(e),
                                }
                            }
                        }
                    }
                    Err(e) => p.issues.push(e),
                }
            }
            if let Some(v) = n.get("TargetRendering") {
                let (settings, issues) =
                    target_renderer::parse_target_renderer(v, "AutoBow.BowAimbot.TargetRendering");
                p.changes.extend(
                    settings
                        .into_iter()
                        .map(|s| SettingChange::AutoBow(AutoBowSetting::Rendering(s))),
                );
                p.issues.extend(issues);
            }
        }
    }
}
