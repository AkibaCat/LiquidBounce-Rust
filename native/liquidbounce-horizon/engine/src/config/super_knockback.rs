// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::super_knockback::{CONDITIONS, MODES, Mode, SuperKnockback};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SuperKnockbackSetting {
    Mode(Mode),
    HurtTime(i32),
    Chance(f32),
    Conditions(u32),
    OnlyOnMove(bool),
    OnlyForward(bool),
    ReSprint([i32; 2]),
    UntilMovementBlock([i32; 2]),
    UntilAllowedMovement([i32; 2]),
}
impl SuperKnockbackSetting {
    pub fn apply(self, m: &mut SuperKnockback) {
        match self {
            Self::Mode(v) => m.set_mode(v),
            Self::HurtTime(v) => m.settings.hurt_time = v,
            Self::Chance(v) => m.settings.chance = v,
            Self::Conditions(v) => m.settings.conditions = v,
            Self::OnlyOnMove(v) => m.settings.only_on_move = v,
            Self::OnlyForward(v) => m.settings.only_forward = v,
            Self::ReSprint(v) => m.settings.re_sprint = v,
            Self::UntilMovementBlock(v) => m.settings.until_movement_block = v,
            Self::UntilAllowedMovement(v) => m.settings.until_allowed_movement = v,
        }
    }
}
pub fn serialize_super_knockback(
    m: &SuperKnockback,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let s = m.settings;
    let mut values = tree.common(ModuleKind::SuperKnockback, common);
    let mut choices = Map::new();
    let delay = |mode: &str, name: &str, key: &str, v: [i32; 2]| {
        tree.ranged(
            name,
            "INT_RANGE",
            json!({"from":v[0],"to":v[1]}),
            json!({"from":0,"to":10}),
            "ticks",
            &format!("liquidbounce.module.superKnockback.mode.{mode}.{key}"),
        )
    };
    choices.insert(
        MODES[0].into(),
        tree.group(MODES[0], "CONFIGURABLE", vec![]),
    );
    choices.insert(
        MODES[1].into(),
        tree.group(
            MODES[1],
            "CONFIGURABLE",
            vec![delay("sprintTap", "ReSprint", "reSprint", s.re_sprint)],
        ),
    );
    choices.insert(
        MODES[2].into(),
        tree.group(
            MODES[2],
            "CONFIGURABLE",
            vec![
                delay(
                    "wTap",
                    "UntilMovementBlock",
                    "untilMovementBlock",
                    s.until_movement_block,
                ),
                delay(
                    "wTap",
                    "UntilAllowedMovement",
                    "untilAllowedMovement",
                    s.until_allowed_movement,
                ),
            ],
        ),
    );
    values.push(tree.mode(s.mode.tag(), choices));
    values.push(tree.ranged(
        "HurtTime",
        "INT",
        json!(s.hurt_time),
        json!({"from":0,"to":10}),
        "",
        "liquidbounce.module.superKnockback.hurtTime",
    ));
    values.push(tree.ranged(
        "Chance",
        "FLOAT",
        float_json(s.chance, "SuperKnockback.Chance")?,
        json!({"from":0.0,"to":100.0}),
        "%",
        "liquidbounce.module.superKnockback.chance",
    ));
    values.push(tree.multi(
        "Conditions",
        s.conditions,
        &CONDITIONS,
        "liquidbounce.module.superKnockback.conditions",
    ));
    let mut moving = tree.group(
        "OnlyOnMove",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.only_on_move),
                "liquidbounce.module.superKnockback.onlyOnMove.enabled",
                None,
            ),
            tree.leaf(
                "OnlyForward",
                "BOOLEAN",
                json!(s.only_forward),
                "liquidbounce.module.superKnockback.onlyOnMove.onlyForward",
                None,
            ),
        ],
    );
    if format == JsonFormat::Interop {
        moving["key"] = json!("liquidbounce.module.superKnockback.onlyOnMove");
        moving["description"] = json!(describe(
            "liquidbounce.module.superKnockback.onlyOnMove.description"
        ));
    }
    values.push(moving);
    Ok(tree.group("SuperKnockback", "TOGGLEABLE", values))
}
pub(super) fn parse_super_knockback(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    use SuperKnockbackSetting as S;
    leaf(
        patch,
        root,
        "HurtTime",
        "SuperKnockback.HurtTime",
        |v, p| parse_int(v, p).map(|v| SettingChange::SuperKnockback(S::HurtTime(v))),
    );
    leaf(patch, root, "Chance", "SuperKnockback.Chance", |v, p| {
        parse_float(v, p).map(|v| SettingChange::SuperKnockback(S::Chance(v)))
    });
    leaf(
        patch,
        root,
        "Conditions",
        "SuperKnockback.Conditions",
        |v, p| {
            parse_multi_mask(v, p, &CONDITIONS)
                .map(|v| SettingChange::SuperKnockback(S::Conditions(v)))
        },
    );
    if let Some(v) = root.get("OnlyOnMove") {
        match nodes(v, "OnlyOnMove", "SuperKnockback.OnlyOnMove") {
            Ok(v) => {
                leaf(
                    patch,
                    &v,
                    "Enabled",
                    "SuperKnockback.OnlyOnMove.Enabled",
                    |v, p| {
                        parse_bool(v, p).map(|v| SettingChange::SuperKnockback(S::OnlyOnMove(v)))
                    },
                );
                leaf(
                    patch,
                    &v,
                    "OnlyForward",
                    "SuperKnockback.OnlyOnMove.OnlyForward",
                    |v, p| {
                        parse_bool(v, p).map(|v| SettingChange::SuperKnockback(S::OnlyForward(v)))
                    },
                );
            }
            Err(e) => patch.issues.push(e),
        }
    }
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
            .push(SettingChange::SuperKnockback(S::Mode(v))),
        None => patch.issues.push(ConfigError::new(
            "SuperKnockback.Mode.active",
            "unknown mode",
        )),
    }
    if let Err(e) = nodes(mode, "Mode", "SuperKnockback.Mode") {
        patch.issues.push(e);
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch.issues.push(ConfigError::new(
            "SuperKnockback.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for tag in MODES {
        let path = format!("SuperKnockback.Mode.{tag}");
        match choices.get(tag) {
            Some(v) => match nodes(v, tag, &path) {
                Ok(v) => {
                    let fields: Vec<(&str, fn([i32; 2]) -> S)> = match tag {
                        "SprintTap" => vec![("ReSprint", S::ReSprint)],
                        "WTap" => vec![
                            ("UntilMovementBlock", S::UntilMovementBlock),
                            ("UntilAllowedMovement", S::UntilAllowedMovement),
                        ],
                        _ => vec![],
                    };
                    for (name, make) in fields {
                        leaf(patch, &v, name, &format!("{path}.{name}"), |v, p| {
                            parse_int_pair(v, p).map(|v| SettingChange::SuperKnockback(make(v)))
                        });
                    }
                }
                Err(e) => patch.issues.push(e),
            },
            None => patch
                .issues
                .push(ConfigError::new(&path, "choice not found")),
        }
    }
}
