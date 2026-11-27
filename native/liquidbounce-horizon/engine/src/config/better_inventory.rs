// SPDX-License-Identifier: GPL-3.0-or-later
//! Every BetterInventory value, including inactive Border and enum settings.
use super::*;
use liquidbounce_behaviors::better_inventory::{
    BetterInventory, CooldownMode, HighlightMode, COOLDOWN_MODES, HIGHLIGHT_MODES,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BetterInventorySetting {
    HighlightClicked(bool),
    HighlightMode(HighlightMode),
    HighlightColor(i32),
    TextCooldownProgress(bool),
    CooldownMode(CooldownMode),
    CooldownScale(f32),
    CooldownColor(i32),
    ContainerItemView(bool),
    SkipEmptyStack(bool),
    ContainerScale(f32),
    RelativeToMouse(bool),
    RenderOffsetX(f32),
    RenderOffsetY(f32),
    MoveCursor(bool),
}
impl BetterInventorySetting {
    pub fn apply(self, state: &mut BetterInventory) {
        match self {
            Self::HighlightClicked(v) => state.highlight_clicked = v,
            Self::HighlightMode(v) => state.highlight_mode = v,
            Self::HighlightColor(v) => state.highlight_color = v,
            Self::TextCooldownProgress(v) => state.text_cooldown = v,
            Self::CooldownMode(v) => state.cooldown_mode = v,
            Self::CooldownScale(v) => state.cooldown_scale = v,
            Self::CooldownColor(v) => state.cooldown_color = v,
            Self::ContainerItemView(v) => state.container_view = v,
            Self::SkipEmptyStack(v) => state.skip_empty_stack = v,
            Self::ContainerScale(v) => state.container_scale = v,
            Self::RelativeToMouse(v) => state.relative_to_mouse = v,
            Self::RenderOffsetX(v) => state.render_offset_x = v,
            Self::RenderOffsetY(v) => state.render_offset_y = v,
            Self::MoveCursor(v) => state.move_cursor = v,
        }
    }
}

/// ColorAdapter reads a signed long (then low 32 bits) or #AARRGGBB/#RRGGBB.
/// Value.deserializeFrom rejects a null result before replacing the old color.
fn parse_color(value: &Value, path: &str) -> Result<i32, ConfigError> {
    let invalid = || {
        ConfigError::new(
            path,
            "expected signed/unsigned ARGB number or 6/8 digit hex string",
        )
    };
    match value {
        // Value.deserializeFrom uses Gson.fromJson(JsonElement), hence
        // JsonTreeReader/LazilyParsedNumber.longValue: decimal truncation and
        // low 64 bits, followed by toInt. The final low 32 bits are exactly
        // decimal_int's result, including values outside the signed long range.
        Value::Number(n) => decimal_int(&n.to_string()).ok_or_else(invalid),
        Value::String(text) => {
            let hex = text.strip_prefix('#').unwrap_or(text);
            if !matches!(hex.len(), 6 | 8) {
                return Err(invalid());
            }
            // Kotlin radix parsing permits a leading sign if the total length
            // is still 6/8. Keep that behavior rather than validating digits.
            let value = i64::from_str_radix(hex, 16).map_err(|_| invalid())?;
            Ok(if hex.len() == 6 {
                (value as i32) | 0xff00_0000_u32 as i32
            } else {
                value as i32
            })
        }
        _ => Err(invalid()),
    }
}

pub fn serialize_better_inventory(
    state: &BetterInventory,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let key = |suffix: &str| format!("liquidbounce.module.betterInventory.{suffix}");
    let boolean = |name: &str, value: bool, suffix: &str| {
        tree.leaf(name, "BOOLEAN", json!(value), &key(suffix), None)
    };
    let color = |name: &str, value: i32, suffix: &str| {
        tree.leaf(name, "COLOR", json!(value), &key(suffix), None)
    };
    let scale = |value: f32, suffix: &str| -> Result<Value, ConfigError> {
        Ok(tree.ranged(
            "Scale",
            "FLOAT",
            float_json(value, suffix)?,
            float_range(0.25, 4.0, suffix)?,
            "",
            &key(suffix),
        ))
    };
    let mut choices = Map::new();
    choices.insert(
        "Border".into(),
        tree.group(
            "Border",
            "CONFIGURABLE",
            vec![color(
                "Color",
                state.highlight_color,
                "highlightClicked.mode.border.color",
            )],
        ),
    );
    choices.insert(
        "Texture".into(),
        tree.group("Texture", "CONFIGURABLE", vec![]),
    );
    let highlight = tree.group(
        "HighlightClicked",
        "TOGGLEABLE",
        vec![
            boolean(
                "Enabled",
                state.highlight_clicked,
                "highlightClicked.enabled",
            ),
            tree.mode(state.highlight_mode.tag(), choices),
        ],
    );
    let mut mode = tree.leaf(
        "Mode",
        "CHOOSE",
        json!(state.cooldown_mode.tag()),
        &key("textCooldownProgress.mode"),
        None,
    );
    if format == JsonFormat::Interop {
        mode["choices"] = json!(COOLDOWN_MODES);
    }
    let cooldown = tree.group(
        "TextCooldownProgress",
        "TOGGLEABLE",
        vec![
            boolean(
                "Enabled",
                state.text_cooldown,
                "textCooldownProgress.enabled",
            ),
            mode,
            scale(state.cooldown_scale, "textCooldownProgress.scale")?,
            color("Color", state.cooldown_color, "textCooldownProgress.color"),
        ],
    );
    let mut container = vec![
        boolean("Enabled", state.container_view, "containerItemView.enabled"),
        boolean(
            "SkipEmptyStack",
            state.skip_empty_stack,
            "containerItemView.skipEmptyStack",
        ),
        scale(state.container_scale, "containerItemView.scale")?,
        boolean(
            "RelativeToMouse",
            state.relative_to_mouse,
            "containerItemView.relativeToMouse",
        ),
    ];
    for (name, value, suffix) in [
        (
            "RenderOffsetX",
            state.render_offset_x,
            "containerItemView.renderOffsetX",
        ),
        (
            "RenderOffsetY",
            state.render_offset_y,
            "containerItemView.renderOffsetY",
        ),
    ] {
        container.push(tree.ranged(
            name,
            "FLOAT",
            float_json(value, name)?,
            float_range(-4096.0, 4096.0, name)?,
            "",
            &key(suffix),
        ));
    }
    let mut values = tree.common(ModuleKind::BetterInventory, common);
    values.extend([
        highlight,
        cooldown,
        tree.group("ContainerItemView", "TOGGLEABLE", container),
        boolean("MoveCursor", state.move_cursor, "moveCursor"),
    ]);
    Ok(tree.group("BetterInventory", "TOGGLEABLE", values))
}

pub(super) fn parse_better_inventory(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    use BetterInventorySetting as S;
    leaf(
        patch,
        root,
        "MoveCursor",
        "BetterInventory.MoveCursor",
        |v, p| parse_bool(v, p).map(|v| SettingChange::BetterInventory(S::MoveCursor(v))),
    );
    let mut group = |name: &str, parse: &dyn Fn(&mut SettingsPatch, &Nodes<'_>)| {
        let Some(value) = root.get(name) else {
            return;
        };
        match nodes(value, name, &format!("BetterInventory.{name}")) {
            Ok(children) => parse(patch, &children),
            Err(error) => patch.issues.push(error),
        }
    };
    group("HighlightClicked", &|patch, children| {
        leaf(
            patch,
            children,
            "Enabled",
            "BetterInventory.HighlightClicked.Enabled",
            |v, p| parse_bool(v, p).map(|v| SettingChange::BetterInventory(S::HighlightClicked(v))),
        );
        let Some(mode) = children.get("Mode") else {
            return;
        };
        match mode
            .get("active")
            .and_then(scalar_string)
            .and_then(|tag| HighlightMode::parse(&tag))
        {
            Some(value) => patch
                .changes
                .push(SettingChange::BetterInventory(S::HighlightMode(value))),
            None => patch.issues.push(ConfigError::new(
                "BetterInventory.HighlightClicked.Mode.active",
                "missing or unknown case-sensitive mode",
            )),
        }
        if let Err(error) = nodes(mode, "Mode", "BetterInventory.HighlightClicked.Mode") {
            patch.issues.push(error);
        }
        let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
            patch.issues.push(ConfigError::new(
                "BetterInventory.HighlightClicked.Mode.choices",
                "expected choices object",
            ));
            return;
        };
        for name in HIGHLIGHT_MODES {
            let path = format!("BetterInventory.HighlightClicked.Mode.{name}");
            match choices.get(name) {
                Some(value) => match nodes(value, name, &path) {
                    Ok(children) if name == "Border" => leaf(
                        patch,
                        &children,
                        "Color",
                        &format!("{path}.Color"),
                        |v, p| {
                            parse_color(v, p)
                                .map(|v| SettingChange::BetterInventory(S::HighlightColor(v)))
                        },
                    ),
                    Ok(_) => {}
                    Err(error) => patch.issues.push(error),
                },
                None => patch
                    .issues
                    .push(ConfigError::new(&path, "choice not found")),
            }
        }
    });
    group("TextCooldownProgress", &|patch, children| {
        let path = "BetterInventory.TextCooldownProgress";
        leaf(
            patch,
            children,
            "Enabled",
            &format!("{path}.Enabled"),
            |v, p| {
                parse_bool(v, p).map(|v| SettingChange::BetterInventory(S::TextCooldownProgress(v)))
            },
        );
        leaf(patch, children, "Mode", &format!("{path}.Mode"), |v, p| {
            scalar_string(v)
                .and_then(|tag| CooldownMode::parse(&tag))
                .map(|v| SettingChange::BetterInventory(S::CooldownMode(v)))
                .ok_or_else(|| ConfigError::new(p, "unknown case-insensitive cooldown mode"))
        });
        leaf(
            patch,
            children,
            "Scale",
            &format!("{path}.Scale"),
            |v, p| parse_float(v, p).map(|v| SettingChange::BetterInventory(S::CooldownScale(v))),
        );
        leaf(
            patch,
            children,
            "Color",
            &format!("{path}.Color"),
            |v, p| parse_color(v, p).map(|v| SettingChange::BetterInventory(S::CooldownColor(v))),
        );
    });
    group("ContainerItemView", &|patch, children| {
        let path = "BetterInventory.ContainerItemView";
        for (name, make) in [
            ("Enabled", S::ContainerItemView as fn(bool) -> S),
            ("SkipEmptyStack", S::SkipEmptyStack),
            ("RelativeToMouse", S::RelativeToMouse),
        ] {
            leaf(patch, children, name, &format!("{path}.{name}"), |v, p| {
                parse_bool(v, p).map(|v| SettingChange::BetterInventory(make(v)))
            });
        }
        for (name, make) in [
            ("Scale", S::ContainerScale as fn(f32) -> S),
            ("RenderOffsetX", S::RenderOffsetX),
            ("RenderOffsetY", S::RenderOffsetY),
        ] {
            leaf(patch, children, name, &format!("{path}.{name}"), |v, p| {
                parse_float(v, p).map(|v| SettingChange::BetterInventory(make(v)))
            });
        }
    });
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
    fn apply(value: &Value) -> (BetterInventory, SettingsPatch) {
        let patch = parse_settings(ModuleKind::BetterInventory, value).unwrap();
        let mut state = BetterInventory::default();
        for change in &patch.changes {
            if let SettingChange::BetterInventory(value) = change {
                value.apply(&mut state);
            }
        }
        (state, patch)
    }
    fn leaf_mut<'a>(root: &'a mut Value, name: &str) -> &'a mut Value {
        root["value"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|v| v["name"] == name)
            .unwrap()
    }
    #[test]
    fn full_tree_roundtrip_includes_inactive_choices_and_all_fourteen_values() {
        for highlight_mode in [HighlightMode::Border, HighlightMode::Texture] {
            for cooldown_mode in [
                CooldownMode::Percentage,
                CooldownMode::DurationTicks,
                CooldownMode::DurationSeconds,
            ] {
                let state = BetterInventory {
                    highlight_clicked: false,
                    highlight_mode,
                    highlight_color: 0x12345678,
                    text_cooldown: false,
                    cooldown_mode,
                    cooldown_scale: 3.25,
                    cooldown_color: -123,
                    container_view: false,
                    skip_empty_stack: true,
                    container_scale: 0.5,
                    relative_to_mouse: false,
                    render_offset_x: -123.5,
                    render_offset_y: 999.5,
                    move_cursor: false,
                };
                for format in [JsonFormat::File, JsonFormat::Interop] {
                    let file =
                        serialize_better_inventory(&state, &common(), format, &str::to_owned)
                            .unwrap();
                    let (actual, patch) = apply(&file);
                    assert!(patch.issues.is_empty(), "{:?}", patch.issues);
                    assert_eq!(actual, state);
                    assert_eq!(
                        patch
                            .changes
                            .iter()
                            .filter(|c| matches!(c, SettingChange::BetterInventory(_)))
                            .count(),
                        14
                    );
                }
            }
        }
    }
    #[test]
    fn metadata_matches_source_ranges_choice_semantics_and_signed_colors() {
        let mut file = serialize_better_inventory(
            &BetterInventory::default(),
            &common(),
            JsonFormat::Interop,
            &str::to_owned,
        )
        .unwrap();
        let cooldown = leaf_mut(&mut file, "TextCooldownProgress");
        assert_eq!(leaf_mut(cooldown, "Mode")["choices"], json!(COOLDOWN_MODES));
        assert_eq!(
            leaf_mut(cooldown, "Scale")["range"],
            json!({"from":0.25,"to":4.0})
        );
        assert_eq!(leaf_mut(cooldown, "Color")["value"], -1);
        let container = leaf_mut(&mut file, "ContainerItemView");
        assert_eq!(
            leaf_mut(container, "RenderOffsetX")["range"],
            json!({"from":-4096.0,"to":4096.0})
        );
        assert_eq!(ModuleKind::BetterInventory.category(), "Render");
    }
    #[test]
    fn invalid_mode_does_not_abort_other_settings_or_inactive_color() {
        let mut file = serialize_better_inventory(
            &BetterInventory::default(),
            &common(),
            JsonFormat::File,
            &str::to_owned,
        )
        .unwrap();
        let h = leaf_mut(&mut file, "HighlightClicked");
        let mode = leaf_mut(h, "Mode");
        mode["active"] = json!("border");
        leaf_mut(&mut mode["choices"]["Border"], "Color")["value"] = json!("#80123456");
        let c = leaf_mut(&mut file, "TextCooldownProgress");
        leaf_mut(c, "Mode")["value"] = json!("duratİontıcks");
        leaf_mut(c, "Scale")["value"] = json!(-10.0);
        leaf_mut(&mut file, "MoveCursor")["value"] = json!(false);
        let (state, patch) = apply(&file);
        assert_eq!(patch.issues.len(), 1);
        assert_eq!(state.highlight_color, 0x80123456_u32 as i32);
        assert_eq!(state.cooldown_mode, CooldownMode::DurationTicks);
        assert_eq!(state.cooldown_scale, -10.0);
        assert!(!state.move_cursor);
    }
    #[test]
    fn color_adapter_accepts_source_numeric_and_hex_forms_and_rejects_null() {
        for (value, expected) in [
            (json!(4294967295_u64), -1),
            (json!(-1), -1),
            (json!(4294967296_u64), 0),
            (json!("#4677ff"), 0xff4677ff_u32 as i32),
            (json!("80123456"), 0x80123456_u32 as i32),
            (json!("+12345"), 0xff012345_u32 as i32),
            (json!("-12345"), -0x12345),
            (json!(1.5), 1),
            (json!(-1.5), -1),
        ] {
            assert_eq!(parse_color(&value, "color").unwrap(), expected);
        }
        for value in [json!(null), json!(true), json!("12345"), json!("#gggggg")] {
            assert!(parse_color(&value, "color").is_err());
        }
    }
    #[test]
    fn raw_nonfinite_imports_and_runtime_forwarding_remain_visible() {
        let mut file = serialize_better_inventory(
            &BetterInventory::default(),
            &common(),
            JsonFormat::File,
            &str::to_owned,
        )
        .unwrap();
        leaf_mut(leaf_mut(&mut file, "ContainerItemView"), "Scale")["value"] = json!("NaN");
        let (state, patch) = apply(&file);
        assert!(patch.issues.is_empty());
        assert!(state.container_scale.is_nan());
        assert!(
            serialize_better_inventory(&state, &common(), JsonFormat::Interop, &str::to_owned)
                .is_err()
        );
        assert!(matches!(
            apply_setting(
                &SettingChange::BetterInventory(BetterInventorySetting::MoveCursor(false)),
                &mut NoFov::default(),
                &mut Zoom::default(),
                &mut FullBright::default()
            ),
            ApplyResult::Runtime(RuntimeRequest::BetterInventory(
                BetterInventorySetting::MoveCursor(false)
            ))
        ));
    }
}
