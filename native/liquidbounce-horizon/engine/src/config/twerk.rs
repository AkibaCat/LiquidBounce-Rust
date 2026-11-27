// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTwerk's sole original Delay value, ae0eb396.
use super::*;
use liquidbounce_behaviors::twerk::Twerk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TwerkSetting {
    Delay(i32),
}

impl TwerkSetting {
    pub fn apply(self, state: &mut Twerk) {
        match self {
            Self::Delay(value) => state.delay = value,
        }
    }
}

pub fn serialize_twerk(
    state: &Twerk,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Twerk, common);
    values.push(tree.ranged(
        "Delay",
        "INT",
        json!(state.delay),
        json!({"from":1,"to":20}),
        "",
        "liquidbounce.module.twerk.delay",
    ));
    Ok(tree.group("Twerk", "TOGGLEABLE", values))
}

pub(super) fn parse_twerk(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Delay", "Twerk.Delay", |value, path| {
        parse_int(value, path).map(|value| SettingChange::Twerk(TwerkSetting::Delay(value)))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn twerk_delay_is_raw_and_tree_has_only_original_options() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        let interop = serialize_twerk(
            &Twerk::default(),
            &common,
            JsonFormat::Interop,
            &str::to_owned,
        )
        .unwrap();
        let values = interop["value"].as_array().unwrap();
        assert_eq!(
            values
                .iter()
                .map(|value| value["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["Bind", "Hidden", "Delay"]
        );
        assert_eq!(values[2]["value"], 2);
        assert_eq!(values[2]["range"], json!({"from":1,"to":20}));
        assert_eq!(values[2]["suffix"], "");
        assert_eq!(values[2]["key"], "liquidbounce.module.twerk.delay");
        assert_eq!(
            values[2]["description"],
            "liquidbounce.module.twerk.delay.description"
        );
        for value in [json!(0), json!(i32::MIN), json!(i32::MAX), json!("-900")] {
            let file = json!({"name":"Twerk","value":[{"name":"Delay","value":value},{"name":"Delay","value":1}]});
            let patch = parse_settings(ModuleKind::Twerk, &file).unwrap();
            assert!(patch.issues.is_empty());
            assert_eq!(patch.changes.len(), 1, "only first duplicate is consumed");
            let mut state = Twerk::default();
            if let SettingChange::Twerk(setting) = patch.changes[0] {
                setting.apply(&mut state)
            } else {
                panic!("wrong setting")
            };
            let stored =
                serialize_twerk(&state, &common, JsonFormat::File, &str::to_owned).unwrap();
            let delay = stored["value"]
                .as_array()
                .unwrap()
                .iter()
                .find(|value| value["name"] == "Delay")
                .unwrap();
            assert_eq!(delay["value"], json!(state.delay));
        }
    }

    #[test]
    fn twerk_integer_adapter_keeps_gson_conversions_and_leaf_failures() {
        for (value, expected) in [
            (json!(2147483648_i64), i32::MIN),
            (json!(4294967297_i64), 1),
            (json!(20.9), 20),
            (json!(-20.9), -20),
            (json!("+21"), 21),
            (json!(-1), -1),
        ] {
            let patch = parse_settings(
                ModuleKind::Twerk,
                &json!({"name":"Twerk","value":[{"name":"Delay","value":value}]}),
            )
            .unwrap();
            assert!(patch.issues.is_empty());
            assert_eq!(
                patch.changes,
                [SettingChange::Twerk(TwerkSetting::Delay(expected))]
            );
        }
        for value in [Value::Null, json!(true), json!([]), json!({}), json!("2.0")] {
            let patch = parse_settings(
                ModuleKind::Twerk,
                &json!({"name":"Twerk","value":[
                    {"name":"Delay","value":value},
                    {"name":"Hidden","value":true},
                    {"name":"Enabled","value":false}
                ]}),
            )
            .unwrap();
            assert_eq!(
                patch.changes,
                [SettingChange::Enabled(false), SettingChange::Hidden(true)]
            );
            assert_eq!(patch.issues.len(), 1);
            assert_eq!(patch.issues[0].path, "Twerk.Delay");
        }
        assert!(parse_settings(ModuleKind::Twerk, &json!({"name":"twerk","value":[]})).is_err());
    }

    #[test]
    fn twerk_file_order_category_and_runtime_forwarding() {
        assert_eq!(ModuleKind::Twerk.category(), "Fun");
        assert_eq!(ModuleKind::Twerk.key(), "liquidbounce.module.twerk");
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        let file =
            serialize_twerk(&Twerk::default(), &common, JsonFormat::File, &str::to_owned).unwrap();
        assert_eq!(
            file,
            json!({"name":"Twerk","value":[
                {"name":"Enabled","value":true},
                {"name":"Bind","value":common.key_bind},
                {"name":"Hidden","value":false},
                {"name":"Delay","value":2}
            ]})
        );
        let patch = parse_settings(ModuleKind::Twerk, &file).unwrap();
        assert!(patch.issues.is_empty());
        assert_eq!(
            patch.changes.last(),
            Some(&SettingChange::Twerk(TwerkSetting::Delay(2)))
        );
        assert_eq!(
            apply_setting(
                &SettingChange::Twerk(TwerkSetting::Delay(0)),
                &mut NoFov::default(),
                &mut Zoom::default(),
                &mut FullBright::default(),
            ),
            ApplyResult::Runtime(RuntimeRequest::Twerk(TwerkSetting::Delay(0)))
        );
        assert!(
            serialize_camera_effect(
                ModuleKind::Twerk,
                &common,
                JsonFormat::Interop,
                &str::to_owned,
            )
            .is_err()
        );
    }
}
