// SPDX-License-Identifier: GPL-3.0-or-later
//! Full CameraClip declaration tree, including ScrollAdjustValueGroup inheritance.
use super::*;
use liquidbounce_behaviors::camera_clip::{CameraClip, ModifierKey};

#[derive(Clone, Debug, PartialEq)]
pub enum CameraClipSetting {
    CameraDistance(f32),
    AnimationEnabled(bool),
    AnimationSpeed(f32),
    AnimationEasing(Easing),
    ScrollEnabled(bool),
    Modifier(ModifierKey),
    Sensitivity(f32),
    RememberScrolled(bool),
    RequireFreeLook(bool),
}
impl CameraClipSetting {
    /// A running parent's child onEnabled may also mutate CameraDistance.
    pub fn apply(&self, module: &mut CameraClip, parent_enabled: bool, in_game: bool) -> bool {
        match self {
            Self::CameraDistance(value) => module.camera_distance = *value,
            Self::AnimationEnabled(value) => module.animation_enabled = *value,
            Self::AnimationSpeed(value) => module.animation_speed = *value,
            Self::AnimationEasing(value) => module.animation_easing = *value,
            Self::ScrollEnabled(value) => {
                return module.set_scroll_enabled(*value, parent_enabled, in_game);
            }
            Self::Modifier(value) => module.modifier = value.clone(),
            Self::Sensitivity(value) => module.sensitivity = *value,
            Self::RememberScrolled(value) => module.remember_scrolled = *value,
            Self::RequireFreeLook(value) => module.require_free_look = *value,
        }
        false
    }
}

pub fn serialize_camera_clip(
    state: &CameraClip,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::CameraClip, common);
    values.push(tree.ranged(
        "CameraDistance",
        "FLOAT",
        float_json(state.camera_distance, "CameraClip.CameraDistance")?,
        json!({"from":1.0,"to":48.0}),
        "",
        "liquidbounce.module.cameraClip.cameraDistance",
    ));
    let mut easing = tree.leaf(
        "Easing",
        "CHOOSE",
        json!(state.animation_easing.tag()),
        "liquidbounce.module.cameraClip.animation.easing",
        None,
    );
    if format == JsonFormat::Interop {
        easing["choices"] = json!(Easing::ALL.map(Easing::tag));
    }
    values.push(tree.group(
        "Animation",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(state.animation_enabled),
                "liquidbounce.module.cameraClip.animation.enabled",
                None,
            ),
            tree.ranged(
                "Speed",
                "FLOAT",
                float_json(state.animation_speed, "CameraClip.Animation.Speed")?,
                json!({"from":0.1,"to":1.0}),
                "",
                "liquidbounce.module.cameraClip.animation.speed",
            ),
            easing,
        ],
    ));
    values.push(tree.group(
        "ScrollAdjust",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(state.scroll_enabled),
                "liquidbounce.module.cameraClip.scrollAdjust.enabled",
                None,
            ),
            tree.leaf(
                "Modifier",
                "KEY",
                json!(state.modifier.name),
                "liquidbounce.module.cameraClip.scrollAdjust.modifier",
                None,
            ),
            tree.ranged(
                "Sensitivity",
                "FLOAT",
                float_json(state.sensitivity, "CameraClip.ScrollAdjust.Sensitivity")?,
                json!({"from":0.1,"to":2.0}),
                "",
                "liquidbounce.module.cameraClip.scrollAdjust.sensitivity",
            ),
            tree.leaf(
                "RememberScrolled",
                "BOOLEAN",
                json!(state.remember_scrolled),
                "liquidbounce.module.cameraClip.scrollAdjust.rememberScrolled",
                None,
            ),
            tree.leaf(
                "RequireFreeLook",
                "BOOLEAN",
                json!(state.require_free_look),
                "liquidbounce.module.cameraClip.scrollAdjust.requireFreeLook",
                None,
            ),
        ],
    ));
    Ok(tree.group("CameraClip", "TOGGLEABLE", values))
}

