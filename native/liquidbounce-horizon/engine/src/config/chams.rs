// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleChams declaration, LiquidBounce ae0eb396.
use super::*;
use liquidbounce_behaviors::chams::{Chams, ChamsMode, ImageFiltering, ImageMapping};

#[derive(Clone, Debug, PartialEq)]
pub enum ChamsSetting {
    Mode(ChamsMode),
    File(String),
    Mapping(ImageMapping),
    TileWidth(i32),
    Filtering(ImageFiltering),
    Offset([f32; 2]),
}
impl ChamsSetting {
    pub fn apply(self, state: &mut Chams) {
        match self {
            Self::Mode(v) => state.mode = v,
            Self::File(v) => state.file = v,
            Self::Mapping(v) => state.mapping = v,
            Self::TileWidth(v) => state.tile_width = v,
            Self::Filtering(v) => state.filtering = v,
            Self::Offset(v) => state.offset = v,
        }
    }
}
pub fn serialize_chams(
    state: &Chams,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut mapping_choices = Map::new();
    mapping_choices.insert(
        "Repeat".into(),
        tree.group(
            "Repeat",
            "CONFIGURABLE",
            vec![tree.ranged(
                "TileWidth",
                "INT",
                json!(state.tile_width),
                json!({"from":16,"to":2048}),
                "px",
                "liquidbounce.module.chams.mode.image.mapping.repeat.tileWidth",
            )],
        ),
    );
    for name in ["Stretch", "Cover"] {
        mapping_choices.insert(name.into(), tree.group(name, "CONFIGURABLE", vec![]));
    }
    let mut mapping = tree.mode(state.mapping.tag(), mapping_choices);
    mapping["name"] = json!("Mapping");
    if format == JsonFormat::Interop {
        mapping["key"] = json!("liquidbounce.module.chams.mode.image.mapping");
        mapping["description"] = json!(describe(
            "liquidbounce.module.chams.mode.image.mapping.description"
        ));
    }
    let mut filtering = tree.leaf(
        "Filtering",
        "CHOOSE",
        json!(state.filtering.tag()),
        "liquidbounce.module.chams.mode.image.filtering",
        None,
    );
    if format == JsonFormat::Interop {
        filtering["choices"] = json!(["Nearest", "Linear"]);
    }
    let offset = json!({"x":float_json(state.offset[0],"Chams.Mode.Image.Offset.x")?,
        "y":float_json(state.offset[1],"Chams.Mode.Image.Offset.y")?});
    let mut file = tree.leaf(
        "File",
        "FILE",
        json!(state.file),
        "liquidbounce.module.chams.mode.image.file",
        None,
    );
    if format == JsonFormat::Interop {
        file["dialogMode"] = json!("OPEN_FILE");
        file["supportedExtensions"] = json!(["png", "jpg", "jpeg"]);
    }
    let mut choices = Map::new();
    choices.insert(
        "Normal".into(),
        tree.group("Normal", "CONFIGURABLE", vec![]),
    );
    choices.insert(
        "Image".into(),
        tree.group(
            "Image",
            "CONFIGURABLE",
            vec![
                file,
                mapping,
                filtering,
                tree.leaf(
                    "Offset",
                    "VECTOR2_F",
                    offset,
                    "liquidbounce.module.chams.mode.image.offset",
                    None,
                ),
            ],
        ),
    );
    let mut mode = tree.mode(state.mode.tag(), choices);
    if format == JsonFormat::Interop {
        mode["key"] = json!("liquidbounce.module.chams.mode");
        mode["description"] = json!(describe("liquidbounce.module.chams.mode.description"));
    }
    let mut values = tree.common(ModuleKind::Chams, common);
    values.push(mode);
    Ok(tree.group("Chams", "TOGGLEABLE", values))
}

