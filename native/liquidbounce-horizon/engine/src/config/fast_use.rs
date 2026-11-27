// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    fast_use::{CONDITIONS, FastUse, MODES, Mode},
    old_regen::{PACKET_TYPES, packet_type},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FastUseSetting {
    Mode(Mode),
    Conditions(u32),
    StopInput(bool),
    PacketType(u8),
    Delay(i32),
    Timer(f32),
    ImmediateSpeed(i32),
    ConsumeTime(i32),
    TimedSpeed(i32),
    TickCooldown(i32),
}
impl FastUseSetting {
    pub fn apply(self, module: &mut FastUse) {
        match self {
            Self::Mode(v) => module.set_mode(v),
            Self::Conditions(v) => module.settings.conditions = v,
            Self::StopInput(v) => module.settings.stop_input = v,
            Self::PacketType(v) => module.settings.packet_type = v,
            Self::Delay(v) => module.settings.delay = v,
            Self::Timer(v) => module.settings.timer = v,
            Self::ImmediateSpeed(v) => module.settings.immediate_speed = v,
            Self::ConsumeTime(v) => module.settings.consume_time = v,
            Self::TimedSpeed(v) => module.settings.timed_speed = v,
            Self::TickCooldown(v) => module.settings.tick_cooldown = v,
        }
    }
}
pub fn serialize_fast_use(
    module: &FastUse,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let s = module.settings;
    let mut values = tree.common(ModuleKind::FastUse, common);
    let mut choices = Map::new();
    choices.insert(
        MODES[0].into(),
        tree.group(
            MODES[0],
            "CONFIGURABLE",
            vec![
                tree.ranged(
                    "Delay",
                    "INT",
                    json!(s.delay),
                    json!({"from":0,"to":10}),
                    "ticks",
                    "liquidbounce.module.fastUse.mode.immediate.delay",
                ),
                tree.ranged(
                    "Timer",
                    "FLOAT",
                    float_json(s.timer, "FastUse.Mode.Immediate.Timer")?,
                    json!({"from":0.1,"to":5.0}),
                    "",
                    "liquidbounce.module.fastUse.mode.immediate.timer",
                ),
                tree.ranged(
                    "Speed",
                    "INT",
                    json!(s.immediate_speed),
                    json!({"from":1,"to":35}),
                    "packets",
                    "liquidbounce.module.fastUse.mode.immediate.speed",
                ),
            ],
        ),
    );
    choices.insert(
        MODES[1].into(),
        tree.group(
            MODES[1],
            "CONFIGURABLE",
            vec![
                tree.ranged(
                    "ConsumeTime",
                    "INT",
                    json!(s.consume_time),
                    json!({"from":0,"to":20}),
                    "",
                    "liquidbounce.module.fastUse.mode.itemUseTime.consumeTime",
                ),
                tree.ranged(
                    "Speed",
                    "INT",
                    json!(s.timed_speed),
                    json!({"from":1,"to":35}),
                    "packets",
                    "liquidbounce.module.fastUse.mode.itemUseTime.speed",
                ),
            ],
        ),
    );
    choices.insert(
        MODES[2].into(),
        tree.group(
            MODES[2],
            "CONFIGURABLE",
            vec![tree.ranged(
                "TickCooldown",
                "INT",
                json!(s.tick_cooldown),
                json!({"from":1,"to":20}),
                "",
                "liquidbounce.module.fastUse.mode.crossbow.tickCooldown",
            )],
        ),
    );
    values.push(tree.mode(s.mode.tag(), choices));
    values.push(tree.multi(
        "Conditions",
        s.conditions,
        &CONDITIONS,
        "liquidbounce.module.fastUse.conditions",
    ));
    values.push(tree.leaf(
        "StopInput",
        "BOOLEAN",
        json!(s.stop_input),
        "liquidbounce.module.fastUse.stopInput",
        None,
    ));
    let mut packet = tree.leaf(
        "PacketType",
        "CHOOSE",
        json!(PACKET_TYPES[s.packet_type as usize]),
        "liquidbounce.module.fastUse.packetType",
        None,
    );
    if format == JsonFormat::Interop {
        packet["choices"] = json!(PACKET_TYPES);
    }
    values.push(packet);
    Ok(tree.group("FastUse", "TOGGLEABLE", values))
}
pub(super) fn parse_fast_use(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Conditions", "FastUse.Conditions", |v, p| {
        parse_multi_mask(v, p, &CONDITIONS)
            .map(|v| SettingChange::FastUse(FastUseSetting::Conditions(v)))
    });
    leaf(patch, root, "StopInput", "FastUse.StopInput", |v, p| {
        parse_bool(v, p).map(|v| SettingChange::FastUse(FastUseSetting::StopInput(v)))
    });
    leaf(patch, root, "PacketType", "FastUse.PacketType", |v, p| {
        scalar_string(v)
            .and_then(|v| packet_type(&v))
            .map(|v| SettingChange::FastUse(FastUseSetting::PacketType(v)))
            .ok_or_else(|| ConfigError::new(p, "unknown packet type"))
    });
    let Some(mode) = root.get("Mode") else {
        return;
    };
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|v| Mode::parse(&v))
    {
        Some(v) => patch
            .changes
            .push(SettingChange::FastUse(FastUseSetting::Mode(v))),
        None => patch
            .issues
            .push(ConfigError::new("FastUse.Mode.active", "unknown mode")),
    }
    if let Err(e) = nodes(mode, "Mode", "FastUse.Mode") {
        patch.issues.push(e);
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch.issues.push(ConfigError::new(
            "FastUse.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for tag in MODES {
        let path = format!("FastUse.Mode.{tag}");
        let Some(choice) = choices.get(tag) else {
            patch
                .issues
                .push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        let children = match nodes(choice, tag, &path) {
            Ok(v) => v,
            Err(e) => {
                patch.issues.push(e);
                continue;
            }
        };
        match tag {
            "Immediate" => {
                leaf(
                    patch,
                    &children,
                    "Delay",
                    &format!("{path}.Delay"),
                    |v, p| {
                        parse_int(v, p).map(|v| SettingChange::FastUse(FastUseSetting::Delay(v)))
                    },
                );
                leaf(
                    patch,
                    &children,
                    "Timer",
                    &format!("{path}.Timer"),
                    |v, p| {
                        parse_float(v, p).map(|v| SettingChange::FastUse(FastUseSetting::Timer(v)))
                    },
                );
                leaf(
                    patch,
                    &children,
                    "Speed",
                    &format!("{path}.Speed"),
                    |v, p| {
                        parse_int(v, p)
                            .map(|v| SettingChange::FastUse(FastUseSetting::ImmediateSpeed(v)))
                    },
                );
            }
            "ItemUseTime" => {
                leaf(
                    patch,
                    &children,
                    "ConsumeTime",
                    &format!("{path}.ConsumeTime"),
                    |v, p| {
                        parse_int(v, p)
                            .map(|v| SettingChange::FastUse(FastUseSetting::ConsumeTime(v)))
                    },
                );
                leaf(
                    patch,
                    &children,
                    "Speed",
                    &format!("{path}.Speed"),
                    |v, p| {
                        parse_int(v, p)
                            .map(|v| SettingChange::FastUse(FastUseSetting::TimedSpeed(v)))
                    },
                );
            }
            _ => leaf(
                patch,
                &children,
                "TickCooldown",
                &format!("{path}.TickCooldown"),
                |v, p| {
                    parse_int(v, p).map(|v| SettingChange::FastUse(FastUseSetting::TickCooldown(v)))
                },
            ),
        }
    }
}
