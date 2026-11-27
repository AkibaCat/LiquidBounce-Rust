// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNoRotateSet and its noncombat RotationsValueGroup, ae0eb396.
use super::*;
use liquidbounce_behaviors::no_rotate_set::{FloatRange, NoRotateMode, NoRotateSet, SmoothMode};
use liquidbounce_behaviors::rotations::MovementCorrection;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoRotateSetSetting {
    Mode(NoRotateMode),
    AngleSmooth(SmoothMode),
    MovementCorrection(MovementCorrection),
    ResetThreshold(f32),
    TicksUntilReset(i32),
    HorizontalTurnSpeed(SmoothMode, FloatRange),
    VerticalTurnSpeed(SmoothMode, FloatRange),
    Steepness(SmoothMode, f32),
    Midpoint(SmoothMode, f32),
    YawAcceleration(FloatRange),
    PitchAcceleration(FloatRange),
    DynamicAccel(bool),
    CoefDistance(f32),
    YawCrosshairAccel(FloatRange),
    PitchCrosshairAccel(FloatRange),
    AccelerationError(bool),
    YawAccelError(f32),
    PitchAccelError(f32),
    ConstantError(bool),
    YawConstantError(f32),
    PitchConstantError(f32),
    SigmoidDeceleration(bool),
    DecelerationSteepness(f32),
    DecelerationMidpoint(f32),
}

impl NoRotateSetSetting {
    pub fn apply(self, state: &mut NoRotateSet) {
        let settings = &mut state.rotations;
        match self {
            Self::Mode(value) => state.mode = value,
            Self::AngleSmooth(value) => settings.active = value,
            Self::MovementCorrection(value) => settings.movement_correction = value,
            Self::ResetThreshold(value) => settings.reset_threshold = value,
            Self::TicksUntilReset(value) => settings.ticks_until_reset = value,
            Self::HorizontalTurnSpeed(mode, value) => match mode {
                SmoothMode::Linear => settings.linear.horizontal_turn_speed = value,
                _ => settings.sigmoid.speeds.horizontal_turn_speed = value,
            },
            Self::VerticalTurnSpeed(mode, value) => match mode {
                SmoothMode::Linear => settings.linear.vertical_turn_speed = value,
                _ => settings.sigmoid.speeds.vertical_turn_speed = value,
            },
            Self::Steepness(_, value) => settings.sigmoid.steepness = value,
            Self::Midpoint(_, value) => settings.sigmoid.midpoint = value,
            Self::YawAcceleration(value) => settings.acceleration.yaw_acceleration = value,
            Self::PitchAcceleration(value) => settings.acceleration.pitch_acceleration = value,
            Self::DynamicAccel(value) => settings.acceleration.dynamic_accel = value,
            Self::CoefDistance(value) => settings.acceleration.coef_distance = value,
            Self::YawCrosshairAccel(value) => settings.acceleration.yaw_crosshair_accel = value,
            Self::PitchCrosshairAccel(value) => settings.acceleration.pitch_crosshair_accel = value,
            Self::AccelerationError(value) => settings.acceleration.acceleration_error = value,
            Self::YawAccelError(value) => settings.acceleration.yaw_accel_error = value,
            Self::PitchAccelError(value) => settings.acceleration.pitch_accel_error = value,
            Self::ConstantError(value) => settings.acceleration.constant_error = value,
            Self::YawConstantError(value) => settings.acceleration.yaw_constant_error = value,
            Self::PitchConstantError(value) => settings.acceleration.pitch_constant_error = value,
            Self::SigmoidDeceleration(value) => settings.acceleration.sigmoid_deceleration = value,
            Self::DecelerationSteepness(value) => settings.acceleration.steepness = value,
            Self::DecelerationMidpoint(value) => settings.acceleration.midpoint = value,
        }
    }
}

fn smooth_tag(mode: SmoothMode) -> &'static str {
    match mode {
        SmoothMode::Linear => "Linear",
        SmoothMode::Sigmoid => "Sigmoid",
        SmoothMode::Acceleration => "Acceleration",
    }
}
fn correction_tag(value: MovementCorrection) -> &'static str {
    match value {
        MovementCorrection::Off => "Off",
        MovementCorrection::Strict => "Strict",
        MovementCorrection::Silent => "Silent",
        MovementCorrection::ChangeLook => "ChangeLook",
    }
}

