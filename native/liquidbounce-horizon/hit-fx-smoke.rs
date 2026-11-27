// SPDX-License-Identifier: GPL-3.0-or-later
#[cfg(test)]
mod hit_fx_smoke {
    use crate::hit_fx::*;
    use crate::random::ThreadRandom;
    use serde_json::json;
    #[test]
    fn target_and_hit_sound_guards_follow_original_lifecycle() {
        let mut s = HitFx::default();
        let mut r = ThreadRandom::seeded(1, 1);
        s.attack(7, true, true);
        s.attack(9, false, true);
        assert_eq!(s.last_target, Some(7));
        for (name, source, alive, at) in [
            ("entity.player.hurt", "block", true, [0.; 3]),
            ("entity.player.death", "player", true, [0.; 3]),
            ("entity.player.hurt", "player", false, [0.; 3]),
            ("entity.player.hurt", "player", true, [4.01, 0., 0.]),
        ] {
            assert!(
                s.packet(name, source, 7, alive, [0.; 3], at, &mut r)
                    .is_none()
            );
            assert_eq!(s.last_target, Some(7));
        }
        let e = s
            .packet(
                "minecraft:entity.player.attack.crit",
                "hostile",
                7,
                true,
                [0.; 3],
                [4., 0., 0.],
                &mut r,
            )
            .unwrap();
        assert_eq!(e.particles, ["Fire"]);
        assert_eq!(e.sound, Some("liquidbounce:pop"));
        assert_eq!(s.last_target, None);
        s.attack(10, true, true);
        s.settings.particles = 0;
        s.settings.other_sound = 0;
        let e = s
            .packet(
                "entity.player.hurt",
                "neutral",
                10,
                true,
                [0.; 3],
                [0.; 3],
                &mut r,
            )
            .unwrap();
        assert!(e.particles.is_empty() && e.sound.is_none());
        assert_eq!(s.last_target, None);
    }
    #[test]
    fn complete_sound_registry_and_random_variant_selection() {
        assert_eq!(SOUNDS.len(), 26);
        assert_eq!(SOUNDS[3].0, "Boykisser");
        assert_eq!(SOUNDS[3].1.len(), 6);
        assert_eq!(SOUNDS[25].0, "UWU");
        let mut r = ThreadRandom::seeded(42, 3);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..200 {
            seen.insert(sound(1 << 3, &mut r).unwrap());
        }
        assert_eq!(seen.len(), 6);
        assert!(sound(0, &mut r).is_none());
    }
    #[test]
    fn all_four_settings_roundtrip_and_empty_choices() {
        use crate::config::*;
        let mut s = Settings::default();
        s.particles = 127;
        s.amount = [2, 20];
        s.other_sound = (1 << 26) - 1;
        s.self_sound = 0;
        let common = CommonSettings {
            enabled: true,
            hidden: true,
            key_bind: json!({}),
        };
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_hit_fx(&s, &common, format, &str::to_owned).unwrap();
            let patch = parse_fixture(&tree);
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            assert_eq!(patch.changes.len(), 4);
            let mut restored = Settings::default();
            for SettingChange::HitFx(change) in patch.changes {
                change.apply(&mut restored);
            }
            assert_eq!(restored, s);
        }
        let patch = parse_fixture(
            &json!({"name":"HitFX","value":[{"name":"ParticleAmount","value":{"from":10,"to":2}},{"name":"SelfSound","value":["Missing"]}]}),
        );
        assert_eq!(patch.issues.len(), 1);
        assert!(
            patch
                .changes
                .iter()
                .any(|SettingChange::HitFx(v)| v.0 == "SelfSound" && v.1 == json!(0))
        );
    }
    #[test]
    fn original_ogg_streams_decode_with_actual_horizon_audio_decoder() {
        for (name, _) in crate::audio_fixture::HIT_FX_OGG {
            let path =
                std::path::PathBuf::from("assets/liquidbounce/sounds").join(format!("{name}.ogg"));
            let (samples, rate, channels) = crate::audio_fixture::decode(&path).unwrap();
            assert!(!samples.is_empty(), "{name}");
            assert!(rate > 0 && (1..=2).contains(&channels));
            assert!(samples.iter().all(|x| x.is_finite()));
        }
    }
    #[test]
    fn production_runtime_binds_attack_context_and_both_sound_paths() {
        use crate::{
            mod_api::{ClientHitFxEvent, Host},
            runtime::LiquidBounceHooks,
        };
        struct Context;
        impl Host for Context {
            fn client_hit_fx_context(&self, id: i32) -> String {
                json!({"position":[0.,0.,0.],"alive":id==7}).to_string()
            }
        }
        let hooks = LiquidBounceHooks::default();
        let host = Context;
        hooks.hit_fx_attack(&host, 7, true);
        let mut output = String::new();
        hooks.hit_fx(&host,&mut ClientHitFxEvent{input:r#"{"action":"sound","name":"entity.player.hurt","source":"player","at":[0.0,0.0,0.0]}"#,output:&mut output});
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["particles"], json!(["Fire"]));
        assert_eq!(value["sound"], "liquidbounce:pop");
        assert_eq!(value["target"], 7);
        hooks.hit_fx(
            &host,
            &mut ClientHitFxEvent {
                input: r#"{"action":"self"}"#,
                output: &mut output,
            },
        );
        assert!(
            serde_json::from_str::<serde_json::Value>(&output).unwrap()["sound"]
                .as_str()
                .unwrap()
                .starts_with("liquidbounce:boykisser-")
        );
        hooks.state().behaviors.borrow_mut().active = false;
        output.clear();
        hooks.hit_fx(
            &host,
            &mut ClientHitFxEvent {
                input: r#"{"action":"self"}"#,
                output: &mut output,
            },
        );
        assert!(output.is_empty());
    }
}