fn change(value: ChamsSetting) -> SettingChange {
    SettingChange::Chams(value)
}
/// Choice names are case-sensitive; enum tags use the original enum lookup's folding.
fn parse_choice(
    patch: &mut SettingsPatch,
    mode: &Value,
    name: &str,
    names: &[&str],
    path: &str,
    active: impl FnOnce(&str) -> Option<ChamsSetting>,
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
pub(super) fn parse_chams(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    parse_choice(
        patch,
        mode,
        "Mode",
        &["Normal", "Image"],
        "Chams.Mode",
        |tag| match tag {
            "Normal" => Some(ChamsSetting::Mode(ChamsMode::Normal)),
            "Image" => Some(ChamsSetting::Mode(ChamsMode::Image)),
            _ => None,
        },
        |patch, name, image| {
            if name != "Image" {
                return;
            }
            leaf(
                patch,
                image,
                "File",
                "Chams.Mode.Image.File",
                |value, path| {
                    scalar_string(value)
                        .map(|value| change(ChamsSetting::File(value)))
                        .ok_or_else(|| ConfigError::new(path, "expected file path"))
                },
            );
            if let Some(mapping) = image.get("Mapping") {
                parse_choice(
                    patch,
                    mapping,
                    "Mapping",
                    &["Repeat", "Stretch", "Cover"],
                    "Chams.Mode.Image.Mapping",
                    |tag| match tag {
                        "Repeat" => Some(ChamsSetting::Mapping(ImageMapping::Repeat)),
                        "Stretch" => Some(ChamsSetting::Mapping(ImageMapping::Stretch)),
                        "Cover" => Some(ChamsSetting::Mapping(ImageMapping::Cover)),
                        _ => None,
                    },
                    |patch, name, child| {
                        if name == "Repeat" {
                            leaf(
                                patch,
                                child,
                                "TileWidth",
                                "Chams.Mode.Image.Mapping.Repeat.TileWidth",
                                |value, path| {
                                    parse_int(value, path)
                                        .map(|value| change(ChamsSetting::TileWidth(value)))
                                },
                            );
                        }
                    },
                );
            }
            leaf(
                patch,
                image,
                "Filtering",
                "Chams.Mode.Image.Filtering",
                |value, path| {
                    let tag = scalar_string(value)
                        .ok_or_else(|| ConfigError::new(path, "expected enum tag"))?;
                    [ImageFiltering::Nearest, ImageFiltering::Linear]
                        .into_iter()
                        .find(|v| v.tag().eq_ignore_ascii_case(&tag))
                        .map(|value| change(ChamsSetting::Filtering(value)))
                        .ok_or_else(|| ConfigError::new(path, "unknown image filtering"))
                },
            );
            leaf(
                patch,
                image,
                "Offset",
                "Chams.Mode.Image.Offset",
                |value, path| {
                    let object = value
                        .as_object()
                        .ok_or_else(|| ConfigError::new(path, "expected vector object"))?;
                    let mut result = [0.0; 2];
                    for (index, name) in ["x", "y"].into_iter().enumerate() {
                        if let Some(value) = object.get(name) {
                            result[index] = parse_float(value, &format!("{path}.{name}"))?;
                        }
                    }
                    Ok(change(ChamsSetting::Offset(result)))
                },
            );
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
            key_bind: json!({}),
        }
    }
    fn changes(patch: &SettingsPatch) -> Vec<ChamsSetting> {
        patch
            .changes
            .iter()
            .filter_map(|change| match change {
                SettingChange::Chams(value) => Some(value.clone()),
                _ => None,
            })
            .collect()
    }
    #[test]
    fn all_modes_and_inactive_mapping_values_roundtrip_without_ui_clamping() {
        for mode in [ChamsMode::Normal, ChamsMode::Image] {
            for mapping in [
                ImageMapping::Repeat,
                ImageMapping::Stretch,
                ImageMapping::Cover,
            ] {
                for filtering in [ImageFiltering::Nearest, ImageFiltering::Linear] {
                    let state = Chams {
                        mode,
                        file: "图片/測試.jpg".into(),
                        mapping,
                        tile_width: -19,
                        filtering,
                        offset: [-1.25, 271.5],
                    };
                    let file = serialize_chams(&state, &common(), JsonFormat::File, &str::to_owned)
                        .unwrap();
                    let patch = parse_settings(ModuleKind::Chams, &file).unwrap();
                    assert!(patch.issues.is_empty(), "{:?}", patch.issues);
                    let mut restored = Chams::default();
                    let settings = changes(&patch);
                    assert_eq!(settings.len(), 6);
                    for value in settings {
                        value.apply(&mut restored);
                    }
                    assert_eq!(restored, state);
                }
            }
        }
    }
    #[test]
    fn original_file_dialog_vector_and_choice_metadata_reach_the_theme() {
        let interop = serialize_chams(
            &Chams::default(),
            &common(),
            JsonFormat::Interop,
            &str::to_owned,
        )
        .unwrap();
        let mode = &interop["value"][2];
        assert_eq!(mode["name"], "Mode");
        assert_eq!(mode["active"], "Normal");
        let image = mode["choices"]["Image"]["value"].as_array().unwrap();
        assert_eq!(
            image
                .iter()
                .map(|v| v["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["File", "Mapping", "Filtering", "Offset"]
        );
        assert_eq!(image[0]["dialogMode"], "OPEN_FILE");
        assert_eq!(
            image[0]["supportedExtensions"],
            json!(["png", "jpg", "jpeg"])
        );
        assert_eq!(
            image[1]["choices"]["Repeat"]["value"][0]["range"],
            json!({"from":16,"to":2048})
        );
        assert_eq!(image[2]["choices"], json!(["Nearest", "Linear"]));
        assert_eq!(image[3]["valueType"], "VECTOR2_F");
        assert_eq!(image[3]["value"], json!({"x":0.0,"y":0.0}));
        let file = serialize_chams(
            &Chams::default(),
            &common(),
            JsonFormat::File,
            &str::to_owned,
        )
        .unwrap();
        assert!(
            file["value"][3]["choices"]["Image"]["value"][0]
                .get("dialogMode")
                .is_none()
        );
    }
    #[test]
    fn malformed_choices_preserve_independent_leaves_and_vectors_default_missing_axes() {
        let input = json!({"name":"Chams","value":[{"name":"Mode","active":"image","value":[],"choices":{
            "Normal":{"name":"Normal","value":[]},
            "Image":{"name":"Image","value":[{"name":"Filtering","value":"linear"},{"name":"Offset","value":{"y":7.5,"ignored":{"anything":true}}}]}
        }}]});
        let patch = parse_settings(ModuleKind::Chams, &input).unwrap();
        assert_eq!(patch.issues.len(), 1);
        assert_eq!(patch.issues[0].path, "Chams.Mode.active");
        assert_eq!(
            changes(&patch),
            [
                ChamsSetting::Filtering(ImageFiltering::Linear),
                ChamsSetting::Offset([0.0, 7.5])
            ]
        );
        let state = Chams {
            offset: [f32::INFINITY, 0.0],
            ..Default::default()
        };
        assert!(serialize_chams(&state, &common(), JsonFormat::File, &str::to_owned).is_err());
    }
}
