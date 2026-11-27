// SPDX-License-Identifier: GPL-3.0-or-later
#[cfg(test)]
mod custom_ambience_smoke {
    use crate::custom_ambience::{CustomAmbience, CustomAmbienceSetting, Particle};
    use serde_json::{Value, json};
    fn set(s: &mut CustomAmbience, p: &str, v: Value) {
        CustomAmbienceSetting(p.into(), v).apply(s)
    }
    fn simple(s: &mut CustomAmbience, p: &str, v: Value) {
        set(s, &format!("WorldParticles.Mode.Simple.{p}"), v)
    }
    fn frame(s: &mut CustomAmbience, now: i64, partial: f32) -> Value {
        s.frame(&json!({"now":now,"partial":partial,"player":[1.,2.,3.],"right":[1.,0.,0.],"up":[0.,1.,0.]}))
    }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} != {b}");
    }
    #[test]
    fn complete_schema_inactive_modes_locales_and_both_config_formats() {
        use crate::config::*;
        let mut s = CustomAmbience::default();
        assert_eq!(s.text("Weather"), "Snowy");
        assert_eq!(s.text("Time"), "Night");
        assert!(s.boolean("Fog.Enabled") && s.boolean("CustomLightmap.Enabled"));
        assert!(!s.boolean("WorldParticles.Enabled"));
        set(
            &mut s,
            "CustomLightmap.Mode.Custom.AmbientColor",
            json!(0x80112233u32 as i32),
        );
        simple(&mut s, "Wander.Speed", json!(3.));
        simple(&mut s, "Particle", json!("Snowflake"));
        simple(&mut s, "Life", json!({"from":7,"to":99}));
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.p","action":"Toggle"}),
        };
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_custom_ambience(&s, &common, format, &str::to_owned).unwrap();
            let (changes, errors) = parse_custom_ambience_value(&tree, "CustomAmbience");
            assert!(errors.is_empty(), "{errors:?}");
            assert_eq!(changes.len(), s.values.len());
            let mut restored = CustomAmbience::default();
            for c in changes {
                c.apply(&mut restored);
            }
            assert_eq!(
                serialize_custom_ambience(&restored, &common, format, &str::to_owned).unwrap(),
                tree
            );
        }
        fn walk(n: &crate::target_renderer::Node, path: &str, keys: &mut Vec<String>) {
            let path = path
                .split('.')
                .map(|p| {
                    if p.is_empty() {
                        String::new()
                    } else {
                        p[..1].to_ascii_lowercase() + &p[1..]
                    }
                })
                .collect::<Vec<_>>()
                .join(".");
            keys.push(format!(
                "liquidbounce.module.customAmbience.{}description",
                if path.is_empty() {
                    String::new()
                } else {
                    format!("{path}.")
                }
            ));
            for c in &n.children {
                walk(
                    c,
                    &if path.is_empty() {
                        c.name.into()
                    } else {
                        format!("{path}.{}", c.name)
                    },
                    keys,
                );
            }
            if n.kind == "CHOICE" {
                for c in &n.choices {
                    walk(c, &format!("{path}.{}", c.name), keys);
                }
            }
        }
        let mut expected = vec![];
        walk(&crate::custom_ambience::schema(), "", &mut expected);
        expected.sort();
        let mut actual: Vec<String> =
            serde_json::from_str(include_str!("description-keys.json")).unwrap();
        actual.sort();
        assert_eq!(actual, expected);
        assert_eq!(crate::custom_ambience::PARTICLE.len(), 11);
    }
    #[test]
    fn all_weather_time_precipitation_branches_match_native_snapshot() {
        let mut s = CustomAmbience::default();
        for (mode, expected) in [
            ("NoChange", [0.2, 0.3]),
            ("Sunny", [0., 0.]),
            ("Rainy", [1., 0.]),
            ("Snowy", [0.9, 0.]),
            ("Thunder", [1., 1.]),
        ] {
            set(&mut s, "Weather", json!(mode));
            assert_eq!(s.weather([0.2, 0.3]), expected);
            assert_eq!(
                crate::native_game::snapshot_weather(&s.settings(), (0.2, 0.3)),
                (expected[0], expected[1])
            );
            close(
                s.precipitation(1.) as f64,
                if mode == "Snowy" { 0. } else { 0.7 },
            );
            assert_eq!(s.precipitation(0.), 0.);
            let (n, snow) = crate::native_game::snapshot_precipitation(&s.settings(), 1.);
            assert_eq!(n, s.precipitation(1.));
            assert_eq!(snow, mode == "Snowy");
        }
        set(&mut s, "Weather", json!("Rainy"));
        set(&mut s, "ModifyPrecipitation.Enabled", json!(false));
        assert_eq!(s.precipitation(0.25), 0.25);
        for (mode, expected) in [
            ("NoChange", 43210),
            ("Dawn", 23041),
            ("Day", 1000),
            ("Noon", 6000),
            ("Dusk", 12610),
            ("Night", 13000),
            ("MidNight", 18000),
        ] {
            set(&mut s, "Time", json!(mode));
            assert_eq!(s.time(43210, true), expected);
            assert_eq!(s.time(43210, false), 43210);
            assert_eq!(
                crate::native_game::snapshot_time(&s.settings(), 43210, "minecraft:overworld"),
                expected
            );
            assert_eq!(
                crate::native_game::snapshot_time(&s.settings(), 43210, "minecraft:the_end"),
                43210
            );
        }
    }
    #[test]
    fn fog_lightmap_sky_actual_native_and_render_consumers_preserve_alpha_rules() {
        let game = crate::Game::default();
        let base = crate::fog::FogData {
            color: [0.1, 0.2, 0.3, 0.4],
            environmental_start: 1.,
            environmental_end: 2.,
            render_distance_start: 3.,
            render_distance_end: 4.,
            sky_end: 5.,
            cloud_end: 6.,
        };
        let mut sky = crate::environment::Sky::default();
        {
            let mut m = game.hooks.state().behaviors.borrow_mut();
            let s = &mut m.custom_ambience;
            set(s, "Fog.FogColorOverride.Color", json!(0x00112233));
            set(s, "Fog.Environmental", json!({"from":-16.,"to":0.}));
            set(s, "SkyColor.Enabled", json!(true));
            set(s, "SkyColor.Color", json!(0x00010203));
        }
        game.client_custom_ambience_refresh();
        let f = game.client_custom_ambience_fog(base);
        assert_eq!(f.color, crate::custom_ambience::rgba(0x00112233));
        assert_eq!((f.environmental_start, f.environmental_end), (-16., 0.));
        game.client_custom_ambience_sky(&mut sky);
        assert_eq!(sky.sky_color, [1. / 255., 2. / 255., 3. / 255.]);
        assert_eq!(sky.fog_color, [0.; 3]);
        {
            let mut m = game.hooks.state().behaviors.borrow_mut();
            let s = &mut m.custom_ambience;
            set(s, "Fog.DisableWorldFog", json!(true));
            set(
                s,
                "CustomLightmap.Mode.SingleColor.Color",
                json!(0x00123456),
            );
            set(s, "Fog.BackgroundColor", json!(0x00123456));
        }
        game.client_custom_ambience_refresh();
        let f = game.client_custom_ambience_fog(base);
        assert_eq!(
            [
                f.environmental_start,
                f.environmental_end,
                f.render_distance_start,
                f.render_distance_end,
                f.sky_end,
                f.cloud_end
            ],
            [f32::MAX; 6]
        );
        let settings = game
            .hooks
            .state()
            .behaviors
            .borrow()
            .custom_ambience
            .settings();
        crate::render::begin_frame(&settings.to_string());
        let mut pixels = [255u8; 16 * 16 * 4];
        crate::render::lightmap_fill(&mut pixels);
        assert!(pixels.chunks_exact(4).all(|v| v == [0x12, 0x34, 0x56, 0]));
        let original = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 0.4,
        };
        assert_eq!(crate::render::clear_color(original), original);
        {
            let mut m = game.hooks.state().behaviors.borrow_mut();
            let s = &mut m.custom_ambience;
            set(s, "CustomLightmap.Mode", json!("Custom"));
            set(
                s,
                "CustomLightmap.Mode.Custom.BlockLightTint",
                json!(0x01123456),
            );
            set(
                s,
                "CustomLightmap.Mode.Custom.NightVisionColor",
                json!(0x80123456u32 as i32),
            );
            set(s, "Fog.BackgroundColor", json!(0x01123456));
        }
        let settings = game
            .hooks
            .state()
            .behaviors
            .borrow()
            .custom_ambience
            .settings();
        crate::render::begin_frame(&settings.to_string());
        assert_eq!(
            crate::render::lightmap_color("block"),
            crate::render::lightmap_color("night")
        );
        assert_eq!(crate::render::lightmap_color("sky"), None);
        let mut unchanged = [17u8; 8];
        crate::render::lightmap_fill(&mut unchanged);
        assert_eq!(unchanged, [17; 8]);
        close(crate::render::clear_color(original).a, 1. / 255.);
    }
    #[test]
    fn spawn_shell_uses_minecraft_double_trig_fma_throttle_and_one_per_frame() {
        let oracle: Value =
            serde_json::from_str(include_str!("custom-ambience-vectors.json")).unwrap();
        for vector in oracle.as_array().unwrap() {
            let mut reference = CustomAmbience::default();
            let random: [f64; 3] = serde_json::from_value(vector["random"].clone()).unwrap();
            let player: [f64; 3] = serde_json::from_value(vector["player"].clone()).unwrap();
            let expected: [f64; 3] = serde_json::from_value(vector["position"].clone()).unwrap();
            assert!(reference.spawn_with(100, player, random));
            for i in 0..3 {
                assert!((reference.coords[0].position[i] - expected[i]).abs() < 1e-8);
            }
            assert_eq!(
                i64::from(crate::custom_ambience::vec3_hash(expected)),
                vector["hash"].as_i64().unwrap()
            );
        }
        let mut s = CustomAmbience::default();
        set(&mut s, "WorldParticles.Enabled", json!(true));
        assert!(s.spawn_with(100, [1., 2., 3.], [0.5, 0., 0.]));
        assert_eq!(s.coords[0].position, [9., 2., 3.]);
        assert_eq!(s.coords[0].expiration, 30);
        assert_eq!(
            frame(&mut s, 124, 0.)["images"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            frame(&mut s, 125, 0.)["images"].as_array().unwrap().len(),
            2
        );
        assert_eq!(
            frame(&mut s, 100000, 0.)["images"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        let before = s.last_spawn;
        simple(&mut s, "Count", json!(3));
        frame(&mut s, 100100, 0.);
        assert_eq!(s.last_spawn, before);
        s.clear();
        assert_eq!(s.last_spawn, before);
        simple(&mut s, "Radius", json!({"from":8,"to":8}));
        assert!(!s.spawn_with(200000, [0.; 3], [0.5; 3]));
        assert_eq!(s.last_spawn, 200000);
    }
    #[test]
    fn lifetime_clear_animation_motion_and_unfaded_color_follow_source_order() {
        let mut s = CustomAmbience::default();
        set(&mut s, "WorldParticles.Enabled", json!(true));
        s.coords.push(Particle {
            position: [10., 20., 30.],
            expiration: 30,
        });
        s.tick = 10;
        s.last_spawn = 1000;
        let f = frame(&mut s, 1000, 0.);
        let q = &f["images"][0];
        assert_eq!(q["age"], 10.);
        assert_eq!(q["progress"], 0.5);
        assert_eq!(q["corners"][0], json!([9.75, 20.25, 30.]));
        assert_eq!(q["color"][3], 1.);
        simple(&mut s, "YMotion.Enabled", json!(true));
        let f = frame(&mut s, 1000, 0.);
        assert_eq!(f["images"][0]["corners"][0][1], 21.25);
        simple(&mut s, "YMotion.AnimBy", json!("Age"));
        let f = frame(&mut s, 1000, 0.);
        assert_eq!(f["images"][0]["corners"][0][1], 40.25);
        simple(&mut s, "Life", json!({"from":1,"to":2}));
        assert!(s.coords.is_empty());
        assert_eq!((s.tick, s.last_spawn), (10, 1000));
        s.coords.push(Particle {
            position: [0.; 3],
            expiration: 12,
        });
        s.game_tick();
        assert_eq!(s.coords.len(), 1);
        s.game_tick();
        assert!(s.coords.is_empty());
        s.coords.push(Particle {
            position: [0.; 3],
            expiration: 100,
        });
        set(&mut s, "WorldParticles.Enabled", json!(false));
        assert!(s.coords.is_empty());
        s.game_tick();
        assert_eq!(s.tick, 12);
        assert_eq!(frame(&mut s, 2000, 0.)["groups"], json!([]));
    }
    #[test]
    fn wander_phase_uses_actual_vec3_hash_and_camera_facing_axes() {
        let mut s = CustomAmbience::default();
        set(&mut s, "WorldParticles.Enabled", json!(true));
        simple(&mut s, "Wander.Enabled", json!(true));
        simple(&mut s, "CanBeCovered", json!(false));
        s.coords.push(Particle {
            position: [0.; 3],
            expiration: 30,
        });
        s.tick = 10;
        s.last_spawn = 1000;
        assert_eq!(crate::custom_ambience::vec3_hash([0.; 3]), 0);
        assert_eq!(crate::custom_ambience::vec3_hash([1., 2., 3.]), 66584576);
        let f = frame(&mut s, 1000, 0.);
        let q = &f["images"][0];
        assert_eq!(q["depth"], false);
        close(
            q["corners"][0][0].as_f64().unwrap(),
            f64::from(0.5f32.sin() * 0.5) - 0.25,
        );
        close(
            q["corners"][0][1].as_f64().unwrap(),
            f64::from(0.35f32.sin() * 0.25) + 0.25,
        );
        close(
            q["corners"][0][2].as_f64().unwrap(),
            f64::from(0.65f32.cos() * 0.5),
        );
        assert_eq!(
            f["groups"][0]["textures"],
            json!(["builtin:Spark", "builtin:Spark"])
        );
    }
    #[test]
    fn production_dispatch_tick_and_native_snapshot_avoid_actor_reentrancy() {
        use crate::mod_api::ClientCustomAmbienceEvent;
        let mut game = crate::Game::default();
        game.client_custom_ambience_refresh();
        let held = game.hooks.state().behaviors.borrow_mut();
        assert_eq!(game.client_custom_ambience_weather((0.2, 0.3)), (0.9, 0.));
        assert_eq!(game.client_custom_ambience_time(123), 13000);
        assert_eq!(
            game.client_custom_ambience_clock(123, "minecraft:the_end"),
            123
        );
        drop(held);
        game.default_clock = Some("minecraft:the_end".into());
        assert_eq!(game.client_custom_ambience_time(123), 123);
        game.hooks
            .state()
            .behaviors
            .borrow_mut()
            .custom_ambience
            .values
            .insert("WorldParticles.Enabled".into(), json!(true));
        game.hooks.custom_ambience_tick();
        assert_eq!(
            game.hooks.state().behaviors.borrow().custom_ambience.tick,
            1
        );
        game.client_custom_ambience_frame(0.5);
        assert_eq!(crate::render::settings()["weather"], "Snowy");
        let mut output = "null".to_owned();
        game.hooks.state().behaviors.borrow_mut().active = false;
        game.hooks.custom_ambience(
            &(),
            &mut ClientCustomAmbienceEvent {
                input: r#"{"action":"frame"}"#,
                output: &mut output,
            },
        );
        assert_eq!(output, "null");
        game.client_custom_ambience_refresh();
        assert_eq!(game.client_custom_ambience_weather((0.2, 0.3)), (0.2, 0.3));
        game.title = Some(());
        game.client_custom_ambience_refresh();
        assert_eq!(game.client_custom_ambience_time(123), 123);
    }
    #[test]
    fn shader_original_eleven_pngs_quad_uv_and_depth_contract_validate() {
        let module = naga::front::wgsl::parse_str(include_str!("custom-ambience.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
        assert_eq!(std::mem::size_of::<crate::render::Vertex>(), 40);
        let q = crate::render::Quad {
            corners: [[-1., 1., 0.], [-1., -1., 0.], [1., -1., 0.], [1., 1., 0.]],
            color: [1.; 4],
            texture: 0,
        };
        let v = crate::render::vertices(&q, glam::Mat4::IDENTITY, glam::DVec3::ZERO);
        assert_eq!([v[0].uv, v[1].uv, v[2].uv], [[0., 0.], [0., 1.], [1., 1.]]);
        assert_eq!(crate::render::assets().len(), 11);
        for (name, data) in crate::render::assets() {
            let b = crate::skins::base64_decode(data).unwrap();
            let (w, h, pixels) = crate::hud::decode_bytes(&b).unwrap();
            assert!(w > 0 && h > 0, "{name}");
            assert_eq!(pixels.len(), (w * h * 4) as usize);
        }
    }
}
