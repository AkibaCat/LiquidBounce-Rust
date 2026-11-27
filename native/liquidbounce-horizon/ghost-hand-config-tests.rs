#[test]
fn complete_registry_tree_roundtrips_both_original_formats() {
    let common = CommonSettings {
        enabled: true,
        hidden: false,
        key_bind: json!(null),
    };
    let source = crate::ghost_hand::GhostHand::default();
    let registry: Vec<_> = crate::tests::oracle()["registry"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    for format in [JsonFormat::Interop, JsonFormat::File] {
        let value =
            ghost::serialize_ghost_hand(&source, &common, format, &|key| format!("中文:{key}"))
                .unwrap();
        assert_eq!(value["name"], "GhostHand");
        let values = value["value"].as_array().unwrap();
        let targets = values
            .iter()
            .find(|v| v["name"] == "TargetedBlocks")
            .unwrap();
        assert_eq!(targets["value"].as_array().unwrap().len(), 20);
        if format == JsonFormat::Interop {
            assert_eq!(targets["valueType"], "REGISTRY_LIST");
            assert_eq!(targets["registry"], "block");
            assert_eq!(
                targets["description"],
                "中文:liquidbounce.module.ghostHand.targetedBlocks.description"
            );
            assert!(!values.iter().any(|v| v["name"] == "Enabled"));
        } else {
            assert!(targets.get("registry").is_none());
        }
        let nodes = nodes(&value, "GhostHand", "GhostHand").unwrap();
        let mut patch = SettingsPatch::default();
        ghost::parse_ghost_hand(&mut patch, &nodes);
        assert!(patch.issues.is_empty());
        assert_eq!(patch.changes.len(), 1);
        let mut restored = crate::ghost_hand::GhostHand::default();
        restored.targeted_blocks.clear();
        let SettingChange::GhostHand(change) = patch.changes.pop().unwrap();
        change.apply(&mut restored, &registry).unwrap();
        assert_eq!(source, restored);
    }
}

#[test]
fn malformed_registry_value_fails_at_application_and_preserves_old_set() {
    let input =
        json!({"name":"GhostHand","value":[{"name":"TargetedBlocks","value":["chest",null]}]});
    let root = nodes(&input, "GhostHand", "GhostHand").unwrap();
    let mut patch = SettingsPatch::default();
    ghost::parse_ghost_hand(&mut patch, &root);
    assert!(patch.issues.is_empty());
    let mut state = crate::ghost_hand::GhostHand::default();
    let old = state.clone();
    let SettingChange::GhostHand(change) = patch.changes.pop().unwrap();
    assert!(
        change
            .apply(
                &mut state,
                &["minecraft:air".into(), "minecraft:chest".into()]
            )
            .is_err()
    );
    assert_eq!(state, old);
}
