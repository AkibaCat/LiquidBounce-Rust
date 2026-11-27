// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::liquid_walk::{MODES, Mode, Settings};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LiquidWalkSetting {
    Mode(Mode),
    Motion(f32),
}
impl LiquidWalkSetting {
    pub fn apply(self, s: &mut Settings) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::Motion(v) => s.motion = v,
        }
    }
}
pub fn serialize_liquid_walk(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut v = t.common(ModuleKind::LiquidWalk, c);
    let mut choices = Map::new();
    for tag in MODES {
        choices.insert(
            tag.into(),
            t.group(
                tag,
                "CONFIGURABLE",
                if tag == "Vulcan-2.9.1" {
                    vec![t.ranged(
                        "Motion",
                        "FLOAT",
                        float_json(s.motion, "LiquidWalk.Motion")?,
                        json!({"from":0.2,"to":1.4}),
                        "",
                        "liquidbounce.module.liquidWalk.mode.vulcan-2.9.1.motion",
                    )]
                } else {
                    vec![]
                },
            ),
        );
    }
    v.push(t.mode(s.mode.tag(), choices));
    Ok(t.group("LiquidWalk", "TOGGLEABLE", v))
}
pub(super) fn parse_liquid_walk(p: &mut SettingsPatch, n: &Nodes<'_>) {
    let Some(mode) = n.get("Mode") else {
        return;
    };
    match mode["active"].as_str().and_then(Mode::parse) {
        Some(v) => p
            .changes
            .push(SettingChange::LiquidWalk(LiquidWalkSetting::Mode(v))),
        None => p.issues.push(ConfigError::new(
            "LiquidWalk.Mode.active",
            "unknown LiquidWalk mode",
        )),
    }
    let Some(choices) = mode["choices"].as_object() else {
        p.issues.push(ConfigError::new(
            "LiquidWalk.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for tag in MODES {
        let path = format!("LiquidWalk.Mode.{tag}");
        let Some(value) = choices.get(tag) else {
            p.issues.push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        match nodes(value, tag, &path) {
            Ok(n) if tag == "Vulcan-2.9.1" => {
                leaf(p, &n, "Motion", &format!("{path}.Motion"), |v, path| {
                    parse_float(v, path)
                        .map(|v| SettingChange::LiquidWalk(LiquidWalkSetting::Motion(v)))
                })
            }
            Ok(_) => {}
            Err(e) => p.issues.push(e),
        }
    }
}
