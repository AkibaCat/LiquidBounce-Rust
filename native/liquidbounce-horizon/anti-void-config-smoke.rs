// SPDX-License-Identifier: GPL-3.0-or-later
#[cfg(test)]
mod anti_void_config_smoke {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        }
    }
    fn find<'a>(v: &'a Value, name: &str) -> &'a Value {
        v["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["name"] == name)
            .unwrap()
    }
    #[test]
    fn anti_void_smoke_config_all_eight_settings_file_and_interop() {
        let mut original = anti_void::State::default();
        original.settings.mode = 3;
        original.settings.void_level = -64;
        original.settings.height = 1.25;
        original.settings.silent = true;
        original.settings.slot_reset = [0, 40];
        let registry = vec!["minecraft:magma_cream".into(), "minecraft:air".into()];
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let tree =
                anti_void_config::serialize_anti_void(&original, &common(), format, &str::to_owned)
                    .unwrap();
            let mut patch = SettingsPatch::default();
            anti_void_config::parse_anti_void(
                &mut patch,
                &nodes(&tree, "AntiVoid", "AntiVoid").unwrap(),
            );
            assert!(patch.issues.is_empty());
            assert_eq!(patch.changes.len(), 8);
            let mut restored = anti_void::State::default();
            for SettingChange::AntiVoid(change) in patch.changes {
                change.apply(&mut restored, &registry).unwrap();
            }
            assert_eq!(restored.settings, original.settings);
            let mode = find(&tree, "Mode");
            assert_eq!(mode["choices"].as_object().unwrap().len(), 4);
            if format == JsonFormat::Interop {
                assert_eq!(
                    find(&mode["choices"]["UseItem"], "Items")["registry"],
                    "item"
                );
                assert_eq!(find(&tree, "VoidLevel")["range"]["from"], -256);
            }
        }
    }
    #[test]
    fn anti_void_smoke_config_atomic_registry_and_exact_mode() {
        let mut state = anti_void::State::default();
        let old = state.settings.items.clone();
        let apply = |s: &mut anti_void::State, p: &str, v: Value| {
            anti_void_config::AntiVoidSetting(p.into(), v)
                .apply(s, &["minecraft:air".into(), "minecraft:magma_cream".into()])
        };
        assert!(apply(&mut state, "Mode.UseItem.Items", json!(["magma_cream", {}])).is_err());
        assert_eq!(state.settings.items, old);
        apply(
            &mut state,
            "Mode.UseItem.Items",
            json!(["missing", 7, "magma_cream", "magma_cream"]),
        )
        .unwrap();
        assert_eq!(state.settings.items.len(), 2);
        assert!(state.settings.items.contains("minecraft:air"));
        apply(&mut state, "Mode.UseItem.Items", json!([])).unwrap();
        assert!(state.settings.items.is_empty());
        assert!(apply(&mut state, "Mode", json!("ghostblock")).is_err());
        assert!(
            apply(
                &mut state,
                "Mode.UseItem.SlotResetDelay",
                json!({"from":9,"to":2})
            )
            .is_err()
        );
        apply(&mut state, "Mode.Flag.Height", json!("NaN")).unwrap();
        assert!(
            anti_void_config::serialize_anti_void(
                &state,
                &common(),
                JsonFormat::File,
                &str::to_owned
            )
            .is_err()
        );
    }
}
