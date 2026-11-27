// SPDX-License-Identifier: GPL-3.0-or-later
// New TotemEffect checks only; frozen ShaderLayer and target-filter matrices are retained.
#[cfg(test)]
mod totem_effect_smoke {
    use crate::totem_effect::{Snapshot, TotemEffect, TotemEffectSetting};
    use serde_json::{Value, json};
    fn set(s: &mut TotemEffect, p: &str, v: Value) {
        TotemEffectSetting(p.into(), v).apply(s);
    }
    fn pop() -> Snapshot {
        Snapshot {
            position: [10., 64., 20.],
            x_rot: 0.,
            y_rot: 0.,
            height: 1.8,
        }
    }
    fn child<'a>(v: &'a Value, name: &str) -> &'a Value {
        v["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == name)
            .unwrap()
    }
    #[test]
    fn complete_file_and_interop_roundtrip_retains_both_modes_and_styles() {
        use crate::config::*;
        let mut s = TotemEffect::default();
        set(&mut s, "Mode.Soul.YMotion.Enabled", json!(false));
        set(&mut s, "Mode.Soul.YMotion.AnimBy", json!("Progress"));
        set(&mut s, "Mode.Soul.Halo", json!(true));
        set(
            &mut s,
            "Mode.Shockwave.radius",
            json!({"from":1.25,"to":3.5}),
        );
        set(&mut s, "Mode.Shockwave.Render", json!("Shader"));
        set(&mut s, "Mode.Shockwave.Render.Shader.Style", json!("Neon"));
        set(
            &mut s,
            "Mode.Shockwave.Render.Shader.Style.Neon.CoreColor",
            json!(0x12345678),
        );
        let c = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.j","action":"Toggle"}),
        };
        for f in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_totem_effect(&s, &c, f, &str::to_owned).unwrap();
            let (changes, errors) = parse_totem_effect_value(&tree, "TotemEffect");
            assert!(errors.is_empty(), "{errors:?}");
            assert_eq!(changes.len(), s.values.len());
            let mut restored = TotemEffect::default();
            for change in changes {
                change.apply(&mut restored);
            }
            assert_eq!(
                serialize_totem_effect(&restored, &c, f, &str::to_owned).unwrap(),
                tree
            );
            let mode = child(&tree, "Mode");
            assert_eq!(mode["choices"].as_object().unwrap().len(), 2);
            let shock = &mode["choices"]["Shockwave"];
            assert_eq!(child(shock, "radius")["name"], "radius");
            assert_eq!(
                child(&child(shock, "Render")["choices"]["Shader"], "Style")["choices"]
                    .as_object()
                    .unwrap()
                    .len(),
                9
            );
            let soul = &mode["choices"]["Soul"];
            assert_eq!(
                soul["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["name"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "Lifetime",
                    "Fade",
                    "AnimCurve",
                    "CanBeCovered",
                    "Halo",
                    "YMotion",
                    "Colors"
                ]
            );
            assert_eq!(child(child(soul, "YMotion"), "Enabled")["value"], false);
        }
        for (name, value) in [
            ("radius", json!({"from":3.,"to":2.})),
            ("Lifetime", json!(0)),
            ("Fade", json!(1.1)),
            ("AnimCurve", json!("Unknown")),
        ] {
            let tree = json!({"name":"TotemEffect","value":[{"name":"Mode","active":"Shockwave","choices":{"Shockwave":{"name":"Shockwave","value":[{"name":name,"value":value}]}}}]});
            assert_eq!(
                parse_totem_effect_value(&tree, "TotemEffect").1.len(),
                1,
                "{name}"
            );
        }
    }
    #[test]
    fn source_lifetime_alias_and_mode_changes_clear_snapshots() {
        use crate::config::parse_totem_effect_value;
        let mut s = TotemEffect::default();
        s.pop(pop());
        assert_eq!(s.entities[0].expiration, 20);
        for _ in 0..19 {
            s.game_tick();
        }
        assert_eq!(s.entities.len(), 1);
        s.game_tick();
        assert!(s.entities.is_empty());
        s.pop(pop());
        set(&mut s, "Mode", json!("Shockwave"));
        assert_eq!(s.entities.len(), 1);
        let tree = json!({"name":"TotemEffect","value":[{"name":"Mode","active":"Shockwave","choices":{"Shockwave":{"name":"Shockwave","value":[{"name":"lifetime","value":30}]}}}]});
        let (changes, errors) = parse_totem_effect_value(&tree, "TotemEffect");
        assert!(errors.is_empty());
        for c in changes {
            c.apply(&mut s);
        }
        assert!(s.entities.is_empty());
        assert_eq!(s.number("Mode.Shockwave.Lifetime"), 30.);
        s.pop(pop());
        set(&mut s, "Mode.Soul.Lifetime", json!(40));
        assert!(
            s.entities.is_empty(),
            "inactive lifetime has the same source onChanged handler"
        );
        s.pop(pop());
        set(&mut s, "Mode", json!("Soul"));
        assert!(s.entities.is_empty());
    }
    #[test]
    fn shockwave_uses_snapshot_midheight_and_replaces_color_alpha() {
        let mut s = TotemEffect::default();
        set(&mut s, "Mode.Shockwave.AnimCurve", json!("None"));
        set(&mut s, "Mode.Shockwave.Fade", json!(1.));
        set(
            &mut s,
            "Mode.Shockwave.Colors.InnerColor",
            json!(0x00112233),
        );
        set(&mut s, "Mode.Shockwave.Colors.SyncColors", json!(true));
        s.pop(pop());
        let f = s.geometry(0.);
        let q = &f["quads"][0];
        assert_eq!(q["shape"], 2);
        assert!((q["corners"][0][1].as_f64().unwrap() - 64.9).abs() < 1e-6);
        assert_eq!(q["inner_color"], q["colors"][0]);
        assert_eq!(q["inner_color"][3], 1.);
        assert_eq!(f["order"], json!([[0, 0]]));
        set(&mut s, "Mode.Shockwave.Fade", json!(0.7));
        assert_eq!(s.geometry(0.)["quads"][0]["inner_color"][3], 0.);
    }
    #[test]
    fn soul_preserves_six_parts_order_motion_and_complete_torus() {
        let mut s = TotemEffect::default();
        set(&mut s, "Mode", json!("Soul"));
        set(&mut s, "Mode.Soul.AnimCurve", json!("Linear"));
        set(&mut s, "Mode.Soul.Fade", json!(1.));
        s.pop(Snapshot {
            position: [0.; 3],
            height: 1.8,
            x_rot: 0.,
            y_rot: 0.,
        });
        let f = s.geometry(2.);
        assert_eq!(f["quads"].as_array().unwrap().len(), 36);
        assert_eq!(f["lines"].as_array().unwrap().len(), 72);
        assert_eq!(f["order"].as_array().unwrap().len(), 108);
        assert_eq!(f["order"][6], json!([1, 0]));
        assert!((f["quads"][0]["corners"][0][1].as_f64().unwrap() - 1.5).abs() < 1e-6);
        assert!((f["quads"][0]["corners"][0][2].as_f64().unwrap() + 0.1).abs() < 1e-6);
        set(&mut s, "Mode.Soul.YMotion.AnimBy", json!("Progress"));
        assert!(
            (s.geometry(2.)["quads"][0]["corners"][0][1]
                .as_f64()
                .unwrap()
                - 0.075)
                .abs()
                < 1e-6
        );
        set(&mut s, "Mode.Soul.YMotion.Enabled", json!(false));
        set(&mut s, "Mode.Soul.Halo", json!(true));
        let f = s.geometry(2.);
        assert_eq!(f["quads"].as_array().unwrap().len(), 36 + 144 * 12);
        assert_eq!(f["lines"].as_array().unwrap().len(), 72);
        assert_eq!(f["quads"][36]["shape"], 0);
        assert!((f["quads"][36]["corners"][0][1].as_f64().unwrap() - 2.1).abs() < 1e-6);
    }
    #[test]
    fn actual_game_packet_and_runtime_apply_visual_target_filter() {
        use crate::{
            game::Game,
            packets::play::{Clientbound, EntityEvent},
            runtime::LiquidBounceHooks,
        };
        let hooks = LiquidBounceHooks::default();
        let game = Game::new(hooks.clone());
        game.packet(&Clientbound::EntityEvent(EntityEvent { id: 7, event: 3 }));
        assert!(
            hooks
                .state()
                .behaviors
                .borrow()
                .totem_effect
                .entities
                .is_empty()
        );
        game.packet(&Clientbound::EntityEvent(EntityEvent { id: 8, event: 35 }));
        assert!(
            hooks
                .state()
                .behaviors
                .borrow()
                .totem_effect
                .entities
                .is_empty()
        );
        game.packet(&Clientbound::EntityEvent(EntityEvent { id: 7, event: 35 }));
        assert_eq!(
            hooks.state().behaviors.borrow().totem_effect.entities[0]
                .snapshot
                .position,
            [1., 2., 3.]
        );
        assert_eq!(
            hooks.state().behaviors.borrow().totem_effect.entities[0]
                .snapshot
                .x_rot,
            15.
        );
        game.frame(0.5);
        assert!(crate::gfx::totem_effect::frame().is_some());
        hooks.state().info.replace(crate::targeting::TargetInfo {
            classification: crate::targeting::Classification::Ignored,
            ..Default::default()
        });
        game.packet(&Clientbound::EntityEvent(EntityEvent { id: 7, event: 35 }));
        assert_eq!(
            hooks.state().behaviors.borrow().totem_effect.entities.len(),
            1
        );
        hooks.totem_effect_tick();
        assert_eq!(hooks.state().behaviors.borrow().totem_effect.tick, 1);
        hooks.state().behaviors.borrow_mut().active = false;
        game.frame(0.5);
        assert!(crate::gfx::totem_effect::frame().is_none());
    }
    #[test]
    fn new_capture_wgsl_mesh_order_one_pixel_lines_and_depth_coordinates() {
        let m = naga::front::wgsl::parse_str(include_str!("totem-effect.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&m)
        .unwrap();
        assert_eq!(std::mem::size_of::<crate::render::Vertex>(), 72);
        let f:crate::render::Frame=serde_json::from_value(json!({"quads":[],"lines":[{"ends":[[0.,0.,0.],[1.,0.,0.]],"colors":[[1.,0.,0.,1.],[0.,1.,0.,1.]]}],"order":[[1,0]],"effect":null,"depth":false})).unwrap();
        let v = crate::render::vertices(&f, glam::Mat4::IDENTITY, glam::DVec3::ZERO, [64, 64]);
        assert_eq!(v.len(), 6);
        assert!((v[0].clip[1] - v[1].clip[1] - 2. / 64.).abs() < 1e-6);
        let mut s = TotemEffect::default();
        s.pop(pop());
        set(&mut s, "Mode.Shockwave.Render", json!("Shader"));
        let f: crate::render::Frame = serde_json::from_value(s.geometry(1.)).unwrap();
        let flipped =
            crate::render::vertices(&f, glam::Mat4::IDENTITY, glam::DVec3::ZERO, [64, 64]);
        assert!(flipped[0].clip[1] < 0.);
        set(&mut s, "Mode.Shockwave.CanBeCovered", json!(true));
        let f: crate::render::Frame = serde_json::from_value(s.geometry(1.)).unwrap();
        let covered =
            crate::render::vertices(&f, glam::Mat4::IDENTITY, glam::DVec3::ZERO, [64, 64]);
        assert_eq!(flipped[0].clip[1], -covered[0].clip[1]);
    }
}
