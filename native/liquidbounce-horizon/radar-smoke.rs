#[cfg(test)]
mod radar_smoke {
    use super::{config::*, radar::*, tracers::ColorMode};
    use serde_json::{Value, json};

    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            key_bind: Value::Null,
            hidden: false,
        }
    }

    fn apply(root: &Value, target: &mut Radar) -> SettingsPatch {
        let mut p = SettingsPatch::default();
        parse_radar(&mut p, &nodes(root, "Radar", "Radar").unwrap());
        for SettingChange::Radar(setting) in &p.changes {
            setting.apply(target).unwrap();
        }
        p
    }

    #[test]
    fn complete_config_preserves_inactive_modes_in_both_formats() {
        let mut original = Radar::default();
        original.pointer_mode = PointerMode::Image2;
        original.image_sizes = [13., 27.];
        original.width = 17.;
        original.height = 20.;
        original.tail_concave_size = 11.;
        original.color_mode = ColorMode::Static;
        original.color = 0x12345678;
        original.hue = vec![[0., 340.], [40., 120.]];
        original.alpha = vec![[0., 1.], [40., 0.]];
        original.show = 2;
        original.hide_teammates = true;
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let root = serialize_radar(&original, &common(), format, &str::to_owned).unwrap();
            let mut restored = Radar::default();
            let p = apply(&root, &mut restored);
            assert!(p.issues.is_empty(), "{:?}", p.issues);
            assert_eq!(restored, original);
            if format == JsonFormat::Interop {
                let root_nodes = nodes(&root, "Radar", "Radar").unwrap();
                assert_eq!(root_nodes["Show"]["canBeNone"], false);
                assert_eq!(root_nodes["Alpha"]["innerType"], "VECTOR2_F");
            }
        }
    }

    #[test]
    fn bad_leaf_does_not_discard_other_valid_settings() {
        let mut root = serialize_radar(
            &Radar::default(),
            &common(),
            JsonFormat::File,
            &str::to_owned,
        )
        .unwrap();
        let values = root["value"].as_array_mut().unwrap();
        for leaf in values {
            match leaf["name"].as_str().unwrap_or("") {
                "OnlyPlayers" => leaf["value"] = json!(true),
                "Show" => leaf["value"] = json!([]),
                "HideTeammates" => leaf["value"] = json!(true),
                _ => {}
            }
        }
        let mut restored = Radar::default();
        let p = apply(&root, &mut restored);
        assert_eq!(p.issues.len(), 1);
        assert!(restored.only_players && restored.hide_teammates);
        assert_eq!(restored.show, 3);
    }

    #[test]
    fn original_color_modes_use_current_hue_and_interpolated_alpha() {
        let mut s = Radar::default();
        s.alpha = vec![[0., 0.5], [200., 0.5]];
        s.hue = vec![[0., 0.], [100., 120.], [200., 120.]];
        assert_eq!(s.indicator_color(0., 0., 10., 20., 0.), Some(0x7fff0000));
        assert_eq!(s.indicator_color(0., 200., 10., 20., 0.), Some(0x7f00ff00));
        s.color_mode = ColorMode::Static;
        s.color = 0x12345678;
        assert_eq!(s.indicator_color(0., 200., 10., 20., 0.), Some(0x7f345678));
        s.color_mode = ColorMode::Health;
        s.health_alpha = 0;
        assert_eq!(s.indicator_color(0., 0., 20., 20., 0.), Some(0x7f00ff00));
        s.alpha = vec![[0., 0.], [200., 0.]];
        assert_eq!(s.indicator_color(0., 0., 10., 20., 0.), None);
    }

    #[test]
    fn concavity_assignment_and_lifecycle_follow_source() {
        let mut s = Radar::default();
        s.set_concavity(80.);
        assert_eq!(s.tail_concave_size, 10.);
        s.height = 4.;
        assert_eq!(s.tail_concave_size, 10.);
        s.set_concavity(3.);
        assert_eq!(s.tail_concave_size, 3.);
        s.selected.insert(23);
        s.perspective = Some(2);
        s.clear();
        assert!(s.selected.is_empty());
        assert_eq!(s.perspective, None);
        assert_eq!(s.tail_concave_size, 3.);
    }
}
