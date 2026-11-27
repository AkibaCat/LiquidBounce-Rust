#[cfg(test)]
mod auto_crafter_config_tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Toggle"}),
        }
    }
    #[test]
    fn file_and_interop_keep_order_duplicates_and_live_options() {
        let mut state = Settings::default();
        state.items = vec![
            "minecraft:deepslate_tiles".into(),
            "minecraft:polished_deepslate".into(),
            "minecraft:deepslate_tiles".into(),
        ];
        state.delay = [7, 11];
        state.stack = false;
        state.sequential = false;
        state.on_full = OnFull::Throw;
        state.allowed = 31;
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let v = serialize_auto_crafter(&state, &common(), format, &str::to_owned).unwrap();
            let nodes = nodes(&v, "AutoCrafter", "AutoCrafter").unwrap();
            let mut patch = SettingsPatch::default();
            settings::parse_auto_crafter(&mut patch, &nodes);
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            let mut restored = Settings::default();
            for SettingChange::AutoCrafter(setting) in patch.changes {
                setting.apply(&mut restored, &state.items).unwrap();
            }
            assert_eq!(restored, state);
            if format == JsonFormat::Interop {
                assert_eq!(nodes["ItemsToCraft"]["valueType"], "REGISTRY_MUTABLE_LIST");
                assert_eq!(nodes["ItemsToCraft"]["registry"], "item");
                assert_eq!(nodes["ItemsToCraft"]["innerValueType"], "ITEM");
                assert_eq!(
                    nodes["OnFull"]["choices"],
                    json!(["Disable", "CloseScreen", "Wait", "Throw"])
                );
                assert_eq!(nodes["Delay"]["range"], json!({"from":1,"to":20}));
            }
        }
    }
    #[test]
    fn original_primitive_choices_and_leaf_failure_isolation() {
        let mut state = Settings::default();
        AutoCrafterSetting("OnFull".into(), json!("closescreen"))
            .apply(&mut state, &[])
            .unwrap();
        assert_eq!(state.on_full, OnFull::CloseScreen);
        AutoCrafterSetting(
            "AllowedContainers".into(),
            json!(["furnace", "UNKNOWN", true, "inventory"]),
        )
        .apply(&mut state, &[])
        .unwrap();
        assert_eq!(state.allowed, 5);
        AutoCrafterSetting("ItemsToCraft".into(), json!(["stone", "missing", ":stone"]))
            .apply(&mut state, &["minecraft:stone".into()])
            .unwrap();
        assert_eq!(
            state.items,
            ["minecraft:stone", "minecraft:air", "minecraft:stone"]
        );
        let v = json!({"name":"AutoCrafter","value":[{"name":"Delay","value":{"from":9,"to":2}},{"name":"OnFull","value":"bad"},{"name":"StackCrafting","value":false}]});
        let mut patch = SettingsPatch::default();
        settings::parse_auto_crafter(
            &mut patch,
            &nodes(&v, "AutoCrafter", "AutoCrafter").unwrap(),
        );
        assert_eq!(patch.issues.len(), 2);
        assert_eq!(patch.changes.len(), 1);
        for SettingChange::AutoCrafter(setting) in patch.changes {
            setting.apply(&mut state, &[]).unwrap();
        }
        assert!(!state.stack);
    }
}