pub fn serialize_no_rotate_set(
    state: &NoRotateSet,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let base = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
    let key = |section: &str, name: &str| {
        let mut chars = name.chars();
        let leaf = format!("{}{}", chars.next().unwrap().to_lowercase(), chars.as_str());
        if section.is_empty() {
            format!("{base}.{leaf}")
        } else {
            format!("{base}.{section}.{leaf}")
        }
    };
    let float =
        |section: &str, name: &str, value: f32, from: f32, to: f32| -> Result<Value, ConfigError> {
            let key = key(section, name);
            Ok(tree.ranged(
                name,
                "FLOAT",
                float_json(value, &key)?,
                float_range(from, to, &key)?,
                "",
                &key,
            ))
        };
    let range = |section: &str,
                 name: &str,
                 value: FloatRange,
                 from: f32,
                 to: f32|
     -> Result<Value, ConfigError> {
        let key = key(section, name);
        Ok(tree.ranged(
            name,
            "FLOAT_RANGE",
            float_range(value.0, value.1, &key)?,
            float_range(from, to, &key)?,
            "",
            &key,
        ))
    };
    let boolean = |section: &str, name: &str, value: bool| {
        tree.leaf(name, "BOOLEAN", json!(value), &key(section, name), None)
    };
    let settings = &state.rotations;
    let mut smooth_choices = Map::new();
    for mode in [SmoothMode::Linear, SmoothMode::Sigmoid] {
        let section = format!("angleSmooth.{}", smooth_tag(mode).to_ascii_lowercase());
        let speeds = match mode {
            SmoothMode::Linear => settings.linear,
            _ => settings.sigmoid.speeds,
        };
        let mut values = vec![
            range(
                &section,
                "HorizontalTurnSpeed",
                speeds.horizontal_turn_speed,
                0.0,
                180.0,
            )?,
            range(
                &section,
                "VerticalTurnSpeed",
                speeds.vertical_turn_speed,
                0.0,
                180.0,
            )?,
        ];
        if mode == SmoothMode::Sigmoid {
            values.push(float(
                &section,
                "Steepness",
                settings.sigmoid.steepness,
                0.0,
                20.0,
            )?);
            values.push(float(
                &section,
                "Midpoint",
                settings.sigmoid.midpoint,
                0.0,
                1.0,
            )?);
        }
        smooth_choices.insert(
            smooth_tag(mode).to_owned(),
            tree.group(smooth_tag(mode), "CONFIGURABLE", values),
        );
    }
    let section = "angleSmooth.acceleration";
    let accel = &settings.acceleration;
    let acceleration = vec![
        range(
            section,
            "YawAcceleration",
            accel.yaw_acceleration,
            1.0,
            180.0,
        )?,
        range(
            section,
            "PitchAcceleration",
            accel.pitch_acceleration,
            1.0,
            180.0,
        )?,
        tree.group(
            "DynamicAccel",
            "TOGGLEABLE",
            vec![
                boolean(
                    "angleSmooth.acceleration.dynamicAccel",
                    "Enabled",
                    accel.dynamic_accel,
                ),
                float(
                    "angleSmooth.acceleration.dynamicAccel",
                    "CoefDistance",
                    accel.coef_distance,
                    -2.0,
                    2.0,
                )?,
                range(
                    "angleSmooth.acceleration.dynamicAccel",
                    "YawCrosshairAccel",
                    accel.yaw_crosshair_accel,
                    1.0,
                    180.0,
                )?,
                range(
                    "angleSmooth.acceleration.dynamicAccel",
                    "PitchCrosshairAccel",
                    accel.pitch_crosshair_accel,
                    1.0,
                    180.0,
                )?,
            ],
        ),
        tree.group(
            "AccelerationError",
            "TOGGLEABLE",
            vec![
                boolean(
                    "angleSmooth.acceleration.accelerationError",
                    "Enabled",
                    accel.acceleration_error,
                ),
                float(
                    "angleSmooth.acceleration.accelerationError",
                    "YawAccelError",
                    accel.yaw_accel_error,
                    0.01,
                    1.0,
                )?,
                float(
                    "angleSmooth.acceleration.accelerationError",
                    "PitchAccelError",
                    accel.pitch_accel_error,
                    0.01,
                    1.0,
                )?,
            ],
        ),
        tree.group(
            "ConstantError",
            "TOGGLEABLE",
            vec![
                boolean(
                    "angleSmooth.acceleration.constantError",
                    "Enabled",
                    accel.constant_error,
                ),
                float(
                    "angleSmooth.acceleration.constantError",
                    "YawConstantError",
                    accel.yaw_constant_error,
                    0.01,
                    1.0,
                )?,
                float(
                    "angleSmooth.acceleration.constantError",
                    "PitchConstantError",
                    accel.pitch_constant_error,
                    0.01,
                    1.0,
                )?,
            ],
        ),
        tree.group(
            "SigmoidDeceleration",
            "TOGGLEABLE",
            vec![
                boolean(
                    "angleSmooth.acceleration.sigmoidDeceleration",
                    "Enabled",
                    accel.sigmoid_deceleration,
                ),
                float(
                    "angleSmooth.acceleration.sigmoidDeceleration",
                    "Steepness",
                    accel.steepness,
                    0.0,
                    20.0,
                )?,
                float(
                    "angleSmooth.acceleration.sigmoidDeceleration",
                    "Midpoint",
                    accel.midpoint,
                    0.0,
                    1.0,
                )?,
            ],
        ),
    ];
    smooth_choices.insert(
        "Acceleration".to_owned(),
        tree.group("Acceleration", "CONFIGURABLE", acceleration),
    );
    let mut smoothing = tree.mode(smooth_tag(settings.active), smooth_choices);
    smoothing["name"] = json!("AngleSmooth");
    let mut correction = tree.leaf(
        "MovementCorrection",
        "CHOOSE",
        json!(correction_tag(settings.movement_correction)),
        &key("", "MovementCorrection"),
        None,
    );
    if format == JsonFormat::Interop {
        correction["choices"] = json!(["Off", "Strict", "Silent", "ChangeLook"]);
    }
    let rotations = tree.group(
        "Rotations",
        "CONFIGURABLE",
        vec![
            smoothing,
            correction,
            float("", "ResetThreshold", settings.reset_threshold, 1.0, 180.0)?,
            tree.ranged(
                "TicksUntilReset",
                "INT",
                json!(settings.ticks_until_reset),
                json!({"from":1,"to":30}),
                "ticks",
                &key("", "TicksUntilReset"),
            ),
        ],
    );
    let mut choices = Map::new();
    choices.insert(
        "SilentAccept".to_owned(),
        tree.group("SilentAccept", "CONFIGURABLE", vec![]),
    );
    choices.insert(
        "ResetRotation".to_owned(),
        tree.group("ResetRotation", "CONFIGURABLE", vec![rotations]),
    );
    let mut values = tree.common(ModuleKind::NoRotateSet, common);
    values.push(tree.mode(
        match state.mode {
            NoRotateMode::SilentAccept => "SilentAccept",
            NoRotateMode::ResetRotation => "ResetRotation",
        },
        choices,
    ));
    Ok(tree.group("NoRotateSet", "TOGGLEABLE", values))
}

