// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete original Timer choices and raw setting serialization.
use super::*;
use liquidbounce_behaviors::timer::{Mode, Settings};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimerSetting {
    Mode(Mode),
    ClassicSpeed(f32),
    NormalSpeed(f32),
    NormalTicks(i32),
    PulseBoostSpeed(f32),
    BoostTicks(i32),
    OnMove(bool),
    BoostSpeed(f32),
    SlowSpeed(f32),
    NormalizeCombat(bool),
    Balance([i32; 2]),
    ResetBalanceOnFlag(bool),
}
impl TimerSetting {
    pub fn apply(self, state: &mut Settings) {
        match self {
            Self::Mode(v) => state.mode = v,
            Self::ClassicSpeed(v) => state.classic_speed = v,
            Self::NormalSpeed(v) => state.normal_speed = v,
            Self::NormalTicks(v) => state.normal_ticks = v,
            Self::PulseBoostSpeed(v) => state.pulse_boost_speed = v,
            Self::BoostTicks(v) => state.boost_ticks = v,
            Self::OnMove(v) => state.on_move = v,
            Self::BoostSpeed(v) => state.boost_speed = v,
            Self::SlowSpeed(v) => state.slow_speed = v,
            Self::NormalizeCombat(v) => state.normalize_combat = v,
            Self::Balance(v) => state.balance = v,
            Self::ResetBalanceOnFlag(v) => state.reset_balance_on_flag = v,
        }
    }
}
pub fn serialize_timer(
    state: &Settings,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Timer, common);
    let float = |name: &str, value: f32, mode: &str| -> Result<Value, ConfigError> {
        Ok(tree.ranged(
            name,
            "FLOAT",
            float_json(value, &format!("Timer.Mode.{mode}.{name}"))?,
            json!({"from":0.1,"to":20.0}),
            "",
            &format!(
                "liquidbounce.module.timer.mode.{}.{}",
                lower_camel(mode),
                lower_camel(name)
            ),
        ))
    };
    let boolean = |name: &str, value: bool, mode: &str| {
        tree.leaf(
            name,
            "BOOLEAN",
            json!(value),
            &format!(
                "liquidbounce.module.timer.mode.{}.{}",
                lower_camel(mode),
                lower_camel(name)
            ),
            None,
        )
    };
    let ticks = |name: &str, value: i32| {
        tree.ranged(
            name,
            "INT",
            json!(value),
            json!({"from":1,"to":500}),
            "ticks",
            &format!("liquidbounce.module.timer.mode.pulse.{}", lower_camel(name)),
        )
    };
    let classic = vec![float("Speed", state.classic_speed, "Classic")?];
    let pulse = vec![
        float("NormalSpeed", state.normal_speed, "Pulse")?,
        ticks("NormalSpeedTicks", state.normal_ticks),
        float("BoostSpeed", state.pulse_boost_speed, "Pulse")?,
        ticks("BoostSpeedTicks", state.boost_ticks),
        boolean("OnMove", state.on_move, "Pulse"),
    ];
    let boost = vec![
        float("BoostSpeed", state.boost_speed, "Boost")?,
        float("SlowSpeed", state.slow_speed, "Boost")?,
        boolean("NormalizeDuringCombat", state.normalize_combat, "Boost"),
        tree.ranged(
            "Balance",
            "INT_RANGE",
            json!({"from":state.balance[0],"to":state.balance[1]}),
            json!({"from":-100,"to":200}),
            "ticks",
            "liquidbounce.module.timer.mode.boost.balance",
        ),
        boolean("ResetBalanceOnFlag", state.reset_balance_on_flag, "Boost"),
    ];
    values.push(
        tree.mode(
            state.mode.tag(),
            [
                (
                    "Classic".into(),
                    tree.group("Classic", "CONFIGURABLE", classic),
                ),
                ("Pulse".into(), tree.group("Pulse", "CONFIGURABLE", pulse)),
                ("Boost".into(), tree.group("Boost", "CONFIGURABLE", boost)),
            ]
            .into_iter()
            .collect(),
        ),
    );
    Ok(tree.group("Timer", "TOGGLEABLE", values))
}
fn lower_camel(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_lowercase().chain(chars).collect()
    })
}
pub(super) fn parse_timer(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    let path = "Timer.Mode";
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|v| Mode::from_tag(&v))
    {
        Some(mode) => patch
            .changes
            .push(SettingChange::Timer(TimerSetting::Mode(mode))),
        None => patch.issues.push(ConfigError::new(
            path,
            "missing or unknown case-sensitive mode",
        )),
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch
            .issues
            .push(ConfigError::new(path, "expected choices object"));
        return;
    };
    for name in ["Classic", "Pulse", "Boost"] {
        let path = format!("Timer.Mode.{name}");
        let group = match choices
            .get(name)
            .ok_or_else(|| ConfigError::new(&path, "choice not found"))
            .and_then(|node| nodes(node, name, &path))
        {
            Ok(v) => v,
            Err(e) => {
                patch.issues.push(e);
                continue;
            }
        };
        let change = |v| SettingChange::Timer(v);
        for (key, make) in match name {
            "Classic" => vec![(
                "Speed",
                TimerSetting::ClassicSpeed as fn(f32) -> TimerSetting,
            )],
            "Pulse" => vec![
                (
                    "NormalSpeed",
                    TimerSetting::NormalSpeed as fn(f32) -> TimerSetting,
                ),
                ("BoostSpeed", TimerSetting::PulseBoostSpeed),
            ],
            _ => vec![
                (
                    "BoostSpeed",
                    TimerSetting::BoostSpeed as fn(f32) -> TimerSetting,
                ),
                ("SlowSpeed", TimerSetting::SlowSpeed),
            ],
        } {
            leaf(patch, &group, key, &format!("{path}.{key}"), |v, p| {
                parse_float(v, p).map(|v| change(make(v)))
            });
        }
        if name == "Pulse" {
            for (key, make) in [
                (
                    "NormalSpeedTicks",
                    TimerSetting::NormalTicks as fn(i32) -> TimerSetting,
                ),
                ("BoostSpeedTicks", TimerSetting::BoostTicks),
            ] {
                leaf(patch, &group, key, &format!("{path}.{key}"), |v, p| {
                    parse_int(v, p).map(|v| change(make(v)))
                });
            }
            leaf(
                patch,
                &group,
                "OnMove",
                &format!("{path}.OnMove"),
                |v, p| parse_bool(v, p).map(|v| change(TimerSetting::OnMove(v))),
            );
        }
        if name == "Boost" {
            for (key, make) in [
                (
                    "NormalizeDuringCombat",
                    TimerSetting::NormalizeCombat as fn(bool) -> TimerSetting,
                ),
                ("ResetBalanceOnFlag", TimerSetting::ResetBalanceOnFlag),
            ] {
                leaf(patch, &group, key, &format!("{path}.{key}"), |v, p| {
                    parse_bool(v, p).map(|v| change(make(v)))
                });
            }
            leaf(
                patch,
                &group,
                "Balance",
                &format!("{path}.Balance"),
                |v, p| parse_int_pair(v, p).map(|v| change(TimerSetting::Balance(v))),
            );
        }
    }
    if let Err(error) = nodes(mode, "Mode", path) {
        patch.issues.push(error);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_choice_and_raw_value_roundtrips_in_both_formats() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        for mode in [Mode::Classic, Mode::Pulse, Mode::Boost] {
            for format in [JsonFormat::File, JsonFormat::Interop] {
                let settings = Settings {
                    mode,
                    classic_speed: -3.0,
                    normal_speed: 0.0,
                    normal_ticks: i32::MIN,
                    pulse_boost_speed: 25.0,
                    boost_ticks: 0,
                    on_move: true,
                    boost_speed: -0.0,
                    slow_speed: 20.5,
                    normalize_combat: false,
                    balance: [99, -5],
                    reset_balance_on_flag: false,
                };
                let tree = serialize_timer(&settings, &common, format, &str::to_owned).unwrap();
                let mode_node = tree["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|node| node["name"] == "Mode")
                    .unwrap();
                assert_eq!(mode_node["choices"].as_object().unwrap().len(), 3);
                assert_eq!(
                    mode_node["choices"]["Classic"]["value"]
                        .as_array()
                        .unwrap()
                        .len(),
                    1
                );
                assert_eq!(
                    mode_node["choices"]["Pulse"]["value"]
                        .as_array()
                        .unwrap()
                        .len(),
                    5
                );
                assert_eq!(
                    mode_node["choices"]["Boost"]["value"]
                        .as_array()
                        .unwrap()
                        .len(),
                    5
                );
                let patch = parse_settings(ModuleKind::Timer, &tree).unwrap();
                assert!(patch.issues.is_empty(), "{:?}", patch.issues);
                let mut restored = Settings::default();
                for change in patch.changes {
                    if let SettingChange::Timer(change) = change {
                        change.apply(&mut restored);
                    }
                }
                assert_eq!(restored, settings);
                assert_eq!(
                    restored.boost_speed.to_bits(),
                    settings.boost_speed.to_bits()
                );
            }
        }
    }
    #[test]
    fn raw_balance_and_partial_leaf_errors_match_parent_codecs() {
        let tree = json!({"name":"Timer","value":[{"name":"Mode","active":"boost","value":[],"choices":{
            "Classic":{"name":"Classic","value":[{"name":"Speed","value":"bad"}]},
            "Pulse":{"name":"Pulse","value":[{"name":"BoostSpeedTicks","value":4294967297_u64}]},
            "Boost":{"name":"Boost","value":[{"name":"Balance","value":{"from":20,"to":-3}},{"name":"NormalizeDuringCombat","value":false}]}
        }}]});
        let patch = parse_settings(ModuleKind::Timer, &tree).unwrap();
        assert_eq!(patch.issues.len(), 2);
        assert!(
            patch
                .changes
                .contains(&SettingChange::Timer(TimerSetting::BoostTicks(1)))
        );
        assert!(
            patch
                .changes
                .contains(&SettingChange::Timer(TimerSetting::Balance([20, -3])))
        );
        assert!(
            patch
                .changes
                .contains(&SettingChange::Timer(TimerSetting::NormalizeCombat(false)))
        );
    }
}
