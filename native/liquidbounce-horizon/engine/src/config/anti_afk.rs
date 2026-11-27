// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ae0eb396 AntiAFK tree. Noncombat RotationsValueGroup shares the
//! existing codec through a structural adapter, never through runtime state.
use super::*;
use liquidbounce_behaviors::{
    anti_afk::{AntiAfk, INTERACTIONS, MODES, Mode},
    no_rotate_set::NoRotateSet,
};

#[derive(Clone, Debug, PartialEq)]
pub enum AntiAfkSetting {
    Mode(Mode),
    Interaction(Value),
    Delay([i32; 2]),
    Rotate(bool),
    IgnoreOpenInventory(bool),
    Rotation(NoRotateSetSetting),
    RotateDelay(i32),
    Angle(f32),
    Swing(bool),
    SwingDelay(i32),
    Jump(bool),
    Move(bool),
}
impl AntiAfkSetting {
    pub fn apply(self, state: &mut AntiAfk) -> Result<(), ConfigError> {
        let settings = &mut state.settings;
        match self {
            Self::Mode(value) => {
                if settings.mode != value {
                    settings.mode = value;
                    // ModeValueGroup.setAndUpdate always removes the old
                    // mode's coroutine scope, including outside a world.
                    state.cancel_sequences();
                }
            }
            Self::Interaction(value) => {
                settings.interactions = 0;
                let mut include = |value: &Value| -> Result<(), ConfigError> {
                    fn string(value: &Value) -> Option<String> {
                        match value {
                            Value::String(v) => Some(v.clone()),
                            Value::Number(_) | Value::Bool(_) => Some(value.to_string()),
                            Value::Array(v) if v.len() == 1 => string(&v[0]),
                            _ => None,
                        }
                    }
                    let tag = string(value).ok_or_else(|| {
                        ConfigError::new(
                            "AntiAFK.Mode.RandomInteraction.Interaction",
                            "entry has no Gson string value",
                        )
                    })?;
                    // Java equalsIgnoreCase includes Unicode's one-character
                    // uppercase/lowercase fold (e.g. dotted/dotless i).
                    fn fold(c: char) -> char {
                        let mut upper = c.to_uppercase();
                        let first = upper.next().unwrap_or(c);
                        let upper = if upper.next().is_none() { first } else { c };
                        upper.to_lowercase().next().unwrap_or(upper)
                    }
                    if let Some(index) = INTERACTIONS
                        .iter()
                        .position(|choice| choice.chars().map(fold).eq(tag.chars().map(fold)))
                    {
                        settings.interactions |= 1 << index;
                    }
                    Ok(())
                };
                match &value {
                    Value::Array(values) => {
                        for value in values {
                            include(value)?;
                        }
                    }
                    Value::String(_) | Value::Number(_) | Value::Bool(_) => include(&value)?,
                    _ => {}
                }
            }
            Self::Delay(value) => settings.delay = value,
            Self::Rotate(value) => settings.rotate = value,
            Self::IgnoreOpenInventory(value) => settings.ignore_open_inventory = value,
            Self::Rotation(value) => {
                let mut proxy = NoRotateSet {
                    rotations: std::mem::take(&mut settings.rotations),
                    ..Default::default()
                };
                value.apply(&mut proxy);
                settings.rotations = proxy.rotations;
            }
            Self::RotateDelay(value) => settings.rotate_delay = value,
            Self::Angle(value) => settings.angle = value,
            Self::Swing(value) => settings.swing = value,
            Self::SwingDelay(value) => settings.swing_delay = value,
            Self::Jump(value) => settings.jump = value,
            Self::Move(value) => settings.move_forward = value,
        }
        Ok(())
    }
}
const ROTATION_KEY: &str = "liquidbounce.module.antiAFK.mode.custom.rotate.rotations";
const OLD_ROTATION_KEY: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const ROTATION_PATH: &str = "AntiAFK.Mode.Custom.Rotate.Rotations";
const OLD_ROTATION_PATH: &str = "NoRotateSet.Mode.ResetRotation.Rotations";
fn remap_error(mut error: ConfigError) -> ConfigError {
    error.path = error
        .path
        .replace(OLD_ROTATION_KEY, ROTATION_KEY)
        .replace(OLD_ROTATION_PATH, ROTATION_PATH);
    error
}

