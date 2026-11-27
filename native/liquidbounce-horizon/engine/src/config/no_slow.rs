// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::no_slow::{self as model, Mode, NoSlow, TREES, Timing};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoSlowSetting {
    Enabled(usize, bool),
    Float(usize, f32),
    Mode(usize, Mode),
    Flag(usize, bool),
    Timing(usize, Timing),
    IntaveNew(bool),
}
impl NoSlowSetting {
    pub fn apply(self, s: &mut NoSlow) {
        match self {
            Self::Enabled(g, v) => {
                if g < 4 {
                    s.settings.uses[g].enabled = v;
                } else {
                    s.settings.enabled[g - 4] = v;
                }
            }
            Self::Float(g, v) => match g {
                0..=3 => s.settings.uses[g].forward = v,
                4..=7 => s.settings.uses[g - 4].sideways = v,
                8..=12 => s.settings.multipliers[g - 8] = v,
                _ => s.slowness = v,
            },
            Self::Mode(g, v) => {
                if g == 4 {
                    s.settings.sneak_mode = v
                } else {
                    s.settings.uses[g].mode = v
                }
            }
            Self::Flag(g, v) => match g {
                0 => s.settings.only_slow_server = v,
                1..=2 => s.settings.no_block_interact[g - 1] = v,
                _ => s.settings.fluid_collision = v,
            },
            Self::Timing(g, v) => match g {
                0 => s.settings.reuse_timing = v,
                1 => s.settings.switch_timing = v,
                2 => s.settings.sneak_switch_timing = v,
                _ => s.settings.sneak_aac_timing = v,
            },
            Self::IntaveNew(v) => s.settings.intave_new = v,
        }
    }
}
fn choose(t: &Tree<'_>, name: &str, value: &str, choices: &[&str], key: &str) -> Value {
    let mut v = t.leaf(name, "CHOOSE", json!(value), key, None);
    if t.format == JsonFormat::Interop {
        v["choices"] = json!(choices);
    }
    v
}
fn group_key(g: usize) -> String {
    let mut s = TREES[g].to_string();
    s[..1].make_ascii_lowercase();
    format!("liquidbounce.module.noSlow.{s}")
}
pub fn serialize_no_slow(
    s: &NoSlow,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::NoSlow, c);
    for (g, name) in TREES.into_iter().enumerate() {
        let key = group_key(g);
        let enabled = if g < 4 {
            s.settings.uses[g].enabled
        } else {
            s.settings.enabled[g - 4]
        };
        let mut v = vec![t.leaf(
            "Enabled",
            "BOOLEAN",
            json!(enabled),
            &format!("{key}.enabled"),
            None,
        )];
        if g < 4 {
            for (label, k, value) in [
                ("Forward", "forward", s.settings.uses[g].forward),
                ("Sideways", "sideways", s.settings.uses[g].sideways),
            ] {
                v.push(t.ranged(
                    label,
                    "FLOAT",
                    float_json(value, &format!("NoSlow.{name}.{label}"))?,
                    json!({"from":0.,"to":1.}),
                    "",
                    &format!("{key}.{k}"),
                ));
            }
        }
        if g == 0 {
            v.push(t.leaf(
                "OnlySlowOnServerSide",
                "BOOLEAN",
                json!(s.settings.only_slow_server),
                &format!("{key}.onlySlowOnServerSide"),
                None,
            ));
        }
        if matches!(g, 1 | 2) {
            v.push(t.group(
                "NoBlockInteract",
                "TOGGLEABLE",
                vec![t.leaf(
                    "Enabled",
                    "BOOLEAN",
                    json!(s.settings.no_block_interact[g - 1]),
                    &format!("{key}.noBlockInteract.enabled"),
                    None,
                )],
            ));
        }
        if g == 4 {
            v.push(t.ranged(
                "MinMultiplier",
                "FLOAT",
                float_json(s.settings.multipliers[0], "NoSlow.Sneaking.MinMultiplier")?,
                json!({"from":0.3,"to":1.}),
                "",
                &format!("{key}.minMultiplier"),
            ));
        }
        if !model::modes(g).is_empty() {
            let mut choices = Map::new();
            for mode in model::modes(g) {
                let tag = mode.tag();
                let mut leaves = Vec::new();
                let timing = match (g, mode) {
                    (0, Mode::Reuse) => Some(s.settings.reuse_timing),
                    (0, Mode::Switch) => Some(s.settings.switch_timing),
                    (4, Mode::Switch) => Some(s.settings.sneak_switch_timing),
                    (4, Mode::Aac5) => Some(s.settings.sneak_aac_timing),
                    _ => None,
                };
                let modekey = format!("{key}.mode.{}", tag.to_ascii_lowercase());
                if let Some(timing) = timing {
                    leaves.push(choose(
                        &t,
                        "Timing",
                        timing.tag(),
                        &["PreAndPost", "Pre", "Post"],
                        &format!("{modekey}.timing"),
                    ));
                }
                if g == 1 && *mode == Mode::Intave14 {
                    leaves.push(choose(
                        &t,
                        "Mode",
                        if s.settings.intave_new {
                            "New"
                        } else {
                            "Release"
                        },
                        &["Release", "New"],
                        &format!("{modekey}.mode"),
                    ));
                }
                choices.insert(tag.into(), t.group(tag, "CONFIGURABLE", leaves));
            }
            let active = if g == 4 {
                s.settings.sneak_mode
            } else {
                s.settings.uses[g].mode
            };
            v.push(t.mode(active.tag(), choices));
        }
        if (5..=8).contains(&g) {
            v.push(t.ranged(
                "Multiplier",
                "FLOAT",
                float_json(
                    s.settings.multipliers[g - 4],
                    &format!("NoSlow.{name}.Multiplier"),
                )?,
                json!({"from":0.4,"to":2.}),
                "",
                &format!("{key}.multiplier"),
            ));
        }
        if g == 9 {
            v.push(t.leaf(
                "Collision",
                "BOOLEAN",
                json!(s.settings.fluid_collision),
                &format!("{key}.collision"),
                None,
            ));
        }
        if g == 10 {
            v.push(t.ranged(
                "PerLevelMultiplier",
                "FLOAT",
                float_json(s.slowness, "NoSlow.Slowness.PerLevelMultiplier")?,
                json!({"from":0.,"to":0.15}),
                "",
                &format!("{key}.perLevelMultiplier"),
            ));
        }
        values.push(t.group(name, "TOGGLEABLE", v));
    }
    Ok(t.group("NoSlow", "TOGGLEABLE", values))
}
fn boolean(
    p: &mut SettingsPatch,
    n: &Nodes<'_>,
    name: &str,
    path: &str,
    change: impl Fn(bool) -> NoSlowSetting,
) {
    leaf(p, n, name, path, |v, path| {
        parse_bool(v, path).map(|v| SettingChange::NoSlow(change(v)))
    });
}
fn float(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str, index: usize) {
    leaf(p, n, name, path, |v, path| {
        parse_float(v, path).map(|v| SettingChange::NoSlow(NoSlowSetting::Float(index, v)))
    });
}
pub(super) fn parse_no_slow(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for (g, name) in TREES.into_iter().enumerate() {
        let path = format!("NoSlow.{name}");
        let Some(v) = root.get(name) else {
            continue;
        };
        let n = match nodes(v, name, &path) {
            Ok(n) => n,
            Err(e) => {
                p.issues.push(e);
                continue;
            }
        };
        boolean(p, &n, "Enabled", &format!("{path}.Enabled"), |v| {
            NoSlowSetting::Enabled(g, v)
        });
        if g < 4 {
            float(p, &n, "Forward", &format!("{path}.Forward"), g);
            float(p, &n, "Sideways", &format!("{path}.Sideways"), g + 4);
        }
        if g == 0 {
            boolean(
                p,
                &n,
                "OnlySlowOnServerSide",
                &format!("{path}.OnlySlowOnServerSide"),
                |v| NoSlowSetting::Flag(0, v),
            );
        }
        if matches!(g, 1 | 2)
            && let Some(v) = n.get("NoBlockInteract")
        {
            let path = format!("{path}.NoBlockInteract");
            match nodes(v, "NoBlockInteract", &path) {
                Ok(n) => boolean(p, &n, "Enabled", &format!("{path}.Enabled"), |v| {
                    NoSlowSetting::Flag(g, v)
                }),
                Err(e) => p.issues.push(e),
            }
        }
        if g == 4 {
            float(p, &n, "MinMultiplier", &format!("{path}.MinMultiplier"), 8);
        }
        if (5..=8).contains(&g) {
            float(p, &n, "Multiplier", &format!("{path}.Multiplier"), g + 4);
        }
        if g == 9 {
            boolean(p, &n, "Collision", &format!("{path}.Collision"), |v| {
                NoSlowSetting::Flag(3, v)
            });
        }
        if g == 10 {
            float(
                p,
                &n,
                "PerLevelMultiplier",
                &format!("{path}.PerLevelMultiplier"),
                13,
            );
        }
        if model::modes(g).is_empty() {
            continue;
        }
        let Some(mode) = n.get("Mode") else {
            continue;
        };
        let path = format!("{path}.Mode");
        match mode["active"]
            .as_str()
            .and_then(Mode::parse)
            .filter(|v| model::modes(g).contains(v))
        {
            Some(v) => p
                .changes
                .push(SettingChange::NoSlow(NoSlowSetting::Mode(g, v))),
            None => p.issues.push(ConfigError::new(
                &format!("{path}.active"),
                "unknown NoSlow mode",
            )),
        }
        let Some(choices) = mode["choices"].as_object() else {
            p.issues
                .push(ConfigError::new(&path, "expected choices object"));
            continue;
        };
        for mode in model::modes(g) {
            let tag = mode.tag();
            let path = format!("{path}.{tag}");
            let value = choices.get(tag).or_else(|| {
                choices
                    .iter()
                    .find(|(name, _)| Mode::parse(name) == Some(*mode))
                    .map(|(_, v)| v)
            });
            let Some(value) = value else {
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
            let timing = match (g, mode) {
                (0, Mode::Reuse) => Some(0),
                (0, Mode::Switch) => Some(1),
                (4, Mode::Switch) => Some(2),
                (4, Mode::Aac5) => Some(3),
                _ => None,
            };
            if let Some(index) = timing {
                leaf(p, &n, "Timing", &format!("{path}.Timing"), |v, path| {
                    scalar_string(v)
                        .as_deref()
                        .and_then(Timing::parse)
                        .map(|v| SettingChange::NoSlow(NoSlowSetting::Timing(index, v)))
                        .ok_or_else(|| ConfigError::new(path, "unknown network timing"))
                });
            }
            if g == 1 && *mode == Mode::Intave14 {
                leaf(
                    p,
                    &n,
                    "Mode",
                    &format!("{path}.Mode"),
                    |v, path| match scalar_string(v).as_deref() {
                        Some("Release") => {
                            Ok(SettingChange::NoSlow(NoSlowSetting::IntaveNew(false)))
                        }
                        Some("New") => Ok(SettingChange::NoSlow(NoSlowSetting::IntaveNew(true))),
                        _ => Err(ConfigError::new(path, "unknown Intave consume mode")),
                    },
                );
            }
        }
    }
}
