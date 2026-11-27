#[test]
fn seeded_layouts_match_real_jdk_splittable_random() {
    let vectors: Vec<Vec<[i32; 2]>> = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("oracle.json"),
        )
        .unwrap(),
    )
    .unwrap();
    for (i, expected) in vectors.into_iter().enumerate() {
        assert_eq!(
            stronghold_finder::generate_single(0x51f15eed + i as u64).as_slice(),
            expected.as_slice()
        );
    }
}
#[test]
fn posterior_tracks_nearest_target_and_same_stronghold_guard() {
    use stronghold_finder::*;
    let mut a = [[1000, 1000]; 128];
    a[0] = [0, 10];
    a[1] = [0, -10];
    let mut b = a;
    b[0] = [10, 0];
    b[1] = [-10, 0];
    let measurement = Measurement {
        position: [8., 64., 8.],
        angle: 0.,
        tick: 4,
    };
    let p = estimate(&[measurement], &[a, b], 0.03, true, 3).unwrap();
    assert_eq!(p.candidates[0].chunk, [0, 10]);
    assert!(p.confidence > 0.999);
    let opposite = Measurement {
        position: [8., 64., -152.],
        angle: 0.,
        tick: 5,
    };
    assert!(estimate(&[measurement, opposite], &[a], 0.03, true, 3).is_none());
    assert!(estimate(&[measurement, opposite], &[a], 0.03, false, 3).is_some());
}
#[test]
fn consumed_throw_spawn_delay_speed_sample_and_reset() {
    use stronghold_finder::*;
    let mut s = StrongholdFinder::default();
    s.settings.hypothesis_count = 2;
    s.used([0., 64., 0.], 10, 0);
    s.spawn(7, [1., 65., 0.], 10, 0);
    let mut eye = Eye {
        id: 7,
        position: [1., 65., 5.],
        velocity: [0., 0., 0.],
    };
    assert!(s.tick(11, &[eye]).is_empty());
    assert!(s.tick(12, &[eye]).is_empty());
    eye.velocity[2] = 0.1;
    assert!(matches!(s.tick(13, &[eye])[0], Notice::Sample(1)));
    assert!(s.tick(14, &[eye]).is_empty());
    assert_eq!(s.measurements.len(), 1);
    s.settings.reset_on_world_change = false;
    s.world_change();
    assert_eq!(s.measurements.len(), 1);
    s.settings.reset_on_world_change = true;
    s.world_change();
    assert!(s.measurements.is_empty());
    s.used([0., 0., 0.], 0, 0);
    s.spawn(9, [0., 0., 0.], 21, 0);
    assert!(s.tick(24, &[Eye { id: 9, ..eye }]).is_empty());
    s.block([-1, 30, -1], "minecraft:end_portal_frame");
    s.block([0, 30, 0], "minecraft:end_portal");
    s.forget([-1, -1]);
    assert_eq!(s.portals.len(), 1);
    s.block([0, 30, 0], "minecraft:air");
    assert!(s.portals.is_empty());
}
#[test]
fn complete_file_and_interop_tree_roundtrips_and_bad_leaf_isolated() {
    use config::*;
    for format in [JsonFormat::File, JsonFormat::Interop] {
        let mut original = stronghold_finder::StrongholdFinder::default();
        original.settings.sigma = 0.1;
        original.settings.hypothesis_count = 2000;
        original.settings.render_top = false;
        original.settings.reset_on_world_change = false;
        let tree = serialize_stronghold_finder(
            &original,
            &CommonSettings {
                enabled: true,
                hidden: false,
                key_bind: serde_json::Value::Null,
            },
            format,
            &str::to_owned,
        )
        .unwrap();
        assert_eq!(
            tree["value"].as_array().unwrap().len(),
            if format == JsonFormat::File { 16 } else { 15 }
        );
        let root = nodes(&tree, "StrongholdFinder", "StrongholdFinder").unwrap();
        let mut patch = SettingsPatch::default();
        parse_stronghold_finder(&mut patch, &root);
        assert!(patch.issues.is_empty(), "{:?}", patch.issues);
        assert_eq!(patch.changes.len(), 13);
        let mut restored = stronghold_finder::StrongholdFinder::default();
        for SettingChange::StrongholdFinder(s) in patch.changes {
            s.apply(&mut restored).unwrap();
        }
        assert_eq!(original.settings, restored.settings);
        let mut bad = tree;
        let leaf = bad["value"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|v| v["name"] == "Sigma")
            .unwrap();
        leaf["value"] = serde_json::json!("invalid-angle");
        let root = nodes(&bad, "StrongholdFinder", "StrongholdFinder").unwrap();
        let mut patch = SettingsPatch::default();
        parse_stronghold_finder(&mut patch, &root);
        assert_eq!(patch.issues.len(), 1);
        assert_eq!(patch.changes.len(), 12);
    }
}