fn change(value: NoRotateSetSetting) -> SettingChange {
    SettingChange::NoRotateSet(value)
}

fn parse_choice(
    patch: &mut SettingsPatch,
    mode: &Value,
    name: &str,
    names: &[&str],
    path: &str,
    active: impl FnOnce(&str) -> Option<NoRotateSetSetting>,
    mut parse: impl FnMut(&mut SettingsPatch, &str, &Nodes<'_>),
) {
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|tag| active(&tag))
    {
        Some(value) => patch.changes.push(change(value)),
        None => patch.issues.push(ConfigError::new(
            &format!("{path}.active"),
            "missing or unknown case-sensitive mode",
        )),
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch.issues.push(ConfigError::new(
            &format!("{path}.choices"),
            "expected choices object",
        ));
        return;
    };
    for name in names {
        let path = format!("{path}.{name}");
        match choices.get(*name) {
            Some(choice) => match nodes(choice, name, &path) {
                Ok(choice) => parse(patch, name, &choice),
                Err(error) => patch.issues.push(error),
            },
            None => patch
                .issues
                .push(ConfigError::new(&path, "choice not found")),
        }
    }
    if let Err(error) = nodes(mode, name, path) {
        patch.issues.push(error);
    }
}

pub(super) fn parse_no_rotate_set(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    parse_choice(
        patch,
        mode,
        "Mode",
        &["SilentAccept", "ResetRotation"],
        "NoRotateSet.Mode",
        |tag| match tag {
            "SilentAccept" => Some(NoRotateSetSetting::Mode(NoRotateMode::SilentAccept)),
            "ResetRotation" => Some(NoRotateSetSetting::Mode(NoRotateMode::ResetRotation)),
            _ => None,
        },
        |patch, name, choice| {
            if name != "ResetRotation" {
                return;
            }
            if let Some(rotations) = choice.get("Rotations") {
                match nodes(
                    rotations,
                    "Rotations",
                    "NoRotateSet.Mode.ResetRotation.Rotations",
                ) {
                    Ok(rotations) => parse_rotations(patch, &rotations),
                    Err(error) => patch.issues.push(error),
                }
            }
        },
    );
}

