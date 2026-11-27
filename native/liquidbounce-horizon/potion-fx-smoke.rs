// SPDX-License-Identifier: GPL-3.0-or-later
#[cfg(test)]
mod potion_fx_smoke {
    use crate::potion_fx::{Cloud, Player, PotionFX, PotionFXSetting};
    use serde_json::{Value, json};
    fn set(s: &mut PotionFX, p: &str, v: Value) {
        PotionFXSetting(p.into(), v).apply(s)
    }
    fn frame(s: &PotionFX, p: f32, players: &[Player], clouds: &[Cloud]) -> Value {
        s.geometry(p, players, clouds, [1., 0., 0.], [0., 1., 0.])
    }
    #[test]
    fn complete_three_groups_and_six_sources_roundtrip() {
        use crate::config::*;
        let mut s = PotionFX::default();
        for prefix in [
            "Players.MainEffect",
            "Players.SecondEffect",
            "SplashPotion.MainEffect",
            "SplashPotion.SecondEffects.Effect",
            "LingeringPotion.MainEffect",
            "LingeringPotion.SecondEffects.Effect",
        ] {
            set(
                &mut s,
                &format!("{prefix}.Source.Custom.File"),
                json!(format!("{prefix}.png")),
            );
            set(
                &mut s,
                &format!("{prefix}.Source.Builtin.Preset"),
                json!(if prefix.ends_with("MainEffect") {
                    "Atlas"
                } else {
                    "Stardust"
                }),
            );
        }
        set(
            &mut s,
            "SplashPotion.MainEffect.Lifetime",
            json!({"from":7,"to":99}),
        );
        let c = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.p","action":"Toggle"}),
        };
        for f in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_potion_fx(&s, &c, f, &str::to_owned).unwrap();
            let (changes, errors) = parse_potion_fx_value(&tree, "PotionFX");
            assert!(errors.is_empty(), "{errors:?}");
            assert_eq!(changes.len(), s.values.len());
            let mut restored = PotionFX::default();
            for c in changes {
                c.apply(&mut restored);
            }
            assert_eq!(
                serialize_potion_fx(&restored, &c, f, &str::to_owned).unwrap(),
                tree
            );
            let names = tree["value"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v["name"].as_str())
                .collect::<Vec<_>>();
            assert!(names.ends_with(&["Players", "SplashPotion", "LingeringPotion"]));
        }
    }
    #[test]
    fn players_use_interpolated_position_original_uv_rotation_and_second_requirement() {
        let mut s = PotionFX::default();
        set(&mut s, "Players.MainEffect.RotationSpeed", json!(0));
        let p = Player {
            position: [4., 6., 8.],
            old: [0., 2., 4.],
            age: 20,
            color: 0x80402010u32 as i32,
        };
        let f = frame(&s, 0.5, &[p], &[]);
        let g = &f["groups"][0];
        assert_eq!(g["textures"], json!(["builtin:Dashed", "builtin:Cracked"]));
        assert_eq!(g["allow_load"], json!([true, false]));
        assert_eq!(g["quads"].as_array().unwrap().len(), 1);
        assert_eq!(g["quads"][0]["corners"][0], json!([1., 4.01, 5.]));
        assert!((g["quads"][0]["color"][3].as_f64().unwrap() - 128. / 255.).abs() < 1e-6);
        set(&mut s, "Players.SecondEffect.Enabled", json!(true));
        set(&mut s, "Players.SecondEffect.RotationSpeed", json!(90));
        let f = frame(
            &s,
            0.,
            &[Player {
                age: 1,
                position: [0.; 3],
                old: [0.; 3],
                ..p
            }],
            &[],
        );
        assert!(
            (f["groups"][0]["quads"][1]["corners"][0][0]
                .as_f64()
                .unwrap()
                + 1.)
                .abs()
                < 1e-6
        );
    }
    #[test]
    fn splash_deadlines_current_lifetime_fade_and_unfaded_flash() {
        let mut s = PotionFX::default();
        set(
            &mut s,
            "SplashPotion.MainEffect.Lifetime",
            json!({"from":2,"to":4}),
        );
        set(&mut s, "SplashPotion.MainEffect.FadeStart", json!(0.5));
        s.splash(0x80ffffffu32 as i32, [2., 3., 4.]);
        for _ in 0..3 {
            s.game_tick();
        }
        let f = frame(&s, 0., &[], &[]);
        let g = &f["groups"][1];
        assert_eq!(g["quads"].as_array().unwrap().len(), 2);
        assert!((g["quads"][0]["color"][3].as_f64().unwrap() - 64. / 255.).abs() < 1e-6);
        assert!((g["quads"][1]["color"][3].as_f64().unwrap() - 128. / 255.).abs() < 1e-6);
        set(
            &mut s,
            "SplashPotion.MainEffect.Lifetime",
            json!({"from":2,"to":10}),
        );
        assert_eq!(s.splashes[0].expiration, 4);
        set(&mut s, "SplashPotion.MainEffect.FadeStart", json!(1.));
        assert!(
            (frame(&s, 0., &[], &[])["groups"][1]["quads"][0]["color"][3]
                .as_f64()
                .unwrap()
                - 128. / 255.)
                .abs()
                < 1e-6
        );
        set(&mut s, "SplashPotion.Enabled", json!(false));
        assert!(s.splashes.is_empty());
        s.splash(-1, [0.; 3]);
        assert!(s.splashes.is_empty());
    }
    #[test]
    fn lingering_keeps_current_position_radius_secondary_offset_and_camera_flash() {
        let mut s = PotionFX::default();
        set(&mut s, "LingeringPotion.Enabled", json!(true));
        set(&mut s, "LingeringPotion.MainEffect.RotationSpeed", json!(0));
        set(
            &mut s,
            "LingeringPotion.SecondEffects.Effect.Enabled",
            json!(true),
        );
        set(
            &mut s,
            "LingeringPotion.SecondEffects.Effect.RotationSpeed",
            json!(0),
        );
        let c = Cloud {
            position: [10., 20., 30.],
            age: 4,
            radius: 2.,
            color: -1,
        };
        let f = frame(&s, 0., &[], &[c]);
        let q = f["groups"][2]["quads"].as_array().unwrap();
        assert_eq!(q.len(), 3);
        assert_eq!(q[0]["corners"][0], json!([7.625, 20.01, 27.625]));
        assert!((q[1]["corners"][0][1].as_f64().unwrap() - 20.005).abs() < 1e-9);
        assert_eq!(q[2]["texture"], 2);
        assert_eq!(q[2]["corners"][0], json!([8., 22.01, 30.]));
    }
    #[test]
    fn production_runtime_receives_splash_and_forwards_real_frame_inputs() {
        use crate::{mod_api::ClientPotionFxEvent, runtime::LiquidBounceHooks};
        let h = LiquidBounceHooks::default();
        let mut output = String::new();
        h.potion_fx(
            &(),
            &mut ClientPotionFxEvent {
                input: r#"{"action":"splash","color":-1,"position":[1.0,2.0,3.0]}"#,
                geometry: &mut output,
            },
        );
        assert_eq!(h.state().behaviors.borrow().potion_fx.splashes.len(), 1);
        h.potion_fx_tick();
        assert_eq!(h.state().behaviors.borrow().potion_fx.tick, 1);
        let input=json!({"action":"frame","partial":0.5,"players":[{"position":[0.,0.,0.],"old":[0.,0.,0.],"age":2,"color":-1}],"clouds":[],"right":[1.,0.,0.],"up":[0.,1.,0.]}).to_string();
        h.potion_fx(
            &(),
            &mut ClientPotionFxEvent {
                input: &input,
                geometry: &mut output,
            },
        );
        let f: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(f["groups"][0]["quads"].as_array().unwrap().len(), 1);
        h.state().behaviors.borrow_mut().active = false;
        output = "null".into();
        h.potion_fx(
            &(),
            &mut ClientPotionFxEvent {
                input: &input,
                geometry: &mut output,
            },
        );
        assert_eq!(output, "null");
    }
    #[test]
    fn new_texture_wgsl_original_nine_assets_and_uv_validate() {
        let m = naga::front::wgsl::parse_str(include_str!("potion-fx.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&m)
        .unwrap();
        assert_eq!(std::mem::size_of::<crate::render::Vertex>(), 40);
        let q = crate::render::Quad {
            corners: [[-1., 1., 0.], [-1., -1., 0.], [1., -1., 0.], [1., 1., 0.]],
            color: [1.; 4],
            texture: 0,
        };
        let v = crate::render::vertices(&q, glam::Mat4::IDENTITY, glam::DVec3::ZERO);
        assert_eq!(v[0].uv, [0., 0.]);
        assert_eq!(v[1].uv, [0., 1.]);
        assert_eq!(v[2].uv, [1., 1.]);
        for (name, b) in crate::assets() {
            let bytes = crate::skins::base64_decode(b).unwrap();
            let (w, h, pixels) = crate::hud::decode_bytes(&bytes).unwrap();
            assert!(w > 0 && h > 0, "{name}");
            assert_eq!(pixels.len(), (w * h * 4) as usize);
        }
    }
}
