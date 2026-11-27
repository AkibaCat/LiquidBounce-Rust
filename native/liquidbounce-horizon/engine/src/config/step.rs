// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::step::{MODES, Mode, Settings};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepSetting {
    Mode(Mode),
    Height(f32),
    Trim(bool),
    JumpOrder([i32; 2]),
    Wait([i32; 2]),
    FullPacket(bool),
    BaseTimer(f32),
    RecoveryTimer(f32),
    AlternateBypass(bool),
    Spoof(bool),
}
impl StepSetting {
    pub fn apply(self, s: &mut Settings) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::Height(v) => s.height = v,
            Self::Trim(v) => s.trim = v,
            Self::JumpOrder(v) => s.simulate_jump_order = v,
            Self::Wait(v) => s.wait = v,
            Self::FullPacket(v) => s.full_packet = v,
            Self::BaseTimer(v) => s.base_timer = v,
            Self::RecoveryTimer(v) => s.recovery_timer = v,
            Self::AlternateBypass(v) => s.alternate_bypass = v,
            Self::Spoof(v) => s.spoof = v,
        }
    }
}
pub fn serialize_step(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::Step, c);
    let mut choices = Map::new();
    let mut packet = t.leaf(
        "PacketType",
        "CHOOSE",
        json!(if s.full_packet {
            "Full"
        } else {
            "PositionAndOnGround"
        }),
        "liquidbounce.module.step.mode.instant.packetType",
        None,
    );
    if f == JsonFormat::Interop {
        packet["choices"] = json!(["Full", "PositionAndOnGround"]);
    }
    choices.insert(
        "Instant".into(),
        t.group(
            "Instant",
            "CONFIGURABLE",
            vec![
                t.ranged(
                    "Height",
                    "FLOAT",
                    float_json(s.height, "Step.Height")?,
                    json!({"from":0.6,"to":5.}),
                    "",
                    "liquidbounce.module.step.mode.instant.height",
                ),
                t.leaf(
                    "Trim",
                    "BOOLEAN",
                    json!(s.trim),
                    "liquidbounce.module.step.mode.instant.trim",
                    None,
                ),
                t.ranged(
                    "SimulateJumpOrder",
                    "INT_RANGE",
                    json!({"from":s.simulate_jump_order[0],"to":s.simulate_jump_order[1]}),
                    json!({"from":0,"to":8}),
                    "",
                    "liquidbounce.module.step.mode.instant.simulateJumpOrder",
                ),
                t.ranged(
                    "Wait",
                    "INT_RANGE",
                    json!({"from":s.wait[0],"to":s.wait[1]}),
                    json!({"from":0,"to":60}),
                    "ticks",
                    "liquidbounce.module.step.mode.instant.wait",
                ),
                packet,
            ],
        ),
    );
    for tag in ["Legit", "Vulcan-2.8.6"] {
        choices.insert(tag.into(), t.group(tag, "CONFIGURABLE", vec![]));
    }
    choices.insert(
        "BlocksMC".into(),
        t.group(
            "BlocksMC",
            "CONFIGURABLE",
            vec![
                t.ranged(
                    "BaseTimer",
                    "FLOAT",
                    float_json(s.base_timer, "Step.BaseTimer")?,
                    json!({"from":0.1,"to":5.}),
                    "",
                    "liquidbounce.module.step.mode.blocksMC.baseTimer",
                ),
                t.ranged(
                    "RecoveryTimer",
                    "FLOAT",
                    float_json(s.recovery_timer, "Step.RecoveryTimer")?,
                    json!({"from":0.1,"to":5.}),
                    "",
                    "liquidbounce.module.step.mode.blocksMC.recoveryTimer",
                ),
            ],
        ),
    );
    choices.insert(
        "Hypixel-2025-03-11".into(),
        t.group(
            "Hypixel-2025-03-11",
            "CONFIGURABLE",
            vec![
                t.leaf(
                    "AlternateBypass",
                    "BOOLEAN",
                    json!(s.alternate_bypass),
                    "liquidbounce.module.step.mode.hypixel-2025-03-11.alternateBypass",
                    None,
                ),
                t.leaf(
                    "Spoof",
                    "BOOLEAN",
                    json!(s.spoof),
                    "liquidbounce.module.step.mode.hypixel-2025-03-11.spoof",
                    None,
                ),
            ],
        ),
    );
    values.push(t.mode(s.mode.tag(), choices));
    Ok(t.group("Step", "TOGGLEABLE", values))
}
pub(super) fn parse_step(p: &mut SettingsPatch, n: &Nodes<'_>) {
    let Some(mode) = n.get("Mode") else {
        return;
    };
    match mode["active"].as_str().and_then(Mode::parse) {
        Some(v) => p.changes.push(SettingChange::Step(StepSetting::Mode(v))),
        None => p
            .issues
            .push(ConfigError::new("Step.Mode.active", "unknown Step mode")),
    }
    let Some(choices) = mode["choices"].as_object() else {
        p.issues.push(ConfigError::new(
            "Step.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for tag in MODES {
        let path = format!("Step.Mode.{tag}");
        let Some(value) = choices.get(tag) else {
            p.issues.push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        let n = match nodes(value, tag, &path) {
            Ok(n) => n,
            Err(e) => {
                p.issues.push(e);
                continue;
            }
        };
        for (name, make) in match tag {
            "Instant" => vec![("Height", StepSetting::Height as fn(f32) -> StepSetting)],
            "BlocksMC" => vec![
                (
                    "BaseTimer",
                    StepSetting::BaseTimer as fn(f32) -> StepSetting,
                ),
                (
                    "RecoveryTimer",
                    StepSetting::RecoveryTimer as fn(f32) -> StepSetting,
                ),
            ],
            _ => vec![],
        } {
            leaf(p, &n, name, &format!("{path}.{name}"), |v, path| {
                parse_float(v, path).map(|v| SettingChange::Step(make(v)))
            });
        }
        for (name, make) in match tag {
            "Instant" => vec![("Trim", StepSetting::Trim as fn(bool) -> StepSetting)],
            "Hypixel-2025-03-11" => vec![
                (
                    "AlternateBypass",
                    StepSetting::AlternateBypass as fn(bool) -> StepSetting,
                ),
                ("Spoof", StepSetting::Spoof as fn(bool) -> StepSetting),
            ],
            _ => vec![],
        } {
            leaf(p, &n, name, &format!("{path}.{name}"), |v, path| {
                parse_bool(v, path).map(|v| SettingChange::Step(make(v)))
            });
        }
        if tag == "Instant" {
            for (name, max, make) in [
                (
                    "SimulateJumpOrder",
                    8,
                    StepSetting::JumpOrder as fn([i32; 2]) -> StepSetting,
                ),
                ("Wait", 60, StepSetting::Wait as fn([i32; 2]) -> StepSetting),
            ] {
                leaf(p, &n, name, &format!("{path}.{name}"), |v, path| {
                    parse_int_pair(v, path).and_then(|v| {
                        if v[0] >= 0 && v[0] <= v[1] && v[1] <= max {
                            Ok(SettingChange::Step(make(v)))
                        } else {
                            Err(ConfigError::new(path, "invalid ordered range"))
                        }
                    })
                });
            }
            leaf(
                p,
                &n,
                "PacketType",
                &format!("{path}.PacketType"),
                |v, path| match v.as_str() {
                    Some("Full") => Ok(SettingChange::Step(StepSetting::FullPacket(true))),
                    Some("PositionAndOnGround") => {
                        Ok(SettingChange::Step(StepSetting::FullPacket(false)))
                    }
                    _ => Err(ConfigError::new(path, "unknown movement packet type")),
                },
            );
        }
    }
}