fn parse_rotations(patch: &mut SettingsPatch, rotations: &Nodes<'_>) {
    let base = "NoRotateSet.Mode.ResetRotation.Rotations";
    if let Some(smoothing) = rotations.get("AngleSmooth") {
        parse_choice(
            patch,
            smoothing,
            "AngleSmooth",
            &["Linear", "Sigmoid", "Acceleration"],
            &format!("{base}.AngleSmooth"),
            |tag| {
                [
                    SmoothMode::Linear,
                    SmoothMode::Sigmoid,
                    SmoothMode::Acceleration,
                ]
                .into_iter()
                .find(|mode| smooth_tag(*mode) == tag)
                .map(NoRotateSetSetting::AngleSmooth)
            },
            |patch, name, choice| {
                let mode = match name {
                    "Linear" => SmoothMode::Linear,
                    "Sigmoid" => SmoothMode::Sigmoid,
                    _ => SmoothMode::Acceleration,
                };
                if mode == SmoothMode::Acceleration {
                    parse_acceleration(patch, choice);
                    return;
                }
                for (name, horizontal) in
                    [("HorizontalTurnSpeed", true), ("VerticalTurnSpeed", false)]
                {
                    leaf(
                        patch,
                        choice,
                        name,
                        &format!("{base}.AngleSmooth.{}.{name}", smooth_tag(mode)),
                        |value, path| {
                            let [from, to] = parse_float_pair(value, path)?;
                            Ok(change(if horizontal {
                                NoRotateSetSetting::HorizontalTurnSpeed(mode, FloatRange(from, to))
                            } else {
                                NoRotateSetSetting::VerticalTurnSpeed(mode, FloatRange(from, to))
                            }))
                        },
                    );
                }
                if mode == SmoothMode::Sigmoid {
                    leaf(
                        patch,
                        choice,
                        "Steepness",
                        &format!("{base}.AngleSmooth.Sigmoid.Steepness"),
                        |value, path| {
                            parse_float(value, path)
                                .map(|value| change(NoRotateSetSetting::Steepness(mode, value)))
                        },
                    );
                    leaf(
                        patch,
                        choice,
                        "Midpoint",
                        &format!("{base}.AngleSmooth.Sigmoid.Midpoint"),
                        |value, path| {
                            parse_float(value, path)
                                .map(|value| change(NoRotateSetSetting::Midpoint(mode, value)))
                        },
                    );
                }
            },
        );
    }
    leaf(
        patch,
        rotations,
        "MovementCorrection",
        &format!("{base}.MovementCorrection"),
        |value, path| {
            let tag =
                scalar_string(value).ok_or_else(|| ConfigError::new(path, "expected enum tag"))?;
            [
                MovementCorrection::Off,
                MovementCorrection::Strict,
                MovementCorrection::Silent,
                MovementCorrection::ChangeLook,
            ]
            .into_iter()
            .find(|choice| correction_tag(*choice).eq_ignore_ascii_case(&tag))
            .map(|value| change(NoRotateSetSetting::MovementCorrection(value)))
            .ok_or_else(|| ConfigError::new(path, "unknown MovementCorrection"))
        },
    );
    leaf(
        patch,
        rotations,
        "ResetThreshold",
        &format!("{base}.ResetThreshold"),
        |value, path| {
            parse_float(value, path).map(|value| change(NoRotateSetSetting::ResetThreshold(value)))
        },
    );
    leaf(
        patch,
        rotations,
        "TicksUntilReset",
        &format!("{base}.TicksUntilReset"),
        |value, path| {
            parse_int(value, path).map(|value| change(NoRotateSetSetting::TicksUntilReset(value)))
        },
    );
}

