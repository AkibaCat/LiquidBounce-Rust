#[test]
fn complete_nested_settings_roundtrip_and_original_order() {
    let common = CommonSettings {
        enabled: true,
        hidden: false,
        key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
    };
    for flags in 0..8 {
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let original = anti_hunger::AntiHunger {
                settings: anti_hunger::Settings {
                    no_sprint: flags & 1 != 0,
                    while_swimming: flags & 2 != 0,
                    keep_floating: flags & 4 != 0,
                },
                last_notification: 98765,
            };
            let tree = serialize_anti_hunger(&original, &common, format, &str::to_owned).unwrap();
            let n = nodes(&tree, "AntiHunger", "AntiHunger").unwrap();
            let group = nodes(n["NoSprint"], "NoSprint", "AntiHunger.NoSprint").unwrap();
            assert_eq!(group["WhileSwimming"]["value"], json!(flags & 2 != 0));
            let mut patch = SettingsPatch::default();
            parse_anti_hunger(&mut patch, &n);
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            assert!(matches!(
                patch.changes.as_slice(),
                [
                    SettingChange::AntiHunger(AntiHungerSetting::NoSprint(_)),
                    SettingChange::AntiHunger(AntiHungerSetting::WhileSwimming(_)),
                    SettingChange::AntiHunger(AntiHungerSetting::KeepFloating(_))
                ]
            ));
            let mut restored = anti_hunger::AntiHunger {
                last_notification: 123,
                ..Default::default()
            };
            for SettingChange::AntiHunger(v) in patch.changes {
                v.apply(&mut restored);
            }
            assert_eq!(restored.settings, original.settings);
            assert_eq!(restored.last_notification, 123);
        }
    }
}
#[test]
fn malformed_group_keeps_independent_later_value_progress() {
    let root = json!({"name":"AntiHunger","value":[{"name":"NoSprint","value":false},{"name":"KeepFloating","value":false}]});
    let mut patch = SettingsPatch::default();
    parse_anti_hunger(
        &mut patch,
        &nodes(&root, "AntiHunger", "AntiHunger").unwrap(),
    );
    assert_eq!(patch.issues.len(), 1);
    assert_eq!(
        patch.changes,
        vec![SettingChange::AntiHunger(AntiHungerSetting::KeepFloating(
            false
        ))]
    );
}
