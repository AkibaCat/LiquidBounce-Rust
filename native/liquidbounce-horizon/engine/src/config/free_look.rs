// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleFreeLook declaration, preserving original enum order and tags.
use super::*;
use liquidbounce_behaviors::free_look::{FreeLook, PerspectiveChoice};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FreeLookSetting {
    Perspective(PerspectiveChoice),
    SenseBoost(f32),
    NoPitchLimit(bool),
}
impl FreeLookSetting {
    pub fn apply(self, module: &mut FreeLook) {
        match self {
            Self::Perspective(value) => module.perspective = value,
            Self::SenseBoost(value) => module.sense_boost = value,
            Self::NoPitchLimit(value) => module.no_pitch_limit = value,
        }
    }
}
pub fn serialize_free_look(
    state: &FreeLook,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::FreeLook, common);
    let mut perspective = tree.leaf(
        "Perspective",
        "CHOOSE",
        json!(state.perspective.tag()),
        "liquidbounce.module.freeLook.perspective",
        None,
    );
    if format == JsonFormat::Interop {
        perspective["choices"] = json!(PerspectiveChoice::ALL.map(PerspectiveChoice::tag));
    }
    values.push(perspective);
    values.push(tree.ranged(
        "SenseBoost",
        "FLOAT",
        float_json(state.sense_boost, "FreeLook.SenseBoost")?,
        json!({"from":0.1,"to":2.0}),
        "",
        "liquidbounce.module.freeLook.senseBoost",
    ));
    values.push(tree.leaf(
        "NoPitchLimit",
        "BOOLEAN",
        json!(state.no_pitch_limit),
        "liquidbounce.module.freeLook.noPitchLimit",
        None,
    ));
    Ok(tree.group("FreeLook", "TOGGLEABLE", values))
}
pub(super) fn parse_free_look(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "Perspective",
        "FreeLook.Perspective",
        |value, path| {
            let tag =
                scalar_string(value).ok_or_else(|| ConfigError::new(path, "expected enum tag"))?;
            PerspectiveChoice::from_tag(&tag)
                .map(|value| SettingChange::FreeLook(FreeLookSetting::Perspective(value)))
                .ok_or_else(|| ConfigError::new(path, "unknown FreeLook perspective"))
        },
    );
    leaf(
        patch,
        root,
        "SenseBoost",
        "FreeLook.SenseBoost",
        |value, path| {
            parse_float(value, path)
                .map(|value| SettingChange::FreeLook(FreeLookSetting::SenseBoost(value)))
        },
    );
    leaf(
        patch,
        root,
        "NoPitchLimit",
        "FreeLook.NoPitchLimit",
        |value, path| {
            parse_bool(value, path)
                .map(|value| SettingChange::FreeLook(FreeLookSetting::NoPitchLimit(value)))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Hold"}),
        }
    }
    #[test]
    fn original_enum_order_ranges_and_all_values_roundtrip_without_resetting_angles() {
        for perspective in PerspectiveChoice::ALL {
            for sense_boost in [-2.5, 0.0, 1.0, 4.5] {
                let original = FreeLook {
                    perspective,
                    sense_boost,
                    no_pitch_limit: false,
                    camera_yaw: 60.0,
                    camera_pitch: 120.0,
                };
                for format in [JsonFormat::Interop, JsonFormat::File] {
                    let value =
                        serialize_free_look(&original, &common(), format, &str::to_owned).unwrap();
                    let patch = parse_settings(ModuleKind::FreeLook, &value).unwrap();
                    assert!(patch.issues.is_empty(), "{:?}", patch.issues);
                    let mut restored = FreeLook {
                        camera_yaw: 60.0,
                        camera_pitch: 120.0,
                        ..Default::default()
                    };
                    for change in patch.changes {
                        if let SettingChange::FreeLook(value) = change {
                            value.apply(&mut restored);
                        }
                    }
                    assert_eq!(restored, original);
                    if format == JsonFormat::Interop {
                        let values = value["value"].as_array().unwrap();
                        let perspective =
                            values.iter().find(|v| v["name"] == "Perspective").unwrap();
                        assert_eq!(perspective["choices"], json!(["Front", "Back"]));
                        let sense = values.iter().find(|v| v["name"] == "SenseBoost").unwrap();
                        assert_eq!(sense["range"], json!({"from":0.1,"to":2.0}));
                    }
                }
            }
        }
    }
    #[test]
    fn bad_enum_reports_an_issue_and_later_valid_settings_still_apply() {
        let value = json!({"name":"FreeLook","value":[
            {"name":"Perspective","value":"FirstPerson"},
            {"name":"SenseBoost","value":3.0},
            {"name":"NoPitchLimit","value":false}]});
        let patch = parse_settings(ModuleKind::FreeLook, &value).unwrap();
        assert_eq!(patch.issues.len(), 1);
        assert!(
            patch
                .changes
                .contains(&SettingChange::FreeLook(FreeLookSetting::SenseBoost(3.0)))
        );
        assert!(
            patch
                .changes
                .contains(&SettingChange::FreeLook(FreeLookSetting::NoPitchLimit(
                    false
                )))
        );
        assert!(
            serialize_free_look(
                &FreeLook {
                    sense_boost: f32::NAN,
                    ..Default::default()
                },
                &common(),
                JsonFormat::File,
                &str::to_owned
            )
            .is_err()
        );
    }
}