pub fn serialize_anti_afk(
    state: &AntiAfk,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let settings = &state.settings;
    let key = |suffix: &str| format!("liquidbounce.module.antiAFK.mode.{suffix}");
    let boolean = |suffix: &str, name: &str, value: bool| {
        tree.leaf(name, "BOOLEAN", json!(value), &key(suffix), None)
    };
    let delay = |suffix: &str, value: i32| {
        tree.ranged(
            "Delay",
            "INT",
            json!(value),
            json!({"from":0,"to":20}),
            "ticks",
            &key(suffix),
        )
    };
    let describe_rotations = |path: &str| describe(&path.replace(OLD_ROTATION_KEY, ROTATION_KEY));
    let proxy = NoRotateSet {
        rotations: settings.rotations.clone(),
        ..Default::default()
    };
    let mut root = serialize_no_rotate_set(&proxy, common, format, &describe_rotations)
        .map_err(remap_error)?;
    let mut rotations = root["value"]
        .as_array_mut()
        .and_then(|values| values.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|mode| mode["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|values| values.first_mut())
        .map(Value::take)
        .ok_or_else(|| ConfigError::new(ROTATION_PATH, "shared Rotations codec shape changed"))?;
    fn remap_keys(value: &mut Value) {
        match value {
            Value::Object(object) => {
                if let Some(Value::String(key)) = object.get_mut("key") {
                    *key = key.replace(OLD_ROTATION_KEY, ROTATION_KEY);
                }
                for value in object.values_mut() {
                    remap_keys(value);
                }
            }
            Value::Array(values) => {
                for value in values {
                    remap_keys(value);
                }
            }
            _ => {}
        }
    }
    remap_keys(&mut rotations);
    let custom = tree.group(
        "Custom",
        "CONFIGURABLE",
        vec![
            tree.group(
                "Rotate",
                "TOGGLEABLE",
                vec![
                    boolean("custom.rotate.enabled", "Enabled", settings.rotate),
                    boolean(
                        "custom.rotate.ignoreOpenInventory",
                        "IgnoreOpenInventory",
                        settings.ignore_open_inventory,
                    ),
                    rotations,
                    delay("custom.rotate.delay", settings.rotate_delay),
                    tree.ranged(
                        "Angle",
                        "FLOAT",
                        float_json(settings.angle, "AntiAFK.Mode.Custom.Rotate.Angle")?,
                        float_range(-180.0, 180.0, "AntiAFK.Mode.Custom.Rotate.Angle")?,
                        "",
                        &key("custom.rotate.angle"),
                    ),
                ],
            ),
            tree.group(
                "Swing",
                "TOGGLEABLE",
                vec![
                    boolean("custom.swing.enabled", "Enabled", settings.swing),
                    delay("custom.swing.delay", settings.swing_delay),
                ],
            ),
            boolean("custom.jump", "Jump", settings.jump),
            boolean("custom.move", "Move", settings.move_forward),
        ],
    );
    let mut choices = Map::new();
    choices.insert("Old".into(), tree.group("Old", "CONFIGURABLE", vec![]));
    choices.insert(
        "RandomInteraction".into(),
        tree.group(
            "RandomInteraction",
            "CONFIGURABLE",
            vec![
                tree.multi(
                    "Interaction",
                    u32::from(settings.interactions),
                    &INTERACTIONS,
                    &key("randomInteraction.interaction"),
                ),
                tree.ranged(
                    "Delay",
                    "INT_RANGE",
                    json!({"from":settings.delay[0],"to":settings.delay[1]}),
                    json!({"from":0,"to":20}),
                    "ticks",
                    &key("randomInteraction.delay"),
                ),
            ],
        ),
    );
    choices.insert("Custom".into(), custom);
    let mut values = tree.common(ModuleKind::AntiAfk, common);
    values.push(tree.mode(settings.mode.tag(), choices));
    Ok(tree.group("AntiAFK", "TOGGLEABLE", values))
}
fn change(value: AntiAfkSetting) -> SettingChange {
    SettingChange::AntiAfk(value)
}
fn parse_rotations(patch: &mut SettingsPatch, rotation: &Value) {
    // The shared codec is parsed in isolation; only rotation leaf changes are
    // translated, and every issue is remapped to the actual AntiAFK path.
    let proxy = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[rotation]}}}]});
    let root = nodes(&proxy, "NoRotateSet", "NoRotateSet").unwrap();
    let mut parsed = SettingsPatch {
        module: ModuleKind::NoRotateSet,
        changes: Vec::new(),
        issues: Vec::new(),
    };
    no_rotate_set::parse_no_rotate_set(&mut parsed, &root);
    for change_value in parsed.changes {
        if let SettingChange::NoRotateSet(value) = change_value
            && !matches!(value, NoRotateSetSetting::Mode(_))
        {
            patch.changes.push(change(AntiAfkSetting::Rotation(value)));
        }
    }
    patch
        .issues
        .extend(parsed.issues.into_iter().map(remap_error));
}
fn parse_custom(patch: &mut SettingsPatch, custom: &Nodes<'_>) {
    if let Some(rotate) = custom.get("Rotate") {
        match nodes(rotate, "Rotate", "AntiAFK.Mode.Custom.Rotate") {
            Ok(rotate) => {
                leaf(
                    patch,
                    &rotate,
                    "Enabled",
                    "AntiAFK.Mode.Custom.Rotate.Enabled",
                    |v, p| parse_bool(v, p).map(|v| change(AntiAfkSetting::Rotate(v))),
                );
                leaf(
                    patch,
                    &rotate,
                    "IgnoreOpenInventory",
                    "AntiAFK.Mode.Custom.Rotate.IgnoreOpenInventory",
                    |v, p| parse_bool(v, p).map(|v| change(AntiAfkSetting::IgnoreOpenInventory(v))),
                );
                if let Some(rotations) = rotate.get("Rotations") {
                    parse_rotations(patch, rotations);
                }
                leaf(
                    patch,
                    &rotate,
                    "Delay",
                    "AntiAFK.Mode.Custom.Rotate.Delay",
                    |v, p| parse_int(v, p).map(|v| change(AntiAfkSetting::RotateDelay(v))),
                );
                leaf(
                    patch,
                    &rotate,
                    "Angle",
                    "AntiAFK.Mode.Custom.Rotate.Angle",
                    |v, p| parse_float(v, p).map(|v| change(AntiAfkSetting::Angle(v))),
                );
            }
            Err(error) => patch.issues.push(error),
        }
    }
    if let Some(swing) = custom.get("Swing") {
        match nodes(swing, "Swing", "AntiAFK.Mode.Custom.Swing") {
            Ok(swing) => {
                leaf(
                    patch,
                    &swing,
                    "Enabled",
                    "AntiAFK.Mode.Custom.Swing.Enabled",
                    |v, p| parse_bool(v, p).map(|v| change(AntiAfkSetting::Swing(v))),
                );
                leaf(
                    patch,
                    &swing,
                    "Delay",
                    "AntiAFK.Mode.Custom.Swing.Delay",
                    |v, p| parse_int(v, p).map(|v| change(AntiAfkSetting::SwingDelay(v))),
                );
            }
            Err(error) => patch.issues.push(error),
        }
    }
    leaf(patch, custom, "Jump", "AntiAFK.Mode.Custom.Jump", |v, p| {
        parse_bool(v, p).map(|v| change(AntiAfkSetting::Jump(v)))
    });
    leaf(patch, custom, "Move", "AntiAFK.Mode.Custom.Move", |v, p| {
        parse_bool(v, p).map(|v| change(AntiAfkSetting::Move(v)))
    });
}
pub(super) fn parse_anti_afk(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|tag| Mode::parse(&tag))
    {
        Some(mode) => patch.changes.push(change(AntiAfkSetting::Mode(mode))),
        None => patch.issues.push(ConfigError::new(
            "AntiAFK.Mode.active",
            "missing or unknown case-sensitive mode",
        )),
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch.issues.push(ConfigError::new(
            "AntiAFK.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for name in MODES {
        let path = format!("AntiAFK.Mode.{name}");
        match choices.get(name) {
            Some(choice) => match nodes(choice, name, &path) {
                Ok(choice) => match name {
                    "RandomInteraction" => {
                        if let Some(node) = choice.get("Interaction") {
                            match node.get("value") {
                                Some(value) => patch
                                    .changes
                                    .push(change(AntiAfkSetting::Interaction(value.clone()))),
                                None => patch.issues.push(ConfigError::new(
                                    &format!("{path}.Interaction"),
                                    "missing value",
                                )),
                            }
                        }
                        leaf(patch, &choice, "Delay", &format!("{path}.Delay"), |v, p| {
                            parse_int_pair(v, p).map(|v| change(AntiAfkSetting::Delay(v)))
                        });
                    }
                    "Custom" => parse_custom(patch, &choice),
                    _ => {}
                },
                Err(error) => patch.issues.push(error),
            },
            None => patch
                .issues
                .push(ConfigError::new(&path, "choice not found")),
        }
    }
    if let Err(error) = nodes(mode, "Mode", "AntiAFK.Mode") {
        patch.issues.push(error);
    }
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
    fn anti_afk_complete_tree_roundtrips_all_modes_and_full_rotations() {
        for mode in [Mode::Old, Mode::RandomInteraction, Mode::Custom] {
            let mut state = AntiAfk::default();
            state.settings.mode = mode;
            let tree =
                serialize_anti_afk(&state, &common(), JsonFormat::Interop, &str::to_owned).unwrap();
            let settings = &tree["value"][2];
            assert_eq!(settings["active"], mode.tag());
            let random = &settings["choices"]["RandomInteraction"]["value"];
            assert_eq!(random[0]["choices"], json!(INTERACTIONS));
            assert_eq!(random[0]["value"], json!(["SwingHand", "Yaw", "Pitch"]));
            assert_eq!(random[1]["value"], json!({"from":4,"to":7}));
            assert_eq!(random[1]["suffix"], "ticks");
            let custom = &settings["choices"]["Custom"]["value"];
            assert_eq!(
                custom
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["name"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["Rotate", "Swing", "Jump", "Move"]
            );
            let rotate = &custom[0]["value"];
            assert_eq!(
                rotate
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["name"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "Enabled",
                    "IgnoreOpenInventory",
                    "Rotations",
                    "Delay",
                    "Angle"
                ]
            );
            assert!(rotate[2].to_string().contains(ROTATION_KEY));
            assert!(!rotate[2].to_string().contains("noRotateSet"));
            let stored =
                serialize_anti_afk(&state, &common(), JsonFormat::File, &str::to_owned).unwrap();
            let parsed = parse_settings(ModuleKind::AntiAfk, &stored).unwrap();
            assert!(parsed.issues.is_empty(), "{:?}", parsed.issues);
            let mut result = AntiAfk::default();
            for change_value in parsed.changes {
                if let SettingChange::AntiAfk(value) = change_value {
                    value.apply(&mut result).unwrap();
                }
            }
            assert_eq!(state.settings, result.settings);
        }
    }
    #[test]
    fn anti_afk_imports_raw_ranges_prefix_mutation_and_isolated_rotation_settings() {
        let mut state = AntiAfk::default();
        assert!(
            AntiAfkSetting::Interaction(json!(["Yaw", null, "Pitch"]))
                .apply(&mut state)
                .is_err()
        );
        assert_eq!(state.settings.interactions, 1 << 3);
        AntiAfkSetting::Interaction(json!(["pİtch", "jump", "Yaw", "Yaw", "unknown"]))
            .apply(&mut state)
            .unwrap();
        assert_eq!(state.settings.interactions, (1 << 0) | (1 << 3) | (1 << 4));
        let mut stored =
            serialize_anti_afk(&state, &common(), JsonFormat::File, &str::to_owned).unwrap();
        let mode = &mut stored["value"][3];
        mode["active"] = json!("custom");
        let random = &mut mode["choices"]["RandomInteraction"]["value"];
        random[1]["value"] = json!({"from":9,"to":-1});
        let rotate = &mut mode["choices"]["Custom"]["value"][0]["value"];
        rotate[3]["value"] = json!(4294967295u64);
        rotate[4]["value"] = json!("NaN");
        rotate[2]["value"][2]["value"] = json!("bad-float");
        let parsed = parse_settings(ModuleKind::AntiAfk, &stored).unwrap();
        assert_eq!(parsed.issues.len(), 2);
        assert!(
            parsed
                .issues
                .iter()
                .any(|e| e.path == "AntiAFK.Mode.Custom.Rotate.Rotations.ResetThreshold")
        );
        for change_value in parsed.changes {
            if let SettingChange::AntiAfk(value) = change_value {
                value.apply(&mut state).unwrap();
            }
        }
        assert_eq!(state.settings.mode, Mode::RandomInteraction);
        assert_eq!(state.settings.delay, [9, -1]);
        assert_eq!(state.settings.rotate_delay, -1);
        assert!(state.settings.angle.is_nan());
        assert_eq!(
            state.settings.rotations,
            liquidbounce_behaviors::no_rotate_set::RotationsSettings::default()
        );
    }
}