fn parse_acceleration(patch: &mut SettingsPatch, choice: &Nodes<'_>) {
    use NoRotateSetSetting as S;
    let base = "NoRotateSet.Mode.ResetRotation.Rotations.AngleSmooth.Acceleration";
    for (name, make) in [
        ("YawAcceleration", S::YawAcceleration as fn(FloatRange) -> S),
        ("PitchAcceleration", S::PitchAcceleration),
    ] {
        let mut aliased = choice.clone();
        if name == "PitchAcceleration"
            && !aliased.contains_key(name)
            && let Some(value) = choice.get("PitchAccelelation")
        {
            aliased.insert(name.to_owned(), value);
        }
        leaf(
            patch,
            &aliased,
            name,
            &format!("{base}.{name}"),
            |value, path| {
                let [from, to] = parse_float_pair(value, path)?;
                Ok(change(make(FloatRange(from, to))))
            },
        );
    }
    for (name, make) in [
        ("DynamicAccel", S::DynamicAccel as fn(bool) -> S),
        ("AccelerationError", S::AccelerationError),
        ("ConstantError", S::ConstantError),
        ("SigmoidDeceleration", S::SigmoidDeceleration),
    ] {
        let Some(group) = choice.get(name) else {
            continue;
        };
        let path = format!("{base}.{name}");
        let group = match nodes(group, name, &path) {
            Ok(group) => group,
            Err(error) => {
                patch.issues.push(error);
                continue;
            }
        };
        leaf(
            patch,
            &group,
            "Enabled",
            &format!("{path}.Enabled"),
            |value, path| parse_bool(value, path).map(|value| change(make(value))),
        );
        type FloatSetting = (&'static str, fn(f32) -> S);
        let floats: &[FloatSetting] = match name {
            "DynamicAccel" => &[("CoefDistance", S::CoefDistance)],
            "AccelerationError" => &[
                ("YawAccelError", S::YawAccelError),
                ("PitchAccelError", S::PitchAccelError),
            ],
            "ConstantError" => &[
                ("YawConstantError", S::YawConstantError),
                ("PitchConstantError", S::PitchConstantError),
            ],
            _ => &[
                ("Steepness", S::DecelerationSteepness),
                ("Midpoint", S::DecelerationMidpoint),
            ],
        };
        for (name, make) in floats {
            leaf(
                patch,
                &group,
                name,
                &format!("{path}.{name}"),
                |value, path| parse_float(value, path).map(|value| change(make(value))),
            );
        }
        if name == "DynamicAccel" {
            for (name, make) in [
                (
                    "YawCrosshairAccel",
                    S::YawCrosshairAccel as fn(FloatRange) -> S,
                ),
                ("PitchCrosshairAccel", S::PitchCrosshairAccel),
            ] {
                leaf(
                    patch,
                    &group,
                    name,
                    &format!("{path}.{name}"),
                    |value, path| {
                        let [from, to] = parse_float_pair(value, path)?;
                        Ok(change(make(FloatRange(from, to))))
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        }
    }
    fn child<'a>(value: &'a Value, name: &str) -> &'a Value {
        value["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|value| value["name"] == name)
            .unwrap()
    }
    fn child_mut<'a>(value: &'a mut Value, name: &str) -> &'a mut Value {
        value["value"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|value| value["name"] == name)
            .unwrap()
    }
    #[test]
    fn no_rotate_set_full_tree_order_defaults_and_raw_roundtrip() {
        let mut state = NoRotateSet {
            mode: NoRotateMode::ResetRotation,
            ..Default::default()
        };
        state.rotations.active = SmoothMode::Acceleration;
        state.rotations.linear.horizontal_turn_speed = FloatRange(900.0, -20.0);
        state.rotations.sigmoid.midpoint = 20.0;
        state.rotations.acceleration.pitch_acceleration = FloatRange(-50.0, -60.0);
        state.rotations.acceleration.coef_distance = 50.0;
        state.rotations.acceleration.yaw_accel_error = 5.0;
        state.rotations.acceleration.pitch_constant_error = -10.0;
        state.rotations.acceleration.sigmoid_deceleration = true;
        state.rotations.acceleration.steepness = -30.0;
        state.rotations.reset_threshold = -20.0;
        state.rotations.ticks_until_reset = -100;
        state.rotations.movement_correction = MovementCorrection::ChangeLook;
        let file =
            serialize_no_rotate_set(&state, &common(), JsonFormat::File, &str::to_owned).unwrap();
        let patch = parse_settings(ModuleKind::NoRotateSet, &file).unwrap();
        assert!(patch.issues.is_empty(), "{:?}", patch.issues);
        let mut restored = NoRotateSet::default();
        for change in patch.changes {
            if let SettingChange::NoRotateSet(setting) = change {
                setting.apply(&mut restored);
            }
        }
        assert_eq!(restored, state);
        let interop = serialize_no_rotate_set(
            &NoRotateSet::default(),
            &common(),
            JsonFormat::Interop,
            &str::to_owned,
        )
        .unwrap();
        let top: Vec<_> = interop["value"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value["name"].as_str().unwrap())
            .collect();
        assert_eq!(top, ["Bind", "Hidden", "Mode"]);
        let rotations = child(
            &child(&interop, "Mode")["choices"]["ResetRotation"],
            "Rotations",
        );
        let names: Vec<_> = rotations["value"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "AngleSmooth",
                "MovementCorrection",
                "ResetThreshold",
                "TicksUntilReset"
            ]
        );
        let smoothing = child(rotations, "AngleSmooth");
        assert_eq!(smoothing["active"], "Linear");
        assert_eq!(smoothing["choices"].as_object().unwrap().len(), 3);
        assert_eq!(
            child(&smoothing["choices"]["Linear"], "HorizontalTurnSpeed")["key"],
            "liquidbounce.module.noRotateSet.mode.resetRotation.rotations.angleSmooth.linear.horizontalTurnSpeed"
        );
        assert_eq!(child(rotations, "TicksUntilReset")["suffix"], "ticks");
        assert_eq!(
            child(&smoothing["choices"]["Acceleration"], "PitchAcceleration")["range"],
            json!({"from":1.0,"to":180.0})
        );
    }

    #[test]
    fn no_rotate_set_alias_modes_and_partial_import_follow_source() {
        let mut file = serialize_no_rotate_set(
            &NoRotateSet::default(),
            &common(),
            JsonFormat::File,
            &str::to_owned,
        )
        .unwrap();
        let mode = child_mut(&mut file, "Mode");
        mode["active"] = json!("resetrotation");
        let rotations = child_mut(&mut mode["choices"]["ResetRotation"], "Rotations");
        child_mut(rotations, "MovementCorrection")["value"] = json!("changelook");
        child_mut(rotations, "TicksUntilReset")["value"] = json!("-900");
        let smoothing = child_mut(rotations, "AngleSmooth");
        smoothing["active"] = json!("Sigmoid");
        let acceleration = &mut smoothing["choices"]["Acceleration"];
        let pitch = child_mut(acceleration, "PitchAcceleration");
        pitch["name"] = json!("PitchAccelelation");
        pitch["value"] = json!({"from":"-40","to":"-50"});
        let patch = parse_settings(ModuleKind::NoRotateSet, &file).unwrap();
        assert_eq!(patch.issues.len(), 1, "{:?}", patch.issues);
        let mut restored = NoRotateSet::default();
        for change in patch.changes {
            if let SettingChange::NoRotateSet(setting) = change {
                setting.apply(&mut restored);
            }
        }
        assert_eq!(restored.mode, NoRotateMode::SilentAccept);
        assert_eq!(restored.rotations.active, SmoothMode::Sigmoid);
        assert_eq!(
            restored.rotations.movement_correction,
            MovementCorrection::ChangeLook
        );
        assert_eq!(restored.rotations.ticks_until_reset, -900);
        assert_eq!(
            restored.rotations.acceleration.pitch_acceleration,
            FloatRange(-40.0, -50.0)
        );
        restored.rotations.sigmoid.midpoint = f32::NAN;
        assert!(
            serialize_no_rotate_set(&restored, &common(), JsonFormat::Interop, &str::to_owned)
                .is_err()
        );
    }
}
