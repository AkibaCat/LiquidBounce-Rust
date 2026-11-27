// SPDX-License-Identifier: GPL-3.0-or-later
//! All ModuleDerp settings, including both independent named choice groups.
//! The source's hidden RotationsValueGroup is never serialized or imported.
use super::*;
use liquidbounce_behaviors::derp::{Derp, PITCH_MODES, PitchMode, YAW_MODES, YawMode};

#[derive(Clone, Debug, PartialEq)]
pub enum DerpSetting {
    YawMode(YawMode),
    Yaw(f32),
    YawOffset(f32),
    ForwardTicks(i32),
    BackwardTicks(i32),
    SpinSpeed(i32),
    PitchMode(PitchMode),
    Pitch(f32),
    PitchOffset(f32),
    SafePitch(bool),
    NotDuringSprint(bool),
}
impl DerpSetting {
    pub fn apply(self, state: &mut Derp) {
        match self {
            Self::YawMode(value) => state.set_yaw_mode(value),
            Self::Yaw(value) => state.settings.yaw = value,
            Self::YawOffset(value) => state.settings.yaw_offset = value,
            Self::ForwardTicks(value) => state.settings.forward_ticks = value,
            Self::BackwardTicks(value) => state.settings.backward_ticks = value,
            Self::SpinSpeed(value) => state.settings.spin_speed = value,
            Self::PitchMode(value) => state.settings.pitch_mode = value,
            Self::Pitch(value) => state.settings.pitch = value,
            Self::PitchOffset(value) => state.settings.pitch_offset = value,
            Self::SafePitch(value) => state.settings.safe_pitch = value,
            Self::NotDuringSprint(value) => state.settings.not_during_sprint = value,
        }
    }
}
pub fn serialize_derp(
    state: &Derp,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let settings = &state.settings;
    let angle = |name: &str, value: f32, path: &str, key: &str| -> Result<Value, ConfigError> {
        Ok(tree.ranged(
            name,
            "FLOAT",
            float_json(value, path)?,
            float_range(-180.0, 180.0, path)?,
            "°",
            key,
        ))
    };
    let ticks = |name: &str, value: i32, key: &str| {
        tree.ranged(
            name,
            "INT",
            json!(value),
            json!({"from":0,"to":100}),
            "ticks",
            key,
        )
    };
    let mut yaw = Map::new();
    yaw.insert(
        "Static".into(),
        tree.group(
            "Static",
            "CONFIGURABLE",
            vec![angle(
                "Yaw",
                settings.yaw,
                "Derp.Yaw.Static.Yaw",
                "liquidbounce.module.derp.yaw.static.yaw",
            )?],
        ),
    );
    yaw.insert(
        "Offset".into(),
        tree.group(
            "Offset",
            "CONFIGURABLE",
            vec![angle(
                "Offset",
                settings.yaw_offset,
                "Derp.Yaw.Offset.Offset",
                "liquidbounce.module.derp.yaw.offset.offset",
            )?],
        ),
    );
    yaw.insert(
        "Random".into(),
        tree.group("Random", "CONFIGURABLE", vec![]),
    );
    yaw.insert(
        "Jitter".into(),
        tree.group(
            "Jitter",
            "CONFIGURABLE",
            vec![
                ticks(
                    "ForwardTicks",
                    settings.forward_ticks,
                    "liquidbounce.module.derp.yaw.jitter.forwardTicks",
                ),
                ticks(
                    "BackwardTicks",
                    settings.backward_ticks,
                    "liquidbounce.module.derp.yaw.jitter.backwardTicks",
                ),
            ],
        ),
    );
    yaw.insert(
        "Spin".into(),
        tree.group(
            "Spin",
            "CONFIGURABLE",
            vec![tree.ranged(
                "Speed",
                "INT",
                json!(settings.spin_speed),
                json!({"from":-70,"to":70}),
                "°/tick",
                "liquidbounce.module.derp.yaw.spin.speed",
            )],
        ),
    );
    let mut yaw = tree.mode(settings.yaw_mode.tag(), yaw);
    yaw["name"] = json!("Yaw");
    let mut pitch = Map::new();
    pitch.insert(
        "Static".into(),
        tree.group(
            "Static",
            "CONFIGURABLE",
            vec![angle(
                "Pitch",
                settings.pitch,
                "Derp.Pitch.Static.Pitch",
                "liquidbounce.module.derp.pitch.static.pitch",
            )?],
        ),
    );
    pitch.insert(
        "Offset".into(),
        tree.group(
            "Offset",
            "CONFIGURABLE",
            vec![angle(
                "Offset",
                settings.pitch_offset,
                "Derp.Pitch.Offset.Offset",
                "liquidbounce.module.derp.pitch.offset.offset",
            )?],
        ),
    );
    pitch.insert(
        "Random".into(),
        tree.group("Random", "CONFIGURABLE", vec![]),
    );
    let mut pitch = tree.mode(settings.pitch_mode.tag(), pitch);
    pitch["name"] = json!("Pitch");
    let mut values = tree.common(ModuleKind::Derp, common);
    values.extend([
        yaw,
        pitch,
        tree.leaf(
            "SafePitch",
            "BOOLEAN",
            json!(settings.safe_pitch),
            "liquidbounce.module.derp.safePitch",
            None,
        ),
        tree.leaf(
            "NotDuringSprint",
            "BOOLEAN",
            json!(settings.not_during_sprint),
            "liquidbounce.module.derp.notDuringSprint",
            None,
        ),
    ]);
    Ok(tree.group("Derp", "TOGGLEABLE", values))
}
fn change(value: DerpSetting) -> SettingChange {
    SettingChange::Derp(value)
}
fn parse_yaw(patch: &mut SettingsPatch, mode: &Value) {
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|tag| YawMode::parse(&tag))
    {
        Some(value) => patch.changes.push(change(DerpSetting::YawMode(value))),
        None => patch.issues.push(ConfigError::new(
            "Derp.Yaw.active",
            "missing or unknown case-sensitive mode",
        )),
    }
    if let Err(error) = nodes(mode, "Yaw", "Derp.Yaw") {
        patch.issues.push(error);
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch.issues.push(ConfigError::new(
            "Derp.Yaw.choices",
            "expected choices object",
        ));
        return;
    };
    for name in YAW_MODES {
        let path = format!("Derp.Yaw.{name}");
        match choices.get(name) {
            Some(choice) => match nodes(choice, name, &path) {
                Ok(choice) => match name {
                    "Static" => leaf(patch, &choice, "Yaw", &format!("{path}.Yaw"), |v, p| {
                        parse_float(v, p).map(|v| change(DerpSetting::Yaw(v)))
                    }),
                    "Offset" => leaf(
                        patch,
                        &choice,
                        "Offset",
                        &format!("{path}.Offset"),
                        |v, p| parse_float(v, p).map(|v| change(DerpSetting::YawOffset(v))),
                    ),
                    "Jitter" => {
                        leaf(
                            patch,
                            &choice,
                            "ForwardTicks",
                            &format!("{path}.ForwardTicks"),
                            |v, p| parse_int(v, p).map(|v| change(DerpSetting::ForwardTicks(v))),
                        );
                        leaf(
                            patch,
                            &choice,
                            "BackwardTicks",
                            &format!("{path}.BackwardTicks"),
                            |v, p| parse_int(v, p).map(|v| change(DerpSetting::BackwardTicks(v))),
                        );
                    }
                    "Spin" => leaf(patch, &choice, "Speed", &format!("{path}.Speed"), |v, p| {
                        parse_int(v, p).map(|v| change(DerpSetting::SpinSpeed(v)))
                    }),
                    _ => {}
                },
                Err(error) => patch.issues.push(error),
            },
            None => patch
                .issues
                .push(ConfigError::new(&path, "choice not found")),
        }
    }
}
fn parse_pitch(patch: &mut SettingsPatch, mode: &Value) {
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|tag| PitchMode::parse(&tag))
    {
        Some(value) => patch.changes.push(change(DerpSetting::PitchMode(value))),
        None => patch.issues.push(ConfigError::new(
            "Derp.Pitch.active",
            "missing or unknown case-sensitive mode",
        )),
    }
    if let Err(error) = nodes(mode, "Pitch", "Derp.Pitch") {
        patch.issues.push(error);
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch.issues.push(ConfigError::new(
            "Derp.Pitch.choices",
            "expected choices object",
        ));
        return;
    };
    for name in PITCH_MODES {
        let path = format!("Derp.Pitch.{name}");
        match choices.get(name) {
            Some(choice) => match nodes(choice, name, &path) {
                Ok(choice) => match name {
                    "Static" => leaf(patch, &choice, "Pitch", &format!("{path}.Pitch"), |v, p| {
                        parse_float(v, p).map(|v| change(DerpSetting::Pitch(v)))
                    }),
                    "Offset" => leaf(
                        patch,
                        &choice,
                        "Offset",
                        &format!("{path}.Offset"),
                        |v, p| parse_float(v, p).map(|v| change(DerpSetting::PitchOffset(v))),
                    ),
                    _ => {}
                },
                Err(error) => patch.issues.push(error),
            },
            None => patch
                .issues
                .push(ConfigError::new(&path, "choice not found")),
        }
    }
}
pub(super) fn parse_derp(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(yaw) = root.get("Yaw") {
        parse_yaw(patch, yaw);
    }
    if let Some(pitch) = root.get("Pitch") {
        parse_pitch(patch, pitch);
    }
    leaf(patch, root, "SafePitch", "Derp.SafePitch", |v, p| {
        parse_bool(v, p).map(|v| change(DerpSetting::SafePitch(v)))
    });
    leaf(
        patch,
        root,
        "NotDuringSprint",
        "Derp.NotDuringSprint",
        |v, p| parse_bool(v, p).map(|v| change(DerpSetting::NotDuringSprint(v))),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        }
    }
    #[test]
    fn derp_all_modes_roundtrip_all_leaves_and_never_expose_hidden_rotations() {
        for yaw in [
            YawMode::Static,
            YawMode::Offset,
            YawMode::Random,
            YawMode::Jitter,
            YawMode::Spin,
        ] {
            for pitch in [PitchMode::Static, PitchMode::Offset, PitchMode::Random] {
                let mut state = Derp::default();
                state.settings.yaw_mode = yaw;
                state.settings.pitch_mode = pitch;
                let tree =
                    serialize_derp(&state, &common(), JsonFormat::Interop, &str::to_owned).unwrap();
                assert_eq!(
                    tree["value"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v["name"].as_str().unwrap())
                        .collect::<Vec<_>>(),
                    [
                        "Bind",
                        "Hidden",
                        "Yaw",
                        "Pitch",
                        "SafePitch",
                        "NotDuringSprint"
                    ]
                );
                assert_eq!(tree["value"][2]["active"], yaw.tag());
                assert_eq!(tree["value"][3]["active"], pitch.tag());
                assert_eq!(
                    tree["value"][2]["choices"]["Static"]["value"][0]["suffix"],
                    "°"
                );
                assert_eq!(
                    tree["value"][2]["choices"]["Jitter"]["value"][0]["range"],
                    json!({"from":0,"to":100})
                );
                assert_eq!(
                    tree["value"][2]["choices"]["Spin"]["value"][0]["range"],
                    json!({"from":-70,"to":70})
                );
                assert_eq!(
                    tree["value"][2]["choices"]["Spin"]["value"][0]["suffix"],
                    "°/tick"
                );
                assert_eq!(
                    tree["value"][3]["choices"]["Static"]["value"][0]["value"],
                    -90.0
                );
                assert!(!tree.to_string().contains("Rotations"));
                let stored =
                    serialize_derp(&state, &common(), JsonFormat::File, &str::to_owned).unwrap();
                let parsed = parse_settings(ModuleKind::Derp, &stored).unwrap();
                assert!(parsed.issues.is_empty(), "{:?}", parsed.issues);
                let mut target = Derp::default();
                for change_value in parsed.changes {
                    if let SettingChange::Derp(value) = change_value {
                        value.apply(&mut target);
                    }
                }
                assert_eq!(state.settings, target.settings);
            }
        }
    }
    #[test]
    fn derp_raw_imports_errors_continue_and_only_changed_yaw_cancels() {
        fn child<'a>(tree: &'a mut Value, name: &str) -> &'a mut Value {
            tree["value"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|v| v["name"] == name)
                .unwrap()
        }
        let mut state = Derp::default();
        state.set_yaw_mode(YawMode::Spin);
        state.start_yaw_handler(true, Default::default());
        DerpSetting::YawMode(YawMode::Spin).apply(&mut state);
        assert!(state.busy());
        DerpSetting::PitchMode(PitchMode::Offset).apply(&mut state);
        assert!(state.busy());
        DerpSetting::YawMode(YawMode::Static).apply(&mut state);
        assert!(!state.busy());
        assert_eq!(state.spin_yaw(), 50.0);
        let mut stored =
            serialize_derp(&state, &common(), JsonFormat::File, &str::to_owned).unwrap();
        let yaw = child(&mut stored, "Yaw");
        yaw["active"] = json!("spin");
        yaw["choices"]["Static"]["value"][0]["value"] = json!("NaN");
        yaw["choices"]["Jitter"]["value"][0]["value"] = json!(-1000);
        yaw["choices"]["Jitter"]["value"][1]["value"] = json!(4294967297_i64);
        yaw["choices"]["Spin"]["value"][0]["value"] = json!(9999);
        child(&mut stored, "Pitch")["choices"]["Static"]["value"][0]["value"] = json!(-777.25);
        child(&mut stored, "SafePitch")["value"] = json!(false);
        child(&mut stored, "NotDuringSprint")["value"] = json!(false);
        let parsed = parse_settings(ModuleKind::Derp, &stored).unwrap();
        assert_eq!(parsed.issues.len(), 1);
        for change_value in parsed.changes {
            if let SettingChange::Derp(value) = change_value {
                value.apply(&mut state);
            }
        }
        assert!(state.settings.yaw.is_nan());
        assert_eq!(state.settings.forward_ticks, -1000);
        assert_eq!(state.settings.backward_ticks, 1);
        assert_eq!(state.settings.spin_speed, 9999);
        assert_eq!(state.settings.pitch, -777.25);
        assert!(!state.settings.safe_pitch);
        assert!(!state.settings.not_during_sprint);
        assert!(serialize_derp(&state, &common(), JsonFormat::File, &str::to_owned).is_err());
        assert!(matches!(
            apply_setting(
                &SettingChange::Derp(DerpSetting::SafePitch(false)),
                &mut NoFov::default(),
                &mut Zoom::default(),
                &mut FullBright::default()
            ),
            ApplyResult::Runtime(RuntimeRequest::Derp(DerpSetting::SafePitch(false)))
        ));
    }
}
