// SPDX-License-Identifier: GPL-3.0-or-later
// Focused JumpEffect checks. No frozen ShaderLayer/GPU matrix is repeated.
#[cfg(test)]
mod jump_effect_smoke {
    use crate::jump_effect::{JumpEffect, JumpEffectSetting, shift_hue};
    use serde_json::{Value, json};

    fn set(s: &mut JumpEffect, path: &str, value: Value) {
        JumpEffectSetting(path.into(), value).apply(s);
    }
    fn child<'a>(tree: &'a Value, name: &str) -> &'a Value {
        tree["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == name)
            .unwrap()
    }
    #[test]
    fn complete_settings_roundtrip_preserves_inactive_modes_and_style_values() {
        use crate::config::{
            CommonSettings, JsonFormat, parse_jump_effect_value, serialize_jump_effect,
        };
        let mut s = JumpEffect::default();
        set(&mut s, "Mode.Image.Source", json!("Custom"));
        set(
            &mut s,
            "Mode.Image.Source.Custom.File",
            json!("C:/fixture/custom.png"),
        );
        set(&mut s, "Mode.Image.RotationSpeed", json!(-27));
        set(
            &mut s,
            "Mode.Simple.EndRadius",
            json!({"from":0.25,"to":1.25}),
        );
        set(&mut s, "Mode.Simple.Render", json!("Shader"));
        set(&mut s, "Mode.Simple.Render.Shader.Style", json!("Neon"));
        set(
            &mut s,
            "Mode.Simple.Render.Shader.Style.Neon.CoreColor",
            json!(0x12345678),
        );
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.j","action":"Toggle"}),
        };
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_jump_effect(&s, &common, format, &str::to_owned).unwrap();
            let (changes, errors) = parse_jump_effect_value(&tree, "JumpEffect");
            assert!(errors.is_empty(), "{errors:?}");
            assert_eq!(changes.len(), s.values.len());
            let mut restored = JumpEffect::default();
            for change in changes {
                change.apply(&mut restored);
            }
            assert_eq!(
                serialize_jump_effect(&restored, &common, format, &str::to_owned).unwrap(),
                tree
            );
            assert_eq!(
                restored.text("Mode.Image.Source.Custom.File"),
                "C:/fixture/custom.png"
            );
            assert_eq!(
                restored.value("Mode.Simple.Render.Shader.Style.Neon.CoreColor"),
                &json!(0x12345678)
            );
            let mode = child(&tree, "Mode");
            assert_eq!(mode["choices"].as_object().unwrap().len(), 2);
            let simple = &mode["choices"]["Simple"];
            let style = child(&child(simple, "Render")["choices"]["Shader"], "Style");
            assert_eq!(style["choices"].as_object().unwrap().len(), 9);
            let image = &mode["choices"]["Image"];
            assert_eq!(
                image["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["name"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "EndRadius",
                    "AnimCurve",
                    "HueOffsetAnim",
                    "Lifetime",
                    "CanBeCovered",
                    "RotationSpeed",
                    "Source",
                    "Colors"
                ]
            );
            if format == JsonFormat::Interop {
                let file = child(&child(image, "Source")["choices"]["Custom"], "File");
                assert_eq!(file["supportedExtensions"], json!(["png", "jpg", "jpeg"]));
                assert_eq!(file["dialogMode"], "OPEN_FILE");
                assert!(
                    tree["value"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|v| v["name"] != "Enabled")
                );
            }
        }
        for (name, value) in [
            ("Lifetime", json!({"from":10,"to":5})),
            ("EndRadius", json!({"from":0.,"to":4.})),
            ("AnimCurve", json!("Unknown")),
            ("HueOffsetAnim", json!(361)),
        ] {
            let tree = json!({"name":"JumpEffect","value":[{"name":"Mode","active":"Simple","choices":{"Simple":{"name":"Simple","value":[{"name":name,"value":value}]}}}]});
            assert_eq!(
                parse_jump_effect_value(&tree, "JumpEffect").1.len(),
                1,
                "{name}"
            );
        }
    }
    #[test]
    fn expiration_uses_original_deadline_and_current_lifetime_for_render_age() {
        let mut s = JumpEffect::default();
        set(&mut s, "Mode.Simple.Lifetime", json!({"from":5,"to":10}));
        set(&mut s, "Mode.Simple.AnimCurve", json!("Linear"));
        set(&mut s, "Mode.Simple.HueOffsetAnim", json!(0));
        s.jump([10., 64., 20.]);
        assert_eq!(s.circles[0].expiration, 10);
        for _ in 0..5 {
            s.game_tick();
        }
        let frame = s.geometry(0.);
        assert_eq!(frame["quads"].as_array().unwrap().len(), 1);
        let q = &frame["quads"][0];
        assert!((q["corners"][0][1].as_f64().unwrap() - 64.01).abs() < 1e-9);
        assert_eq!(q["shape"], 2);
        assert!((q["colors"][0][3].as_f64().unwrap() - 150. / 255.).abs() < 1e-6);
        set(&mut s, "Mode.Simple.Lifetime", json!({"from":5,"to":20}));
        assert_eq!(s.circles[0].expiration, 10);
        let alpha = (150f32 * (1. - (16. - 5.) / (20. - 5.))) as u32;
        assert_eq!(
            s.animate_color(0x964677ffu32 as i32, 16.) as u32 >> 24,
            alpha
        );
        for _ in 0..5 {
            s.game_tick();
        }
        assert!(s.circles.is_empty());
        s.jump([0.; 3]);
        set(&mut s, "Mode", json!("Simple"));
        assert_eq!(s.circles.len(), 1);
        set(&mut s, "Mode", json!("Image"));
        assert!(s.circles.is_empty());
        assert_eq!(shift_hue(0x7fff0000, 120) as u32, 0x7f00ff00);
        assert_eq!(shift_hue(0x7fff0000, -120) as u32, 0x7f0000ff);
    }
    #[test]
    fn original_image_mesh_texture_and_rotation_are_preserved() {
        let mut s = JumpEffect::default();
        set(&mut s, "Mode", json!("Image"));
        set(&mut s, "Mode.Image.AnimCurve", json!("None"));
        set(&mut s, "Mode.Image.RotationSpeed", json!(90));
        set(&mut s, "Mode.Image.HueOffsetAnim", json!(0));
        set(&mut s, "Mode.Image.CanBeCovered", json!(true));
        s.jump([0., 1., 0.]);
        let frame = s.geometry(1.);
        assert_eq!(frame["quads"].as_array().unwrap().len(), 256);
        assert_eq!(frame["texture"], "builtin:LiquidBounce");
        assert_eq!(frame["depth"], true);
        assert!(frame["effect"].is_null());
        let q = &frame["quads"][0];
        assert_eq!(q["shape"], 1);
        assert_eq!(
            q["uv"],
            json!([[0., 0.0625], [0., 0.], [0.0625, 0.], [0.0625, 0.0625]])
        );
        assert!((q["corners"][0][0].as_f64().unwrap() - 0.7).abs() < 1e-5);
        assert!((q["corners"][0][2].as_f64().unwrap() + 0.8).abs() < 1e-5);
        set(&mut s, "Mode.Image.Source", json!("Custom"));
        set(&mut s, "Mode.Image.Source.Custom.File", json!("image.png"));
        s.clear();
        assert_eq!(s.geometry(0.)["texture"], "image.png");
    }
    #[test]
    fn actual_runtime_and_sync_head_bridge_use_current_event_position() {
        use crate::{game::Game, jump_effect_bridge, runtime::LiquidBounceHooks};
        let hooks = LiquidBounceHooks::default();
        crate::JUMP_HOOK.with_borrow_mut(|slot| *slot = Some(hooks.clone()));
        jump_effect_bridge::install(crate::dispatch_jump_effect);
        let at = [1.25, 65., -3.5];
        jump_effect_bridge::jump(at);
        assert_eq!(
            hooks.state().behaviors.borrow().jump_effect.circles[0].position,
            at
        );
        hooks.jump_effect_tick();
        assert_eq!(hooks.state().behaviors.borrow().jump_effect.tick, 1);
        let game = Game::new(hooks.clone());
        game.frame(0.5);
        let rendered = crate::gfx::jump_effect::frame().unwrap();
        assert!(!rendered["quads"].as_array().unwrap().is_empty());
        hooks.state().behaviors.borrow_mut().active = false;
        hooks.jump_effect_tick();
        jump_effect_bridge::jump([99.; 3]);
        assert_eq!(hooks.state().behaviors.borrow().jump_effect.tick, 1);
        assert_eq!(
            hooks.state().behaviors.borrow().jump_effect.circles.len(),
            1
        );
        game.frame(0.5);
        assert!(crate::gfx::jump_effect::frame().is_none());
        hooks.state().behaviors.borrow_mut().jump_effect.clear();
        assert!(
            hooks
                .state()
                .behaviors
                .borrow()
                .jump_effect
                .circles
                .is_empty()
        );
    }
    #[test]
    fn new_capture_wgsl_and_mesh_layout_validate_without_style_matrix() {
        let source = include_str!("jump-effect.wgsl");
        let module = naga::front::wgsl::parse_str(source).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
        assert_eq!(std::mem::size_of::<crate::render::Vertex>(), 72);
        let mut s = JumpEffect::default();
        s.jump([0., 1., 0.]);
        let frame: crate::render::Frame = serde_json::from_value(s.geometry(1.)).unwrap();
        assert!(frame.effect.is_none());
        let vertices =
            crate::render::vertices(&frame, glam::Mat4::IDENTITY, glam::DVec3::ZERO, [64, 64]);
        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].params[0], 2.);
        assert!(vertices[0].clip[1] > 1.);
        set(&mut s, "Mode.Simple.Render", json!("Shader"));
        let shader_frame: crate::render::Frame = serde_json::from_value(s.geometry(1.)).unwrap();
        assert!(shader_frame.effect.is_some());
        let shader_vertices = crate::render::vertices(
            &shader_frame,
            glam::Mat4::IDENTITY,
            glam::DVec3::ZERO,
            [64, 64],
        );
        assert_eq!(shader_vertices.len(), vertices.len());
        set(&mut s, "Mode.Simple.CanBeCovered", json!(true));
        let covered_frame: crate::render::Frame = serde_json::from_value(s.geometry(1.)).unwrap();
        let covered_vertices = crate::render::vertices(&covered_frame,glam::Mat4::IDENTITY,glam::DVec3::ZERO,[64,64]);
        for (plain, covered) in vertices.iter().zip(&covered_vertices) {
            assert_eq!(covered.clip,plain.clip,"covered capture must use actual scene-depth coordinates before image flip");
        }

        for (plain, shader) in vertices.iter().zip(&shader_vertices) {
            assert_eq!(
                shader.clip,
                [plain.clip[0], -plain.clip[1], plain.clip[2], plain.clip[3]]
            );
            assert_eq!(shader.uv, plain.uv);
        }
    }
}
