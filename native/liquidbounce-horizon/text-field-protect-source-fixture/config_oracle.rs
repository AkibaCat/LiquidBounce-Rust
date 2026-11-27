use super::*;
use liquidbounce_behaviors::text_field_protect::TextFieldProtect;

#[test]
fn original_linked_set_deserialization_and_serializers() {
    let inputs: Vec<Value> = serde_json::from_str(include_str!("inputs.json")).unwrap();
    let expected: Vec<Value> = serde_json::from_str(include_str!("expected.json")).unwrap();
    let common = CommonSettings {
        enabled: false,
        hidden: false,
        key_bind: json!({}),
    };
    for (input, expected) in inputs.iter().zip(&expected) {
        let mut state = TextFieldProtect::default();
        let patch = parse_settings(
            ModuleKind::TextFieldProtect,
            &json!({"name":"TextFieldProtect", "value":[{"name":"Patterns", "value":input}]}),
        )
        .unwrap();
        assert_eq!(
            !patch.issues.is_empty(),
            expected["failed"].as_bool().unwrap(),
            "{input}"
        );
        for change in patch.changes {
            if let SettingChange::TextFieldProtect(value) = change {
                value.apply(&mut state);
            }
        }
        for (format, field) in [(JsonFormat::File, "file"), (JsonFormat::Interop, "interop")] {
            let actual =
                serialize_text_field_protect(&state, &common, format, &str::to_owned).unwrap();
            let n = nodes(&actual, "TextFieldProtect", "TextFieldProtect").unwrap();
            let e = nodes(&expected[field], "TextFieldProtect", "TextFieldProtect").unwrap();
            assert_eq!(n["Patterns"]["value"], expected["patterns"], "{input}");
            assert_eq!(n["Patterns"]["value"], e["Patterns"]["value"]);
            assert_eq!(
                n["Patterns"].get("innerValueType"),
                e["Patterns"].get("innerValueType")
            );
            assert_eq!(
                n["Patterns"].get("valueType"),
                e["Patterns"].get("valueType")
            );
        }
    }
    assert_eq!(inputs.len(), expected.len());
}
