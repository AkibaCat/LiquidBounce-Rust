// SPDX-License-Identifier: GPL-3.0-or-later
//! All three original settings; imported values retain their raw Float range.
use super::*;
use liquidbounce_behaviors::strafe::Strafe;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrafeSetting {
    StrengthInAir(f32),
    StrengthOnGround(f32),
    StrictMovement(bool),
}
impl StrafeSetting {
    pub fn apply(self, state: &mut Strafe) {
        match self {
            Self::StrengthInAir(value) => state.strength_in_air = value,
            Self::StrengthOnGround(value) => state.strength_on_ground = value,
            Self::StrictMovement(value) => state.strict_movement = value,
        }
    }
}
pub fn serialize_strafe(
    state: &Strafe,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Strafe, common);
    for (name, key, value) in [
        ("StrengthInAir", "strengthInAir", state.strength_in_air),
        (
            "StrengthOnGround",
            "strengthOnGround",
            state.strength_on_ground,
        ),
    ] {
        values.push(tree.ranged(
            name,
            "FLOAT",
            float_json(value, &format!("Strafe.{name}"))?,
            json!({"from":0.0,"to":1.0}),
            "",
            &format!("liquidbounce.module.strafe.{key}"),
        ));
    }
    values.push(tree.leaf(
        "StrictMovement",
        "BOOLEAN",
        json!(state.strict_movement),
        "liquidbounce.module.strafe.strictMovement",
        None,
    ));
    Ok(tree.group("Strafe", "TOGGLEABLE", values))
}
pub(super) fn parse_strafe(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "StrengthInAir",
        "Strafe.StrengthInAir",
        |value, path| {
            parse_float(value, path).map(|v| SettingChange::Strafe(StrafeSetting::StrengthInAir(v)))
        },
    );
    leaf(
        patch,
        root,
        "StrengthOnGround",
        "Strafe.StrengthOnGround",
        |value, path| {
            parse_float(value, path)
                .map(|v| SettingChange::Strafe(StrafeSetting::StrengthOnGround(v)))
        },
    );
    leaf(
        patch,
        root,
        "StrictMovement",
        "Strafe.StrictMovement",
        |value, path| {
            parse_bool(value, path).map(|v| SettingChange::Strafe(StrafeSetting::StrictMovement(v)))
        },
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_tree_roundtrips_both_formats_and_imports_unclamped_strengths() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        let original = Strafe::default();
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_strafe(&original, &common, format, &str::to_owned).unwrap();
            let leaves = tree["value"].as_array().unwrap();
            let mut names=vec!["Bind","Hidden","StrengthInAir","StrengthOnGround","StrictMovement"];
            if format==JsonFormat::File {names.insert(0,"Enabled");}
            assert_eq!(leaves.iter().map(|v|v["name"].as_str().unwrap()).collect::<Vec<_>>(),names);
            if format == JsonFormat::Interop {
                assert_eq!(leaves[2]["range"], json!({"from":0.0,"to":1.0}));
            }
            let patch = parse_settings(ModuleKind::Strafe, &tree).unwrap();
            assert!(patch.issues.is_empty());
            let mut state = Strafe {
                strength_in_air: 0.0,
                strength_on_ground: 0.0,
                strict_movement: true,
            };
            for change in patch.changes {
                if let SettingChange::Strafe(value) = change {
                    value.apply(&mut state);
                }
            }
            assert_eq!(state, original);
        }
        let patch = parse_settings(
            ModuleKind::Strafe,
            &json!({"name":"Strafe","value":[
            {"name":"StrengthInAir","value":-4.5},{"name":"StrengthOnGround","value":"bad"},
            {"name":"StrictMovement","value":true}]}),
        )
        .unwrap();
        assert_eq!(patch.issues.len(), 1);
        let mut state = original;
        for change in patch.changes {
            if let SettingChange::Strafe(value) = change {
                value.apply(&mut state);
            }
        }
        assert_eq!(
            state,
            Strafe {
                strength_in_air: -4.5,
                strength_on_ground: 1.0,
                strict_movement: true
            }
        );
    }
}