pub(super) fn parse_camera_clip(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "CameraDistance",
        "CameraClip.CameraDistance",
        |v, p| {
            parse_float(v, p)
                .map(|v| SettingChange::CameraClip(CameraClipSetting::CameraDistance(v)))
        },
    );
    if let Some(group) = root.get("Animation") {
        match nodes(group, "Animation", "CameraClip.Animation") {
            Ok(group) => {
                leaf(
                    patch,
                    &group,
                    "Enabled",
                    "CameraClip.Animation.Enabled",
                    |v, p| {
                        parse_bool(v, p).map(|v| {
                            SettingChange::CameraClip(CameraClipSetting::AnimationEnabled(v))
                        })
                    },
                );
                leaf(
                    patch,
                    &group,
                    "Speed",
                    "CameraClip.Animation.Speed",
                    |v, p| {
                        parse_float(v, p).map(|v| {
                            SettingChange::CameraClip(CameraClipSetting::AnimationSpeed(v))
                        })
                    },
                );
                leaf(
                    patch,
                    &group,
                    "Easing",
                    "CameraClip.Animation.Easing",
                    |v, p| {
                        let tag = scalar_string(v)
                            .ok_or_else(|| ConfigError::new(p, "expected enum tag"))?;
                        Easing::ALL
                            .into_iter()
                            .find(|choice| choice.tag().eq_ignore_ascii_case(&tag))
                            .map(|v| {
                                SettingChange::CameraClip(CameraClipSetting::AnimationEasing(v))
                            })
                            .ok_or_else(|| ConfigError::new(p, format!("unknown Easing {tag:?}")))
                    },
                );
            }
            Err(error) => patch.issues.push(error),
        }
    }
    if let Some(group) = root.get("ScrollAdjust") {
        match nodes(group, "ScrollAdjust", "CameraClip.ScrollAdjust") {
            Ok(group) => {
                leaf(
                    patch,
                    &group,
                    "Enabled",
                    "CameraClip.ScrollAdjust.Enabled",
                    |v, p| {
                        parse_bool(v, p)
                            .map(|v| SettingChange::CameraClip(CameraClipSetting::ScrollEnabled(v)))
                    },
                );
                leaf(
                    patch,
                    &group,
                    "Modifier",
                    "CameraClip.ScrollAdjust.Modifier",
                    |v, p| {
                        let name = v.as_str().ok_or_else(|| {
                            ConfigError::new(p, "expected InputConstants key string")
                        })?;
                        ModifierKey::from_name(name)
                            .map(|v| SettingChange::CameraClip(CameraClipSetting::Modifier(v)))
                            .ok_or_else(|| {
                                ConfigError::new(p, format!("unknown input key {name:?}"))
                            })
                    },
                );
                leaf(
                    patch,
                    &group,
                    "Sensitivity",
                    "CameraClip.ScrollAdjust.Sensitivity",
                    |v, p| {
                        parse_float(v, p)
                            .map(|v| SettingChange::CameraClip(CameraClipSetting::Sensitivity(v)))
                    },
                );
                leaf(
                    patch,
                    &group,
                    "RememberScrolled",
                    "CameraClip.ScrollAdjust.RememberScrolled",
                    |v, p| {
                        parse_bool(v, p).map(|v| {
                            SettingChange::CameraClip(CameraClipSetting::RememberScrolled(v))
                        })
                    },
                );
                leaf(
                    patch,
                    &group,
                    "RequireFreeLook",
                    "CameraClip.ScrollAdjust.RequireFreeLook",
                    |v, p| {
                        parse_bool(v, p).map(|v| {
                            SettingChange::CameraClip(CameraClipSetting::RequireFreeLook(v))
                        })
                    },
                );
            }
            Err(error) => patch.issues.push(error),
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
            key_bind: json!({"key":"key.keyboard.unknown","action":"TOGGLE"}),
        }
    }
    fn camera_changes(patch: &SettingsPatch) -> Vec<CameraClipSetting> {
        patch
            .changes
            .iter()
            .filter_map(|change| match change {
                SettingChange::CameraClip(value) => Some(value.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn complete_file_roundtrip_preserves_raw_imports_and_key_identity() {
        for easing in Easing::ALL {
            let state = CameraClip {
                camera_distance: -97.5,
                animation_enabled: false,
                animation_speed: 8.25,
                animation_easing: easing,
                scroll_enabled: false,
                modifier: ModifierKey::from_name("key.mouse.99").unwrap(),
                sensitivity: -3.75,
                remember_scrolled: true,
                require_free_look: true,
                ..Default::default()
            };
            let file =
                serialize_camera_clip(&state, &common(), JsonFormat::File, &str::to_owned).unwrap();
            let patch = parse_settings(ModuleKind::CameraClip, &file).unwrap();
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            let mut restored = CameraClip::default();
            let changes = camera_changes(&patch);
            assert_eq!(changes.len(), 9);
            for change in changes {
                assert!(!change.apply(&mut restored, false, false));
            }
            assert_eq!(restored, state);
        }
    }

    #[test]
    fn json_order_cannot_reorder_child_enable_and_remember_callbacks() {
        let input = json!({"name":"CameraClip","value":[
            {"name":"ScrollAdjust","value":[
                {"name":"RequireFreeLook","value":true},
                {"name":"RememberScrolled","value":true},
                {"name":"Sensitivity","value":-4.0},
                {"name":"Modifier","value":"key.keyboard.224"},
                {"name":"Enabled","value":true}]},
            {"name":"Animation","value":[
                {"name":"Easing","value":"linear"},
                {"name":"Speed","value":2.5},
                {"name":"Enabled","value":false}]},
            {"name":"CameraDistance","value":19.0}]});
        let patch = parse_settings(ModuleKind::CameraClip, &input).unwrap();
        assert!(patch.issues.is_empty());
        let changes = camera_changes(&patch);
        assert_eq!(
            changes,
            vec![
                CameraClipSetting::CameraDistance(19.0),
                CameraClipSetting::AnimationEnabled(false),
                CameraClipSetting::AnimationSpeed(2.5),
                CameraClipSetting::AnimationEasing(Easing::Linear),
                CameraClipSetting::ScrollEnabled(true),
                CameraClipSetting::Modifier(ModifierKey::default()),
                CameraClipSetting::Sensitivity(-4.0),
                CameraClipSetting::RememberScrolled(true),
                CameraClipSetting::RequireFreeLook(true),
            ]
        );
        let mut state = CameraClip {
            scroll_enabled: false,
            scrolled_distance: 7.0,
            ..Default::default()
        };
        for change in changes {
            assert!(!change.apply(&mut state, true, true));
        }
        // The child enable executes before RememberScrolled and resets to the
        // already imported target. Reordering would incorrectly persist 7.
        assert_eq!(state.camera_distance, 19.0);
        assert_eq!(state.scrolled_distance, 19.0);
        assert!(state.remember_scrolled);
    }

    #[test]
    fn invalid_key_token_is_local_to_its_leaf_and_numeric_keys_canonicalize() {
        for invalid in [
            json!(224),
            json!(true),
            json!("left.control"),
            json!("key.keyboard.2147483648"),
        ] {
            let patch = parse_settings(ModuleKind::CameraClip, &json!({"name":"CameraClip","value":[
                {"name":"ScrollAdjust","value":[{"name":"Modifier","value":invalid},{"name":"Sensitivity","value":7.0}]}]})).unwrap();
            assert_eq!(patch.issues.len(), 1);
            assert_eq!(patch.issues[0].path, "CameraClip.ScrollAdjust.Modifier");
            assert_eq!(
                camera_changes(&patch),
                vec![CameraClipSetting::Sensitivity(7.0)]
            );
        }
        let patch = parse_settings(
            ModuleKind::CameraClip,
            &json!({"name":"CameraClip","value":[
            {"name":"ScrollAdjust","value":[{"name":"Modifier","value":"key.keyboard.224"}]}]}),
        )
        .unwrap();
        assert_eq!(
            camera_changes(&patch),
            vec![CameraClipSetting::Modifier(ModifierKey::default())]
        );
    }

    #[test]
    fn interop_retains_ranges_choices_and_rejects_nonfinite_float_values() {
        let mut state = CameraClip::default();
        let json =
            serialize_camera_clip(&state, &common(), JsonFormat::Interop, &str::to_owned).unwrap();
        let values = json["value"].as_array().unwrap();
        let target = values
            .iter()
            .find(|v| v["name"] == "CameraDistance")
            .unwrap();
        assert_eq!(target["range"], json!({"from":1.0,"to":48.0}));
        let animation = values.iter().find(|v| v["name"] == "Animation").unwrap();
        assert_eq!(
            animation["value"][2]["choices"],
            json!(Easing::ALL.map(Easing::tag))
        );
        state.sensitivity = f32::NAN;
        assert!(
            serialize_camera_clip(&state, &common(), JsonFormat::File, &str::to_owned).is_err()
        );
        state.sensitivity = 0.3;
        state.camera_distance = f32::INFINITY;
        assert!(
            serialize_camera_clip(&state, &common(), JsonFormat::Interop, &str::to_owned).is_err()
        );
    }
}
