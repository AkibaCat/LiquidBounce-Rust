// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::reverse_step::{MODES, Mode, Settings};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReverseStepSetting {
    Mode(Mode),
    MaximumFallDistance(f32),
    Ticks(i32),
    SimulateFalling(bool),
    Motion(f32),
    Factor(f32),
}
impl ReverseStepSetting {
    pub fn apply(self, s: &mut Settings) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::MaximumFallDistance(v) => s.maximum_fall_distance = v,
            Self::Ticks(v) => s.ticks = v,
            Self::SimulateFalling(v) => s.simulate_falling = v,
            Self::Motion(v) => s.motion = v,
            Self::Factor(v) => s.factor = v,
        }
    }
}
pub fn serialize_reverse_step(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::ReverseStep, c);
    let mut choices = Map::new();
    choices.insert(
        "Instant".into(),
        t.group(
            "Instant",
            "CONFIGURABLE",
            vec![
                t.ranged(
                    "Ticks",
                    "INT",
                    json!(s.ticks),
                    json!({"from":1,"to":40}),
                    "ticks",
                    "liquidbounce.module.reverseStep.mode.instant.ticks",
                ),
                t.leaf(
                    "SimulateFalling",
                    "BOOLEAN",
                    json!(s.simulate_falling),
                    "liquidbounce.module.reverseStep.mode.instant.simulateFalling",
                    None,
                ),
            ],
        ),
    );
    for (tag, name, key, v) in [
        ("Strict", "Motion", "motion", s.motion),
        ("Accelerator", "Factor", "factor", s.factor),
    ] {
        choices.insert(
            tag.into(),
            t.group(
                tag,
                "CONFIGURABLE",
                vec![t.ranged(
                    name,
                    "FLOAT",
                    float_json(v, &format!("ReverseStep.{name}"))?,
                    json!({"from":0.1,"to":5.}),
                    "",
                    &format!(
                        "liquidbounce.module.reverseStep.mode.{}.{key}",
                        if tag == "Strict" {
                            "strict"
                        } else {
                            "accelerator"
                        }
                    ),
                )],
            ),
        );
    }
    values.push(t.mode(s.mode.tag(), choices));
    values.push(t.ranged(
        "MaximumFallDistance",
        "FLOAT",
        float_json(s.maximum_fall_distance, "ReverseStep.MaximumFallDistance")?,
        json!({"from":1.,"to":50.}),
        "",
        "liquidbounce.module.reverseStep.maximumFallDistance",
    ));
    Ok(t.group("ReverseStep", "TOGGLEABLE", values))
}
pub(super) fn parse_reverse_step(p: &mut SettingsPatch, n: &Nodes<'_>) {
    leaf(
        p,
        n,
        "MaximumFallDistance",
        "ReverseStep.MaximumFallDistance",
        |v, path| {
            parse_float(v, path)
                .map(|v| SettingChange::ReverseStep(ReverseStepSetting::MaximumFallDistance(v)))
        },
    );
    let Some(mode) = n.get("Mode") else {
        return;
    };
    match mode["active"].as_str().and_then(Mode::parse) {
        Some(v) => p
            .changes
            .push(SettingChange::ReverseStep(ReverseStepSetting::Mode(v))),
        None => p.issues.push(ConfigError::new(
            "ReverseStep.Mode.active",
            "unknown ReverseStep mode",
        )),
    }
    let Some(choices) = mode["choices"].as_object() else {
        p.issues.push(ConfigError::new(
            "ReverseStep.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for tag in MODES {
        let path = format!("ReverseStep.Mode.{tag}");
        let Some(value) = choices.get(tag) else {
            p.issues.push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        match nodes(value, tag, &path) {
            Ok(n) => match tag {
                "Instant" => {
                    leaf(p, &n, "Ticks", &format!("{path}.Ticks"), |v, path| {
                        parse_int(v, path).and_then(|v| {
                            if (1..=40).contains(&v) {
                                Ok(SettingChange::ReverseStep(ReverseStepSetting::Ticks(v)))
                            } else {
                                Err(ConfigError::new(path, "ticks must be between 1 and 40"))
                            }
                        })
                    });
                    leaf(
                        p,
                        &n,
                        "SimulateFalling",
                        &format!("{path}.SimulateFalling"),
                        |v, path| {
                            parse_bool(v, path).map(|v| {
                                SettingChange::ReverseStep(ReverseStepSetting::SimulateFalling(v))
                            })
                        },
                    );
                }
                "Strict" => leaf(p, &n, "Motion", &format!("{path}.Motion"), |v, path| {
                    parse_float(v, path)
                        .map(|v| SettingChange::ReverseStep(ReverseStepSetting::Motion(v)))
                }),
                _ => leaf(p, &n, "Factor", &format!("{path}.Factor"), |v, path| {
                    parse_float(v, path)
                        .map(|v| SettingChange::ReverseStep(ReverseStepSetting::Factor(v)))
                }),
            },
            Err(e) => p.issues.push(e),
        }
    }
}
