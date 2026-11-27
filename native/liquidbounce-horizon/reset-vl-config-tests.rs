#[test]
fn declaration_order_ranges_common_values_and_raw_roundtrip() {
    let common = CommonSettings {
        enabled: true,
        hidden: true,
        key_bind: json!({"key":"key.keyboard.r","action":"Toggle"}),
    };
    for times in [1, 10, 50, 0, -1, 51, i32::MAX] {
        for timer in [0.1_f32, 2.0, 10.0, -0.0, -2.5, 12.5] {
            for format in [JsonFormat::File, JsonFormat::Interop] {
                let mut source = reset_vl::ResetVl::default();
                source.settings.times = times;
                source.settings.timer = timer;
                let tree = serialize_reset_vl(&source, &common, format, &str::to_owned).unwrap();
                let names: Vec<_> = tree["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["name"].as_str().unwrap())
                    .collect();
                assert_eq!(
                    names,
                    if format == JsonFormat::File {
                        vec!["Enabled", "Bind", "Hidden", "Times", "Timer"]
                    } else {
                        vec!["Bind", "Hidden", "Times", "Timer"]
                    }
                );
                let root = nodes(&tree, "ResetVL", "ResetVL").unwrap();
                assert_eq!(
                    root["Times"]["range"],
                    if format == JsonFormat::Interop {
                        json!({"from":1,"to":50})
                    } else {
                        Value::Null
                    }
                );
                assert_eq!(
                    root["Timer"]["range"],
                    if format == JsonFormat::Interop {
                        json!({"from":0.1,"to":10.0})
                    } else {
                        Value::Null
                    }
                );
                if format == JsonFormat::Interop {
                    assert_eq!(root["Timer"]["suffix"], "");
                    assert_eq!(
                        root["Timer"]["description"],
                        "liquidbounce.module.resetVL.timer.description"
                    );
                }
                let mut patch = SettingsPatch::default();
                parse_reset_vl(&mut patch, &root);
                assert!(patch.issues.is_empty(), "{:?}", patch.issues);
                let mut restored = reset_vl::ResetVl::default();
                for SettingChange::ResetVl(change) in patch.changes {
                    change.apply(&mut restored);
                }
                assert_eq!(restored.settings.times, times);
                assert_eq!(restored.settings.timer.to_bits(), timer.to_bits());
            }
        }
    }
}
#[test]
fn independent_leaf_errors_and_gson_numeric_semantics() {
    for (times, timer, want_times, want_timer, issues) in [
        (json!("-7"), json!(" 12.5f "), -7, 12.5_f32, 0),
        (json!({}), json!(4), 10, 4.0, 1),
        (json!(2.9), json!(false), 2, 2.0, 1),
        (json!(4294967297_u64), json!("-0.0"), 1, -0.0, 0),
    ] {
        let tree = json!({"name":"ResetVL","value":[{"name":"Timer","value":timer},{"name":"Times","value":times}]});
        let mut patch = SettingsPatch::default();
        parse_reset_vl(&mut patch, &nodes(&tree, "ResetVL", "ResetVL").unwrap());
        assert_eq!(patch.issues.len(), issues);
        let mut module = reset_vl::ResetVl::default();
        for SettingChange::ResetVl(change) in patch.changes {
            change.apply(&mut module);
        }
        assert_eq!(module.settings.times, want_times);
        assert_eq!(module.settings.timer.to_bits(), want_timer.to_bits());
    }
}
#[test]
fn nonfinite_exports_fail_like_original_gson() {
    for timer in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut module = reset_vl::ResetVl::default();
        module.settings.timer = timer;
        assert!(
            serialize_reset_vl(
                &module,
                &CommonSettings {
                    enabled: false,
                    hidden: false,
                    key_bind: json!({})
                },
                JsonFormat::File,
                &str::to_owned
            )
            .is_err()
        );
    }
}
