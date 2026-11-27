// SPDX-License-Identifier: GPL-3.0-or-later
use super::combat_rotations::CombatRotationSetting;
use super::*;
use crate::kill_aura_runtime::KillAuraRuntime;
use liquidbounce_behaviors::{auto_bow::PRIORITIES, clicking::Technique};
const REQUIRES: [&str; 6] = [
    "Click",
    "MouseMovement",
    "Weapon",
    "EmptyHand",
    "VanillaName",
    "NotBreaking",
];
const RAYCAST: [&str; 3] = ["None", "Enemy", "All"];
const CRITICALS: [&str; 3] = ["Smart", "Ignore", "Always"];
const TIMING: [&str; 3] = ["Normal", "Snap", "OnTick"];
const BLOCK: [&str; 3] = ["Basic", "Interact", "Fake"];
const UNBLOCK: [&str; 4] = ["StopUsingItem", "ChangeSlot", "SwapHand", "None"];
const NOTIFY: [&str; 3] = ["None", "Box", "Sound"];
const COLOR: [&str; 3] = ["Static", "Rainbow", "Distance"];
#[derive(Clone, Debug, PartialEq)]
pub enum KillAuraSetting {
    Value(String, Value),
    Point(String, Value),
    Rotation(CombatRotationSetting),
    Rendering(liquidbounce_behaviors::target_renderer::TargetRendererSetting),
    Navigation(AutoWalkSetting),
}
fn choose(v: &Value, tags: &[&str], p: &str) -> Result<u8, ConfigError> {
    scalar_string(v)
        .and_then(|s| tags.iter().position(|t| t.eq_ignore_ascii_case(&s)))
        .map(|v| v as u8)
        .ok_or_else(|| ConfigError::new(p, "unknown choice"))
}
fn floats(v: &Value, p: &str) -> Result<[f32; 2], ConfigError> {
    let a = parse_float_pair(v, p)?;
    if a[0] > a[1] {
        Err(ConfigError::new(p, "empty float range"))
    } else {
        Ok(a)
    }
}
fn ints(v: &Value, p: &str) -> Result<[i32; 2], ConfigError> {
    let a = parse_int_pair(v, p)?;
    if a[0] > a[1] {
        Err(ConfigError::new(p, "empty integer range"))
    } else {
        Ok(a)
    }
}
fn color(v: &Value, p: &str) -> Result<u32, ConfigError> {
    if let Some(i) = v.as_i64() {
        return Ok(i as u32);
    }
    if let Some(i) = v.as_u64() {
        return Ok(i as u32);
    }
    let s = v
        .as_str()
        .and_then(|s| s.strip_prefix('#').or(Some(s)))
        .ok_or_else(|| ConfigError::new(p, "expected color"))?;
    u32::from_str_radix(s, 16)
        .map(|c| if s.len() == 6 { c | 0xff000000 } else { c })
        .map_err(|_| ConfigError::new(p, "expected hexadecimal color"))
}
impl KillAuraSetting {
    pub fn apply(self, r: &mut KillAuraRuntime) -> Result<(), ConfigError> {
        match self {
            Self::Rotation(v) => return v.apply(&mut r.rotations),
            Self::Rendering(v) => {
                v.apply(&mut r.target_renderer);
                return Ok(());
            }
            Self::Navigation(v) => {
                v.apply(&mut r.navigation);
                return Ok(());
            }
            Self::Point(path, v) => {
                let mut proxy = crate::aimbot_runtime::AimbotRuntime::default();
                proxy.point_tracker = r.point_tracker.clone();
                proxy.random = r.random.clone();
                super::aimbot::AimbotSetting::Point(path, v)
                    .apply(&mut proxy)
                    .map_err(|mut e| {
                        e.path = e.path.replace("Aimbot", "KillAura");
                        e
                    })?;
                r.point_tracker = proxy.point_tracker;
                r.random = proxy.random;
                return Ok(());
            }
            Self::Value(path, v) => {
                let p = format!("KillAura.{path}");
                let s = &mut r.behavior.settings;
                match path.as_str() {
                    "Clicker.Technique" => {
                        r.clicker.settings.technique =
                            if choose(&v, &["Human", "Constant"], &p)? == 0 {
                                Technique::Human
                            } else {
                                Technique::Constant
                            }
                    }
                    "Clicker.CPS" => r.clicker.settings.cps = ints(&v, &p)?,
                    "Clicker.MaxPerTick" => r.clicker.settings.max_per_tick = parse_int(&v, &p)?,
                    "Clicker.MissCooldown" => {
                        r.clicker.settings.miss_cooldown = parse_bool(&v, &p)?
                    }
                    "Clicker.ItemCooldown.Minimum" => {
                        r.clicker.settings.item_cooldown = Some(floats(&v, &p)?)
                    }
                    "Clicker.ItemCooldown.IgnoreOnShieldBreak" => {
                        s.cooldown_ignore[0] = parse_bool(&v, &p)?
                    }
                    "Clicker.ItemCooldown.IgnoreOnMaceSmash" => {
                        s.cooldown_ignore[1] = parse_bool(&v, &p)?
                    }
                    "Clicker.ItemCooldown.IgnoreWhenExitingRange" => {
                        s.cooldown_ignore[2] = parse_bool(&v, &p)?
                    }
                    "Range.RangeIncrease" => s.increase = parse_float(&v, &p)?,
                    "Range.ThroughWallsRange" => {
                        let x = parse_float(&v, &p)?;
                        let a =
                            r.facts["attributeRange"].as_f64().unwrap_or(3.) as f32 + s.increase;
                        s.walls = if x.is_nan() || a.is_nan() {
                            f32::NAN
                        } else {
                            x.min(a)
                        };
                    }
                    "Range.ScanRangeIncrease" => {
                        let x = floats(&v, &p)?;
                        if x != s.scan {
                            s.scan = x;
                            r.behavior.scan_addition =
                                crate::auto_rod_runtime::NativeSamples(&mut r.random).float(x);
                        }
                    }
                    "Target.FOV" => s.fov = parse_float(&v, &p)?,
                    "Target.HurtTime" => s.hurt = parse_int(&v, &p)?,
                    "Target.IgnoreShield" => s.ignore_shield = parse_bool(&v, &p)?,
                    "Target.Priority" => {
                        let values = v.as_array().cloned().unwrap_or_else(|| vec![v.clone()]);
                        let mut out = vec![];
                        for v in values {
                            let i = choose(&v, &PRIORITIES.map(|p| p.name()), &p)? as usize;
                            if !out.contains(&PRIORITIES[i]) {
                                out.push(PRIORITIES[i]);
                            }
                        }
                        if out.is_empty() {
                            return Err(ConfigError::new(&p, "priority cannot be empty"));
                        }
                        s.priority = out;
                    }
                    "Requires" => {
                        s.requirements = [false; 6];
                        for v in v
                            .as_array()
                            .cloned()
                            .unwrap_or_else(|| if v.is_null() { vec![] } else { vec![v.clone()] })
                        {
                            s.requirements[choose(&v, &REQUIRES, &p)? as usize] = true;
                        }
                    }
                    "Raycast" => s.raycast = choose(&v, &RAYCAST, &p)?,
                    "Criticals" => s.criticals = choose(&v, &CRITICALS, &p)?,
                    "KeepSprint" => s.keep_sprint = parse_bool(&v, &p)?,
                    "IgnoreOpenInventory" => s.ignore_inventory = parse_bool(&v, &p)?,
                    "SimulateInventoryClosing" => s.close_inventory = parse_bool(&v, &p)?,
                    "Rotations.RotationTiming" => s.timing = choose(&v, &TIMING, &p)?,
                    "Rotations.ThroughWalls" => s.through = parse_bool(&v, &p)?,
                    "Rotations.LazyRotation" => s.lazy = parse_bool(&v, &p)?,
                    "AutoBlocking.Enabled" => s.block.enabled = parse_bool(&v, &p)?,
                    "AutoBlocking.BlockMode" => s.block.mode = choose(&v, &BLOCK, &p)?,
                    "AutoBlocking.SimulateVanillaUse" => s.block.vanilla = parse_bool(&v, &p)?,
                    "AutoBlocking.UnblockMode" => s.block.unblock = choose(&v, &UNBLOCK, &p)?,
                    "AutoBlocking.Reblock" | "AutoBlocking.TickOn" => {
                        s.block.reblock = ints(&v, &p)?;
                        r.blocking.state.lock().expect("AutoBlocking").reblock =
                            liquidbounce_behaviors::random::random_i32_inclusive(
                                s.block.reblock[0],
                                s.block.reblock[1],
                            );
                    }
                    "AutoBlocking.PauseOnUnblock" | "AutoBlocking.TickOff" => {
                        s.block.pause = ints(&v, &p)?;
                        r.blocking.state.lock().expect("AutoBlocking").pause =
                            liquidbounce_behaviors::random::random_i32_inclusive(
                                s.block.pause[0],
                                s.block.pause[1],
                            );
                    }
                    "AutoBlocking.Chance" => s.block.chance = parse_float(&v, &p)?,
                    "AutoBlocking.Blink" => s.block.blink = parse_int(&v, &p)?,
                    "AutoBlocking.PrioritizeBlocking" => s.block.prioritize = parse_bool(&v, &p)?,
                    "AutoBlocking.OnScanRange" => s.block.scan = parse_bool(&v, &p)?,
                    "AutoBlocking.AssumeShield" => s.block.assume = parse_bool(&v, &p)?,
                    "AutoBlocking.OnlyWhenInDanger.Enabled" => s.block.danger = parse_bool(&v, &p)?,
                    "AutoBlocking.OnlyWhenInDanger.Tolerance" => {
                        s.block.tolerance = parse_float(&v, &p)?
                    }
                    "AutoBlocking.OnlyWhenInDanger.ForceActiveRange" => {
                        s.block.force = floats(&v, &p)?
                    }
                    "FailSwing.Enabled" => s.fail.enabled = parse_bool(&v, &p)?,
                    "FailSwing.AdditionalRange" => {
                        let a = floats(&v, &p)?;
                        if a != s.fail.additional {
                            s.fail.additional = a;
                            r.behavior.fail_addition =
                                crate::auto_rod_runtime::NativeSamples(&mut r.random).float(a);
                        }
                    }
                    "FailSwing.NotifyWhenFail" => s.fail.mode = choose(&v, &NOTIFY, &p)?,
                    "FailSwing.NotifyWhenFail.Box.Fade" => s.fail.fade = parse_int(&v, &p)?,
                    "FailSwing.NotifyWhenFail.Box.Color" => s.fail.color = color(&v, &p)?,
                    "FailSwing.NotifyWhenFail.Box.Rainbow" => s.fail.rainbow = parse_bool(&v, &p)?,
                    "FailSwing.NotifyWhenFail.Sound.Volume" => s.fail.volume = parse_float(&v, &p)?,
                    "FailSwing.NotifyWhenFail.Sound.Pitch" => s.fail.pitch = parse_float(&v, &p)?,
                    "FightBot.Enabled" => s.fight.enabled = parse_bool(&v, &p)?,
                    "FightBot.OpponentRange" => s.fight.opponent = parse_float(&v, &p)?,
                    "FightBot.DangerousYaw" => s.fight.yaw = parse_float(&v, &p)?,
                    "FightBot.RunawayOnCooldown" => s.fight.runaway = parse_bool(&v, &p)?,
                    "FightBot.TargetFilter.Range" => s.fight.range = parse_float(&v, &p)?,
                    "FightBot.TargetFilter.VisibleOnly" => s.fight.visible = parse_bool(&v, &p)?,
                    "FightBot.TargetFilter.NotWhenVoid" => s.fight.void = parse_bool(&v, &p)?,
                    "FightBot.Leader.Enabled" => s.fight.leader = parse_bool(&v, &p)?,
                    "FightBot.Leader.Username" => {
                        s.fight.username = scalar_string(&v)
                            .ok_or_else(|| ConfigError::new(&p, "expected username"))?
                    }
                    "FightBot.Leader.Radius" => s.fight.radius = parse_float(&v, &p)?,
                    "RangeIndicator.Enabled" => s.indicator.enabled = parse_bool(&v, &p)?,
                    "RangeIndicator.ColorMode" => s.indicator.mode = choose(&v, &COLOR, &p)?,
                    "RangeIndicator.IdleColor" => s.indicator.idle = color(&v, &p)?,
                    "RangeIndicator.ActiveColor" => s.indicator.active = color(&v, &p)?,
                    "RangeIndicator.Outline" => s.indicator.outline = parse_bool(&v, &p)?,
                    "RangeIndicator.OutlineColor" => s.indicator.outline_color = color(&v, &p)?,
                    "RangeIndicator.PulseAnimation" => s.indicator.pulse = parse_bool(&v, &p)?,
                    "RangeIndicator.PulseSpeed" => s.indicator.pulse_speed = parse_float(&v, &p)?,
                    "RangeIndicator.PulseIntensity" => {
                        s.indicator.pulse_intensity = parse_float(&v, &p)?
                    }
                    "RangeIndicator.FadeAnimation" => s.indicator.fade = parse_bool(&v, &p)?,
                    "RangeIndicator.FadeSpeed" => s.indicator.fade_speed = parse_float(&v, &p)?,
                    "RangeIndicator.WallRangeColor" => s.indicator.walls = color(&v, &p)?,
                    "RangeIndicator.ScanRangeColor" => s.indicator.scan = color(&v, &p)?,
                    "RangeIndicator.OpponentRangeColor" => s.indicator.opponent = color(&v, &p)?,
                    "RangeIndicator.HideWhenDead" => s.indicator.hide_dead = parse_bool(&v, &p)?,
                    "RangeIndicator.HideWhenSpectator" => {
                        s.indicator.hide_spectator = parse_bool(&v, &p)?
                    }
                    "RangeIndicator.HideInVehicle" => {
                        s.indicator.hide_vehicle = parse_bool(&v, &p)?
                    }
                    "RangeIndicator.RespectInventorySetting" => {
                        s.indicator.inventory = parse_bool(&v, &p)?
                    }
                    "RangeIndicator.CanBeCovered" => s.indicator.covered = parse_bool(&v, &p)?,
                    _ => return Err(ConfigError::new(&p, "unknown KillAura setting")),
                }
            }
        }
        Ok(())
    }
}
use liquidbounce_behaviors::point_tracker::Samples;
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.killAura.{}",
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
fn remap(v: &mut Value, from: &str, to: &str) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(from, to);
            }
            for v in o.values_mut() {
                remap(v, from, to);
            }
        }
        Value::Array(a) => {
            for v in a {
                remap(v, from, to);
            }
        }
        _ => {}
    }
}
fn subtree(mut tree: Value, name: &str) -> Result<Value, ConfigError> {
    tree["value"]
        .as_array_mut()
        .and_then(|a| a.iter_mut().find(|v| v["name"] == name))
        .map(Value::take)
        .ok_or_else(|| ConfigError::new(name, "shared codec shape changed"))
}
pub fn serialize_kill_aura(
    r: &KillAuraRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = &r.behavior.settings;
    for (name, value) in [
        ("Range.RangeIncrease", s.increase),
        ("Range.ThroughWallsRange", s.walls),
        ("Target.FOV", s.fov),
        ("AutoBlocking.Chance", s.block.chance),
        ("AutoBlocking.OnlyWhenInDanger.Tolerance", s.block.tolerance),
        ("FightBot.OpponentRange", s.fight.opponent),
        ("FightBot.DangerousYaw", s.fight.yaw),
        ("FightBot.TargetFilter.Range", s.fight.range),
        ("FightBot.Leader.Radius", s.fight.radius),
        ("FailSwing.NotifyWhenFail.Sound.Volume", s.fail.volume),
        ("FailSwing.NotifyWhenFail.Sound.Pitch", s.fail.pitch),
        ("RangeIndicator.PulseSpeed", s.indicator.pulse_speed),
        ("RangeIndicator.PulseIntensity", s.indicator.pulse_intensity),
        ("RangeIndicator.FadeSpeed", s.indicator.fade_speed),
    ] {
        float_json(value, &format!("KillAura.{name}"))?;
    }
    for (name, pair) in [
        (
            "Clicker.ItemCooldown.Minimum",
            r.clicker.settings.item_cooldown.unwrap_or([1., 1.]),
        ),
        ("Range.ScanRangeIncrease", s.scan),
        (
            "AutoBlocking.OnlyWhenInDanger.ForceActiveRange",
            s.block.force,
        ),
        ("FailSwing.AdditionalRange", s.fail.additional),
    ] {
        for value in pair {
            float_json(value, &format!("KillAura.{name}"))?;
        }
    }
    let leaf = |name: &str, kind: &str, value: Value, path: &str| {
        t.leaf(name, kind, value, &key(path), None)
    };
    let b = |name: &str, value: bool, path: &str| leaf(name, "BOOLEAN", json!(value), path);
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
    let float_pair = |v: [f32; 2]| json!({"from":v[0],"to":v[1]});
    let int_pair = |v: [i32; 2]| json!({"from":v[0],"to":v[1]});
    let choice = |name: &str, value: u8, tags: &[&str], path: &str| {
        let mut v = leaf(name, "CHOOSE", json!(tags[value as usize]), path);
        if f == JsonFormat::Interop {
            v["choices"] = json!(tags);
        }
        v
    };
    let mut values = t.common(ModuleKind::KillAura, c);
    let click = r.clicker.settings;
    values.push(t.group(
        "Clicker",
        "CONFIGURABLE",
        vec![
            choice(
                "Technique",
                u8::from(click.technique != Technique::Human),
                &["Human", "Constant"],
                "Clicker.Technique",
            ),
            range(
                "CPS",
                "INT_RANGE",
                int_pair(click.cps),
                1.,
                30.,
                "clicks",
                "Clicker.CPS",
            ),
            range(
                "MaxPerTick",
                "INT",
                json!(click.max_per_tick),
                1.,
                5.,
                "clicks",
                "Clicker.MaxPerTick",
            ),
            t.group(
                "ItemCooldown",
                "CONFIGURABLE",
                vec![
                    range(
                        "Minimum",
                        "FLOAT_RANGE",
                        float_pair(click.item_cooldown.unwrap_or([1., 1.])),
                        0.,
                        2.,
                        "",
                        "Clicker.ItemCooldown.Minimum",
                    ),
                    b(
                        "IgnoreOnShieldBreak",
                        s.cooldown_ignore[0],
                        "Clicker.ItemCooldown.IgnoreOnShieldBreak",
                    ),
                    b(
                        "IgnoreOnMaceSmash",
                        s.cooldown_ignore[1],
                        "Clicker.ItemCooldown.IgnoreOnMaceSmash",
                    ),
                    b(
                        "IgnoreWhenExitingRange",
                        s.cooldown_ignore[2],
                        "Clicker.ItemCooldown.IgnoreWhenExitingRange",
                    ),
                ],
            ),
            b("MissCooldown", click.miss_cooldown, "Clicker.MissCooldown"),
        ],
    ));
    values.push(t.group(
        "Range",
        "CONFIGURABLE",
        vec![
            range(
                "RangeIncrease",
                "FLOAT",
                json!(s.increase),
                0.,
                5.,
                "blocks",
                "Range.RangeIncrease",
            ),
            range(
                "ThroughWallsRange",
                "FLOAT",
                json!(s.walls),
                0.,
                8.,
                "blocks",
                "Range.ThroughWallsRange",
            ),
            range(
                "ScanRangeIncrease",
                "FLOAT_RANGE",
                float_pair(s.scan),
                0.,
                7.,
                "blocks",
                "Range.ScanRangeIncrease",
            ),
        ],
    ));
    let mut priority = leaf(
        "Priority",
        "MULTI_CHOOSE",
        json!(s.priority.iter().map(|v| v.name()).collect::<Vec<_>>()),
        "Target.Priority",
    );
    if f == JsonFormat::Interop {
        priority["choices"] = json!(PRIORITIES.map(|p| p.name()));
        priority["canBeNone"] = json!(false);
        priority["isOrderSensitive"] = json!(true);
    }
    values.push(t.group(
        "Target",
        "CONFIGURABLE",
        vec![
            range("FOV", "FLOAT", json!(s.fov), 0., 180., "°", "Target.FOV"),
            range(
                "HurtTime",
                "INT",
                json!(s.hurt),
                0.,
                10.,
                "",
                "Target.HurtTime",
            ),
            priority,
            b("IgnoreShield", s.ignore_shield, "Target.IgnoreShield"),
        ],
    ));
    let mut rotations = combat_rotations::serialize(
        &r.rotations,
        c,
        f,
        d,
        "liquidbounce.module.killAura.rotations",
    )?;
    rotations["value"]
        .as_array_mut()
        .expect("Rotations values")
        .extend([
            choice(
                "RotationTiming",
                s.timing,
                &TIMING,
                "Rotations.RotationTiming",
            ),
            b("ThroughWalls", s.through, "Rotations.ThroughWalls"),
            b("LazyRotation", s.lazy, "Rotations.LazyRotation"),
        ]);
    values.push(rotations);
    let mut proxy = crate::aimbot_runtime::AimbotRuntime::default();
    proxy.point_tracker = r.point_tracker.clone();
    let mut point = subtree(
        super::aimbot::serialize_aimbot(&proxy, c, f, &|k| {
            d(&k.replace("liquidbounce.module.aimbot", "liquidbounce.module.killAura"))
        })?,
        "AimPoint",
    )?;
    remap(
        &mut point,
        "liquidbounce.module.aimbot",
        "liquidbounce.module.killAura",
    );
    values.push(point);
    let mut requires = leaf(
        "Requires",
        "MULTI_CHOOSE",
        json!(
            REQUIRES
                .iter()
                .enumerate()
                .filter(|(i, _)| s.requirements[*i])
                .map(|(_, v)| v)
                .collect::<Vec<_>>()
        ),
        "Requires",
    );
    if f == JsonFormat::Interop {
        requires["choices"] = json!(REQUIRES);
        requires["canBeNone"] = json!(true);
    }
    values.push(requires);
    values.extend([
        choice("Raycast", s.raycast, &RAYCAST, "Raycast"),
        choice("Criticals", s.criticals, &CRITICALS, "Criticals"),
        b("KeepSprint", s.keep_sprint, "KeepSprint"),
        b(
            "IgnoreOpenInventory",
            s.ignore_inventory,
            "IgnoreOpenInventory",
        ),
        b(
            "SimulateInventoryClosing",
            s.close_inventory,
            "SimulateInventoryClosing",
        ),
    ]);
    let a = &s.block;
    values.push(t.group(
        "AutoBlocking",
        "TOGGLEABLE",
        vec![
            b("Enabled", a.enabled, "AutoBlocking.Enabled"),
            choice("BlockMode", a.mode, &BLOCK, "AutoBlocking.BlockMode"),
            b(
                "SimulateVanillaUse",
                a.vanilla,
                "AutoBlocking.SimulateVanillaUse",
            ),
            choice(
                "UnblockMode",
                a.unblock,
                &UNBLOCK,
                "AutoBlocking.UnblockMode",
            ),
            range(
                "Reblock",
                "INT_RANGE",
                int_pair(a.reblock),
                0.,
                3.,
                "ticks",
                "AutoBlocking.Reblock",
            ),
            range(
                "PauseOnUnblock",
                "INT_RANGE",
                int_pair(a.pause),
                0.,
                3.,
                "ticks",
                "AutoBlocking.PauseOnUnblock",
            ),
            range(
                "Chance",
                "FLOAT",
                json!(a.chance),
                0.,
                100.,
                "%",
                "AutoBlocking.Chance",
            ),
            range(
                "Blink",
                "INT",
                json!(a.blink),
                0.,
                10.,
                "ticks",
                "AutoBlocking.Blink",
            ),
            b(
                "PrioritizeBlocking",
                a.prioritize,
                "AutoBlocking.PrioritizeBlocking",
            ),
            b("OnScanRange", a.scan, "AutoBlocking.OnScanRange"),
            t.group(
                "OnlyWhenInDanger",
                "TOGGLEABLE",
                vec![
                    b("Enabled", a.danger, "AutoBlocking.OnlyWhenInDanger.Enabled"),
                    range(
                        "Tolerance",
                        "FLOAT",
                        json!(a.tolerance),
                        0.,
                        1.,
                        "blocks",
                        "AutoBlocking.OnlyWhenInDanger.Tolerance",
                    ),
                    range(
                        "ForceActiveRange",
                        "FLOAT_RANGE",
                        float_pair(a.force),
                        0.,
                        6.,
                        "",
                        "AutoBlocking.OnlyWhenInDanger.ForceActiveRange",
                    ),
                ],
            ),
            b("AssumeShield", a.assume, "AutoBlocking.AssumeShield"),
        ],
    ));
    let a = &s.fail;
    let mut modes = Map::new();
    modes.insert("None".into(), t.group("None", "CONFIGURABLE", vec![]));
    modes.insert(
        "Box".into(),
        t.group(
            "Box",
            "CONFIGURABLE",
            vec![
                range(
                    "Fade",
                    "INT",
                    json!(a.fade),
                    1.,
                    10.,
                    "secs",
                    "FailSwing.NotifyWhenFail.Box.Fade",
                ),
                leaf(
                    "Color",
                    "COLOR",
                    json!(a.color as i32),
                    "FailSwing.NotifyWhenFail.Box.Color",
                ),
                b("Rainbow", a.rainbow, "FailSwing.NotifyWhenFail.Box.Rainbow"),
            ],
        ),
    );
    modes.insert(
        "Sound".into(),
        t.group(
            "Sound",
            "CONFIGURABLE",
            vec![
                range(
                    "Volume",
                    "FLOAT",
                    json!(a.volume),
                    0.,
                    100.,
                    "%",
                    "FailSwing.NotifyWhenFail.Sound.Volume",
                ),
                range(
                    "Pitch",
                    "FLOAT",
                    json!(a.pitch),
                    0.,
                    2.,
                    "",
                    "FailSwing.NotifyWhenFail.Sound.Pitch",
                ),
            ],
        ),
    );
    let mut notify = t.group("NotifyWhenFail", "CHOICE", vec![]);
    notify["active"] = json!(NOTIFY[a.mode as usize]);
    notify["choices"] = Value::Object(modes);
    values.push(t.group(
        "FailSwing",
        "TOGGLEABLE",
        vec![
            b("Enabled", a.enabled, "FailSwing.Enabled"),
            range(
                "AdditionalRange",
                "FLOAT_RANGE",
                float_pair(a.additional),
                0.,
                10.,
                "blocks",
                "FailSwing.AdditionalRange",
            ),
            notify,
        ],
    ));
    let a = &s.fight;
    let nav = super::auto_walk::serialize_auto_walk(&r.navigation, c, f, &|k| {
        d(&k.replace(
            "liquidbounce.module.autoWalk",
            "liquidbounce.module.killAura.fightBot",
        ))
    })?;
    let mut fight = vec![b("Enabled", a.enabled, "FightBot.Enabled")];
    for mut v in nav["value"].as_array().cloned().unwrap_or_default() {
        if [
            "Mode",
            "Rotations",
            "Enabled",
            "Bind",
            "Hidden",
            "OnDisable",
            "OnEnable",
        ]
        .contains(&v["name"].as_str().unwrap_or(""))
        {
            continue;
        }
        remap(
            &mut v,
            "liquidbounce.module.autoWalk",
            "liquidbounce.module.killAura.fightBot",
        );
        fight.push(v);
    }
    fight.extend([
        range(
            "OpponentRange",
            "FLOAT",
            json!(a.opponent),
            0.1,
            10.,
            "blocks",
            "FightBot.OpponentRange",
        ),
        range(
            "DangerousYaw",
            "FLOAT",
            json!(a.yaw),
            0.,
            90.,
            "°",
            "FightBot.DangerousYaw",
        ),
        b("RunawayOnCooldown", a.runaway, "FightBot.RunawayOnCooldown"),
        t.group(
            "TargetFilter",
            "CONFIGURABLE",
            vec![
                range(
                    "Range",
                    "FLOAT",
                    json!(a.range),
                    10.,
                    100.,
                    "blocks",
                    "FightBot.TargetFilter.Range",
                ),
                b(
                    "VisibleOnly",
                    a.visible,
                    "FightBot.TargetFilter.VisibleOnly",
                ),
                b("NotWhenVoid", a.void, "FightBot.TargetFilter.NotWhenVoid"),
            ],
        ),
        t.group(
            "Leader",
            "TOGGLEABLE",
            vec![
                b("Enabled", a.leader, "FightBot.Leader.Enabled"),
                leaf(
                    "Username",
                    "TEXT",
                    json!(a.username),
                    "FightBot.Leader.Username",
                ),
                range(
                    "Radius",
                    "FLOAT",
                    json!(a.radius),
                    2.,
                    10.,
                    "blocks",
                    "FightBot.Leader.Radius",
                ),
            ],
        ),
    ]);
    values.push(t.group("FightBot", "TOGGLEABLE", fight));
    let i = &s.indicator;
    let mut indicator = vec![
        b("Enabled", i.enabled, "RangeIndicator.Enabled"),
        choice("ColorMode", i.mode, &COLOR, "RangeIndicator.ColorMode"),
    ];
    for (name, v) in [
        ("IdleColor", i.idle),
        ("ActiveColor", i.active),
        ("OutlineColor", i.outline_color),
        ("WallRangeColor", i.walls),
        ("ScanRangeColor", i.scan),
        ("OpponentRangeColor", i.opponent),
    ] {
        indicator.push(leaf(
            name,
            "COLOR",
            json!(v as i32),
            &format!("RangeIndicator.{name}"),
        ));
    }
    for (name, v) in [
        ("Outline", i.outline),
        ("PulseAnimation", i.pulse),
        ("FadeAnimation", i.fade),
        ("HideWhenDead", i.hide_dead),
        ("HideWhenSpectator", i.hide_spectator),
        ("HideInVehicle", i.hide_vehicle),
        ("RespectInventorySetting", i.inventory),
        ("CanBeCovered", i.covered),
    ] {
        indicator.push(b(name, v, &format!("RangeIndicator.{name}")));
    }
    for (name, v, lo, hi) in [
        ("PulseSpeed", i.pulse_speed, 0.5, 5.),
        ("PulseIntensity", i.pulse_intensity, 0.05, 0.5),
        ("FadeSpeed", i.fade_speed, 0.01, 0.5),
    ] {
        indicator.push(range(
            name,
            "FLOAT",
            json!(v),
            lo,
            hi,
            "",
            &format!("RangeIndicator.{name}"),
        ));
    }
    values.push(t.group("RangeIndicator", "TOGGLEABLE", indicator));
    values.push(target_renderer::serialize_target_renderer(
        &r.target_renderer,
        f,
        d,
        "liquidbounce.module.killAura.targetRendering",
    )?);
    Ok(t.group("KillAura", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, path: &str, v: &Value) {
    let c = KillAuraSetting::Value(path.into(), v.clone());
    match c.clone().apply(&mut KillAuraRuntime::default()) {
        Ok(()) => p.changes.push(SettingChange::KillAura(c)),
        Err(e) => p.issues.push(e),
    }
}
fn walk(p: &mut SettingsPatch, v: &Value, path: &str) {
    if let Some(active) = v.get("active") {
        add(p, path, active);
    }
    let name = path.rsplit('.').next().unwrap_or(path);
    let named_children = v
        .get("value")
        .and_then(Value::as_array)
        .is_some_and(|a| a.iter().any(|n| n.get("name").is_some()));
    let known_group = [
        "Clicker",
        "Clicker.ItemCooldown",
        "Range",
        "Target",
        "AutoBlocking",
        "AutoBlocking.OnlyWhenInDanger",
        "FailSwing",
        "FailSwing.NotifyWhenFail",
        "FailSwing.NotifyWhenFail.None",
        "FailSwing.NotifyWhenFail.Box",
        "FailSwing.NotifyWhenFail.Sound",
        "FightBot.TargetFilter",
        "FightBot.Leader",
        "RangeIndicator",
    ]
    .contains(&path);
    if named_children || known_group || v.get("active").is_some() {
        match nodes(v, name, &format!("KillAura.{path}")) {
            Ok(n) => {
                for (name, v) in n {
                    walk(p, v, &format!("{path}.{name}"));
                }
            }
            Err(e) => p.issues.push(e),
        }
        if let Some(choices) = v.get("choices").and_then(Value::as_object) {
            for (name, v) in choices {
                walk(p, v, &format!("{path}.{name}"));
            }
        }
    } else {
        match value_of(v, &format!("KillAura.{path}")) {
            Ok(v) => add(p, path, v),
            Err(e) => p.issues.push(e),
        }
    }
}
pub(super) fn parse_kill_aura(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let legacy_range = root.contains_key("WallRange") && root.contains_key("ScanExtraRange");
    if legacy_range {
        if let Some(range) = root.get("Range") {
            match value_of(range, "KillAura.Range").and_then(|v| parse_float(v, "KillAura.Range")) {
                Ok(v) => add(p, "Range.RangeIncrease", &json!((v - 3.).max(0.))),
                Err(e) => p.issues.push(e),
            }
        }
        for (old, new) in [
            ("WallRange", "Range.ThroughWallsRange"),
            ("ScanExtraRange", "Range.ScanRangeIncrease"),
        ] {
            if let Some(value) = root.get(old) {
                match value_of(value, &format!("KillAura.{old}")) {
                    Ok(value) => add(p, new, value),
                    Err(e) => p.issues.push(e),
                }
            }
        }
    }
    for (name, v) in root {
        if legacy_range && ["Range", "WallRange", "ScanExtraRange"].contains(&name.as_str()) {
            continue;
        }
        match name.as_str() {
            "Enabled" | "Bind" | "Hidden" | "OnEnable" | "OnDisable" => {}
            "AimPoint" => {
                let proxy = json!({"name":"Aimbot","value":[v]});
                let n = nodes(&proxy, "Aimbot", "Aimbot").expect("AimPoint proxy");
                let mut q = SettingsPatch {
                    module: ModuleKind::Aimbot,
                    changes: vec![],
                    issues: vec![],
                };
                super::aimbot::parse_aimbot(&mut q, &n);
                for c in q.changes {
                    if let SettingChange::Aimbot(super::aimbot::AimbotSetting::Point(path, v)) = c {
                        p.changes
                            .push(SettingChange::KillAura(KillAuraSetting::Point(path, v)));
                    }
                }
                p.issues.extend(q.issues.into_iter().map(|mut e| {
                    e.path = e.path.replace("Aimbot", "KillAura");
                    e
                }));
            }
            "TargetRendering" => {
                let (changes, issues) =
                    target_renderer::parse_target_renderer(v, "KillAura.TargetRendering");
                p.changes.extend(
                    changes
                        .into_iter()
                        .map(|v| SettingChange::KillAura(KillAuraSetting::Rendering(v))),
                );
                p.issues.extend(issues);
            }
            "Rotations" => {
                let (changes, issues) = combat_rotations::parse(v, "KillAura.Rotations");
                p.changes.extend(
                    changes
                        .into_iter()
                        .map(|v| SettingChange::KillAura(KillAuraSetting::Rotation(v))),
                );
                p.issues.extend(issues);
                match nodes(v, "Rotations", "KillAura.Rotations") {
                    Ok(n) => {
                        for name in ["RotationTiming", "ThroughWalls", "LazyRotation"] {
                            if let Some(v) = n.get(name) {
                                walk(p, v, &format!("Rotations.{name}"));
                            }
                        }
                    }
                    Err(e) => p.issues.push(e),
                }
            }
            "FightBot" => match nodes(v, "FightBot", "KillAura.FightBot") {
                Ok(n) => {
                    let mut q = SettingsPatch {
                        module: ModuleKind::AutoWalk,
                        changes: vec![],
                        issues: vec![],
                    };
                    super::auto_walk::parse_auto_walk(&mut q, &n);
                    for c in q.changes {
                        if let SettingChange::AutoWalk(v) = c {
                            p.changes
                                .push(SettingChange::KillAura(KillAuraSetting::Navigation(v)));
                        }
                    }
                    p.issues.extend(q.issues.into_iter().map(|mut e| {
                        e.path = e.path.replace("AutoWalk", "KillAura.FightBot");
                        e
                    }));
                    for name in [
                        "Enabled",
                        "OpponentRange",
                        "DangerousYaw",
                        "RunawayOnCooldown",
                        "TargetFilter",
                        "Leader",
                    ] {
                        if let Some(v) = n.get(name) {
                            walk(p, v, &format!("FightBot.{name}"));
                        }
                    }
                }
                Err(e) => p.issues.push(e),
            },
            _ => walk(p, v, name),
        }
    }
}

#[cfg(test)]
mod kill_aura_config_tests {
    use super::*;
    fn apply(r: &mut KillAuraRuntime, p: SettingsPatch) {
        assert!(p.issues.is_empty(), "{:?}", p.issues);
        for change in p.changes {
            if let SettingChange::KillAura(change) = change {
                change.apply(r).unwrap();
            }
        }
    }
    #[test]
    fn file_and_interop_preserve_inactive_notify_navigation_and_rotations() {
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let mut r = KillAuraRuntime::default();
            r.behavior.settings.increase = 1.25;
            r.behavior.settings.walls = 2.5;
            r.behavior.settings.block.mode = 2;
            r.behavior.settings.block.reblock = [1, 3];
            r.behavior.settings.fail.mode = 2;
            r.behavior.settings.fail.fade = 7;
            r.behavior.settings.fail.color = 0xdecafbad;
            r.behavior.settings.fail.rainbow = true;
            r.behavior.settings.fail.volume = 31.;
            r.behavior.settings.fight.username = "AkibaCat".into();
            r.behavior.settings.requirements = [true, false, true, false, false, true];
            r.navigation.allow_break = false;
            r.navigation.max_fall = 7;
            r.navigation.render.enabled = true;
            r.navigation.render.route = 0xfa341278;
            r.point_tracker.settings.delay = true;
            r.point_tracker.settings.delay_ticks = [2, 3];
            r.rotations.interpolation.horizontal = [63, 71];
            let value = serialize_kill_aura(
                &r,
                &CommonSettings {
                    enabled: false,
                    hidden: false,
                    key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Toggle"}),
                },
                format,
                &str::to_owned,
            )
            .unwrap();
            let patch = parse_settings(ModuleKind::KillAura, &value).unwrap();
            let mut restored = KillAuraRuntime::default();
            apply(&mut restored, patch);
            assert_eq!(r.behavior.settings, restored.behavior.settings);
            assert_eq!(r.point_tracker.settings, restored.point_tracker.settings);
            assert_eq!(
                r.rotations.interpolation.horizontal,
                restored.rotations.interpolation.horizontal
            );
            assert_eq!(r.navigation.allow_break, restored.navigation.allow_break);
            assert_eq!(r.navigation.max_fall, restored.navigation.max_fall);
            assert_eq!(r.navigation.render.route, restored.navigation.render.route);
            assert!(restored.navigation.render.enabled);
        }
    }
    #[test]
    fn malformed_leaf_keeps_siblings_and_old_tick_aliases() {
        let value = json!({"name":"KillAura","value":[{"name":"Range","value":[{"name":"RangeIncrease","value":2.0},{"name":"ThroughWallsRange","value":"bad"}]},{"name":"AutoBlocking","value":[{"name":"TickOn","value":{"from":1,"to":2}},{"name":"TickOff","value":3}]}]});
        let patch = parse_settings(ModuleKind::KillAura, &value).unwrap();
        assert_eq!(patch.issues.len(), 1);
        let mut r = KillAuraRuntime::default();
        for c in patch.changes {
            if let SettingChange::KillAura(c) = c {
                c.apply(&mut r).unwrap();
            }
        }
        assert_eq!(r.behavior.settings.increase, 2.);
        assert_eq!(r.behavior.settings.block.reblock, [1, 2]);
        assert_eq!(r.behavior.settings.block.pause, [3, 3]);
    }
    #[test]
    fn non_finite_json_is_rejected_and_range_assignment_clamps_once() {
        let mut r = KillAuraRuntime::default();
        for (path, value) in [
            ("Range.RangeIncrease", json!(5.)),
            ("Range.ThroughWallsRange", json!(7.)),
            ("Range.RangeIncrease", json!(0.)),
        ] {
            KillAuraSetting::Value(path.into(), value)
                .apply(&mut r)
                .unwrap();
        }
        assert_eq!(r.behavior.settings.walls, 7.);
        r.behavior.settings.increase = f32::NAN;
        assert!(
            serialize_kill_aura(
                &r,
                &CommonSettings {
                    enabled: false,
                    hidden: false,
                    key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Toggle"})
                },
                JsonFormat::File,
                &str::to_owned
            )
            .is_err()
        );
    }
    #[test]
    fn legacy_absolute_range_values_migrate_to_the_current_range_group() {
        let value = json!({"name":"KillAura","value":[{"name":"Range","value":4.5},{"name":"WallRange","value":5.0},{"name":"ScanExtraRange","value":{"from":2.0,"to":4.0}}]});
        let mut r = KillAuraRuntime::default();
        apply(
            &mut r,
            parse_settings(ModuleKind::KillAura, &value).unwrap(),
        );
        assert_eq!(r.behavior.settings.increase, 1.5);
        assert_eq!(r.behavior.settings.walls, 4.5);
        assert_eq!(r.behavior.settings.scan, [2., 4.]);
    }
}
