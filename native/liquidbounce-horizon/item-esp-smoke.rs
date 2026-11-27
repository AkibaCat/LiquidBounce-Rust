#[cfg(test)]
mod item_esp_smoke {
    use super::*;
    use item_esp::{ItemEsp, Kind, Mode, merge_boxes};
    use serde_json::{Value, json};
    #[test]
    fn item_filter_and_eye_distance_follow_source() {
        let mut s = ItemEsp::default();
        assert!(s.should_render(Kind::Item("minecraft:stone"), [128., 0., 0.], [0.; 3]));
        assert!(!s.should_render(Kind::Item("minecraft:stone"), [128., 0.1, 0.], [0.; 3]));
        s.items.insert("minecraft:stone".into());
        assert!(!s.should_render(Kind::Item("minecraft:stone"), [0.; 3], [0.; 3]));
        s.whitelist = true;
        assert!(s.should_render(Kind::Item("minecraft:stone"), [0.; 3], [0.; 3]));
        assert!(!s.should_render(Kind::Item("minecraft:dirt"), [0.; 3], [0.; 3]));
    }
    #[test]
    fn arrow_subtypes_pickup_and_trident_are_independent() {
        let mut s = ItemEsp::default();
        for allowed in [false, true] {
            assert_eq!(
                s.should_render(
                    Kind::Arrow {
                        allowed,
                        effect_color: -1
                    },
                    [0.; 3],
                    [0.; 3]
                ),
                allowed
            );
            assert_eq!(
                s.should_render(Kind::Spectral { allowed }, [0.; 3], [0.; 3]),
                allowed
            );
        }
        s.regular_arrows = false;
        assert!(!s.should_render(
            Kind::Arrow {
                allowed: true,
                effect_color: -1
            },
            [0.; 3],
            [0.; 3]
        ));
        assert!(s.should_render(
            Kind::Arrow {
                allowed: true,
                effect_color: 0
            },
            [0.; 3],
            [0.; 3]
        ));
        s.arrows = false;
        assert!(!s.should_render(
            Kind::Arrow {
                allowed: true,
                effect_color: 0
            },
            [0.; 3],
            [0.; 3]
        ));
        assert!(s.should_render(Kind::Trident, [0.; 3], [0.; 3]));
    }
    #[test]
    fn merging_uses_strict_original_intersections_and_stable_input_order() {
        let boxes = [
            [0., 0., 0., 2., 1., 1.],
            [1., 0., 0., 2., 2., 1.],
            [0., 1., 0., 0.5, 2., 1.],
            [2., 0., 0., 3., 1., 1.],
        ];
        assert_eq!(
            merge_boxes(&boxes),
            vec![[0., 0., 0., 2., 2., 1.], boxes[2], boxes[3]]
        );
        assert!(merge_boxes(&[]).is_empty());
        let b = item_esp::item_box([1., 2., 3.]);
        assert_eq!(b, [0.875, 2.125, 2.875, 1.125, 2.375, 3.125]);
    }
    #[test]
    fn transitive_merges_do_not_require_pairwise_complete_overlap() {
        let a = [
            [0., 0., 0., 1., 1., 1.],
            [1.8, 0., 0., 2.8, 1., 1.],
            [0.9, 0., 0., 1.9, 1., 1.],
        ];
        assert_eq!(merge_boxes(&a), vec![[0., 0., 0., 2.8, 1., 1.]]);
    }
    #[test]
    fn legacy_marker_keeps_all_original_rectangles_and_background_passes() {
        let q = item_esp::legacy_marker([2., 3., 4.], [0.; 3], 0., 0.5, 0.1);
        assert_eq!(q.len(), 16);
        assert_eq!(q.iter().filter(|v| v.1).count(), 8);
        assert!((q[0].0[0][0] - 2.4).abs() < 1e-6);
        assert!((q[0].0[0][1] - 2.8).abs() < 1e-6);
        assert_eq!(q[0].0[0][2], 4.);
        assert_eq!(item_esp::TOP_BACKGROUND[3], [7., 0.5, 7.3, 3.3]);
        assert_eq!(item_esp::BOTTOM_FOREGROUND[2], [6., -20., 7., -17.5]);
    }
    #[test]
    fn defaults_and_all_glow_styles_preserve_original_alpha() {
        let mut s = ItemEsp::default();
        assert_eq!(s.mode, Mode::Glow);
        assert_eq!(s.color, 0xffffb348u32 as i32);
        for i in 0..9 {
            assert_eq!(s.styles[i].get("Fill"), storage_esp::FILLS[i]);
            s.style = i + 1;
            assert_eq!(s.effect().style, i);
        }
        s.style = 0;
        assert_eq!(s.effect().style, 9);
        s.rainbow = true;
        assert_eq!(s.color_at(0.)[3], 50. / 255.);
    }
    #[test]
    fn complete_config_roundtrips_all_inactive_modes_and_styles() {
        use config::*;
        let items = vec!["minecraft:stone".into(), "minecraft:dirt".into()];
        let mut s = ItemEsp::default();
        for (p, v) in [
            ("Items", json!(["stone", "dirt", "stone"])),
            ("Filter", json!("whitelist")),
            ("Mode", json!("Legacy2D")),
            ("Mode.Box.MergeIntersecting", json!(true)),
            ("Mode.Legacy2D.Scale", json!(0.25)),
            ("Mode.Legacy2D.YOffset", json!(-0.5)),
            ("ColorMode", json!("Rainbow")),
            ("ColorMode.Static.Color", json!("#13579b")),
            ("Mode.Glow.Style", json!("Neon")),
            ("Mode.Glow.Style.Neon.CoreMix", json!(0.9)),
        ] {
            ItemEspSetting(p.into(), v).apply(&mut s, &items).unwrap();
        }
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let v = serialize_item_esp(
                &s,
                &CommonSettings {
                    enabled: true,
                    hidden: false,
                    key_bind: Value::Null,
                },
                format,
                &str::to_owned,
            )
            .unwrap();
            let n = nodes(&v, "ItemESP", "ItemESP").unwrap();
            let mut patch = SettingsPatch::default();
            parse_item_esp(&mut patch, &n);
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            let mut restored = ItemEsp::default();
            for SettingChange::ItemEsp(change) in patch.changes {
                change.apply(&mut restored, &items).unwrap();
            }
            assert_eq!(s, restored);
        }
    }
    #[test]
    fn malformed_leaf_does_not_discard_valid_siblings() {
        use config::*;
        let mut v = serialize_item_esp(
            &ItemEsp::default(),
            &CommonSettings {
                enabled: false,
                hidden: false,
                key_bind: Value::Null,
            },
            JsonFormat::File,
            &str::to_owned,
        )
        .unwrap();
        for field in v["value"].as_array_mut().unwrap() {
            if field["name"] == "MaximumDistance" {
                field["value"] = json!({"bad":true});
            }
            if field["name"] == "ShowTridents" {
                field["value"] = json!(false);
            }
        }
        let n = nodes(&v, "ItemESP", "ItemESP").unwrap();
        let mut p = SettingsPatch::default();
        parse_item_esp(&mut p, &n);
        assert_eq!(p.issues.len(), 1);
        let mut s = ItemEsp::default();
        for SettingChange::ItemEsp(change) in p.changes {
            change.apply(&mut s, &[]).unwrap();
        }
        assert!(!s.tridents);
        assert_eq!(s.maximum_distance, 128.);
    }
}
