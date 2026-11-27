// SPDX-License-Identifier: GPL-3.0-or-later
// Included after real shared Tree/parser helpers and the owned config adapter.
#[cfg(test)]
mod anti_bot_config_smoke {
    use super::*;
    fn find<'a>(v: &'a Value, name: &str) -> &'a Value {
        v["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == name)
            .unwrap()
    }
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Hold"}),
        }
    }
    fn tree(s: &anti_bot::State, f: JsonFormat) -> Value {
        anti_bot_config::serialize_anti_bot(s, &common(), f, &|k| k.into()).unwrap()
    }
    fn custom(v: &Value) -> &Value {
        &find(v, "Mode")["choices"]["Custom"]
    }
    #[test]
    fn anti_bot_smoke_config_full_roundtrip_and_sparse_helmet() {
        let mut original = anti_bot::State::default();
        original.settings.conditions = 0;
        original.settings.armor_choices = [(1 << 8) | (1 << 10), 1 << 7, 0, 1 << 6];
        original.settings.validators = 0;
        original.settings.radius_range = 23.5;
        original.set_mode(3);
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let v = tree(&original, format);
            let mut patch = SettingsPatch::default();
            anti_bot_config::parse_anti_bot(&mut patch, &nodes(&v, "AntiBot", "AntiBot").unwrap());
            assert!(patch.issues.is_empty());
            assert_eq!(patch.changes.len(), 23);
            let mut restored = anti_bot::State::default();
            for SettingChange::AntiBot(change) in patch.changes {
                change.apply(&mut restored).unwrap();
            }
            assert_eq!(restored.settings, original.settings);
            let helmet = find(find(custom(&v), "Armor"), "Helmet");
            assert_eq!(helmet["value"], json!(["TurtleScute", "Skull"]));
            if format == JsonFormat::Interop {
                assert_eq!(helmet["choices"].as_array().unwrap().len(), 10);
                assert!(
                    !helmet["choices"]
                        .as_array()
                        .unwrap()
                        .contains(&json!("Elytra"))
                );
                assert!(find(find(custom(&v), "Age"), "Minimum")["range"]["from"].is_i64());
                assert!(find(find(custom(&v), "Name"), "Length")["range"]["from"].is_i64());
            }
        }
    }
    #[test]
    fn anti_bot_smoke_config_unknown_choices_partial_failure_and_aliases() {
        let mut s = anti_bot::State::default();
        let apply = |s: &mut anti_bot::State, path: &str, v: Value| {
            anti_bot_config::AntiBotSetting(path.into(), v).apply(s)
        };
        apply(
            &mut s,
            "Mode.Custom.Armor.Helmet",
            json!(["Elytra", "TurtleScute", "Unknown", "Skull"]),
        )
        .unwrap();
        assert_eq!(s.settings.armor_choices[0], (1 << 8) | (1 << 10));
        assert!(apply(&mut s, "Mode.Custom.Armor.Helmet", json!(["Skull", {}])).is_err());
        assert_eq!(s.settings.armor_choices[0], 1 << 10);
        assert!(
            apply(
                &mut s,
                "Mode.Custom.Name.ValidateChars",
                json!(["Cyrillic", {}])
            )
            .is_err()
        );
        assert_eq!(s.settings.validators, 2);
        apply(&mut s, "Mode.Custom.Conditions", json!({})).unwrap();
        assert_eq!(s.settings.conditions, 0);
        apply(&mut s, "Mode", "IntaveHeavy".into()).unwrap();
        assert_eq!(s.settings.mode, 2);
        let mut value = tree(&s, JsonFormat::File);
        let mode = value["value"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["name"] == "Mode")
            .unwrap();
        mode["choices"]["Custom"]["value"]
            .as_array_mut()
            .unwrap()
            .insert(0, json!({"name":"Conditions","value":["NeedHit"]}));
        let mut patch = SettingsPatch::default();
        anti_bot_config::parse_anti_bot(&mut patch, &nodes(&value, "AntiBot", "AntiBot").unwrap());
        for SettingChange::AntiBot(change) in patch.changes {
            change.apply(&mut s).unwrap();
        }
        assert_eq!(s.settings.conditions, 1 << 5);
    }
}
