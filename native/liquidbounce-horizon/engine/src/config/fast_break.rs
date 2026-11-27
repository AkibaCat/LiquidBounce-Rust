// SPDX-License-Identifier: GPL-3.0-or-later
//! All original FastBreak values and both modes, including the former alias.
use super::*;
use liquidbounce_behaviors::fast_break::{FastBreak, Mode};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FastBreakSetting {
    BreakDamage(f32),
    OnlyTool(bool),
    Mode(Mode),
}
impl FastBreakSetting {
    pub fn apply(self, state: &mut FastBreak) {
        match self {
            Self::BreakDamage(value) => state.break_damage = value,
            Self::OnlyTool(value) => state.only_tool = value,
            Self::Mode(value) => state.mode = value,
        }
    }
}
pub fn serialize_fast_break(
    state: &FastBreak,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::FastBreak, common);
    values.push(tree.ranged(
        "BreakDamage",
        "FLOAT",
        float_json(state.break_damage, "FastBreak.BreakDamage")?,
        json!({"from":0.1,"to":1.0}),
        "",
        "liquidbounce.module.fastBreak.breakDamage",
    ));
    values.push(tree.leaf(
        "OnlyTool",
        "BOOLEAN",
        json!(state.only_tool),
        "liquidbounce.module.fastBreak.onlyTool",
        None,
    ));
    let choices = [Mode::None, Mode::Grim2348]
        .into_iter()
        .map(|mode| {
            (
                mode.tag().to_owned(),
                tree.group(mode.tag(), "CONFIGURABLE", vec![]),
            )
        })
        .collect();
    values.push(tree.mode(state.mode.tag(), choices));
    Ok(tree.group("FastBreak", "TOGGLEABLE", values))
}
pub(super) fn parse_fast_break(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "BreakDamage",
        "FastBreak.BreakDamage",
        |value, path| {
            parse_float(value, path)
                .map(|value| SettingChange::FastBreak(FastBreakSetting::BreakDamage(value)))
        },
    );
    leaf(
        patch,
        root,
        "OnlyTool",
        "FastBreak.OnlyTool",
        |value, path| {
            parse_bool(value, path)
                .map(|value| SettingChange::FastBreak(FastBreakSetting::OnlyTool(value)))
        },
    );
    let Some(mode) = root.get("Mode") else {
        return;
    };
    let path = "FastBreak.Mode";
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|name| Mode::from_tag(&name))
    {
        Some(mode) => patch
            .changes
            .push(SettingChange::FastBreak(FastBreakSetting::Mode(mode))),
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
    for name in ["None", "Grim-2.3.48"] {
        let key = format!("{path}.{name}");
        match choices.get(name) {
            Some(choice) => {
                if let Err(error) = nodes(choice, name, &key) {
                    patch.issues.push(error);
                }
            }
            None => patch
                .issues
                .push(ConfigError::new(&key, "choice not found")),
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
    fn complete_tree_roundtrips_modes_and_raw_values() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        for mode in [Mode::None, Mode::Grim2348] {
            for damage in [-4.0, 0.0, 0.8, 5.0] {
                for format in [JsonFormat::File, JsonFormat::Interop] {
                    let state = FastBreak {
                        break_damage: damage,
                        only_tool: true,
                        mode,
                    };
                    let tree =
                        serialize_fast_break(&state, &common, format, &str::to_owned).unwrap();
                    let values = tree["value"].as_array().unwrap();
                    let mut expected = vec!["Bind", "Hidden", "BreakDamage", "OnlyTool", "Mode"];
                    if format == JsonFormat::File {
                        expected.insert(0, "Enabled");
                    }
                    assert_eq!(
                        values
                            .iter()
                            .map(|value| value["name"].as_str().unwrap())
                            .collect::<Vec<_>>(),
                        expected
                    );
                    let patch = parse_settings(ModuleKind::FastBreak, &tree).unwrap();
                    assert!(patch.issues.is_empty(), "{:?}", patch.issues);
                    let mut restored = FastBreak::default();
                    for change in patch.changes {
                        if let SettingChange::FastBreak(value) = change {
                            value.apply(&mut restored);
                        }
                    }
                    assert_eq!(restored, state);
                    if format == JsonFormat::Interop {
                        assert_eq!(values[2]["range"], json!({"from":0.1,"to":1.0}));
                        assert_eq!(values[4]["choices"].as_object().unwrap().len(), 2);
                    }
                }
            }
        }
        let tree = json!({"name":"FastBreak","value":[{"name":"BreakDamage","value":"bad"},{"name":"OnlyTool","value":true},
            {"name":"Mode","active":"AbortAnother","value":[],"choices":{"None":{"name":"None","value":[]},"Grim-2.3.48":{"name":"Grim-2.3.48","value":[]}}}]});
        let patch = parse_settings(ModuleKind::FastBreak, &tree).unwrap();
        assert_eq!(patch.issues.len(), 1);
        assert!(patch.changes.iter().any(|value| matches!(
            value,
            SettingChange::FastBreak(FastBreakSetting::Mode(Mode::Grim2348))
        )));
        assert!(patch.changes.iter().any(|value| matches!(
            value,
            SettingChange::FastBreak(FastBreakSetting::OnlyTool(true))
        )));
    }
}
