#[cfg(test)]
mod esp_smoke {
    use super::*;
    use esp::{Entity, Esp};
    use serde_json::{Value, json};
    fn e() -> Entity {
        Entity {
            identity: 11,
            id: 11,
            position: [1., 0., 0.],
            interpolated: [1.25, 2., 3.],
            width: 0.6,
            height: 2.,
            bounding_height: 2.,
            living: true,
            player: false,
            hurt: false,
            invisible: false,
            friend: false,
            tag: None,
            health: 5.,
            max_health: 20.,
            object: 0,
            own_vehicle: false,
        }
    }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} != {b}");
    }
    fn named<'a>(v: &'a Value, name: &str) -> &'a Value {
        v["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == name)
            .unwrap()
    }
    #[test]
    fn all_modes_inactive_fields_and_styles_roundtrip() {
        let mut s = Esp::default();
        s.expand = [0.22, 0.33];
        s.outline = false;
        s.merge_intersecting = true;
        s.fill = false;
        s.outline_2d = false;
        s.outline_thickness = 4.;
        s.corner = true;
        s.gap = 37.;
        s.border = false;
        s.border_thickness = 6.;
        s.health_bar = false;
        s.health_spacing = 11.;
        s.scale = 0.17;
        s.y_offset = -0.4;
        s.background_alpha = 42;
        s.invisible = 0x12345678;
        s.friends = 0x23456789;
        s.maximum_distance = 77.;
        s.objects = 7;
        s.object_color = 0x34567890;
        s.color.hue = vec![[0., 17.], [87., 250.], [200., 90.]];
        s.color.alpha = 0.4;
        s.color.health_alpha = 73;
        s.color.color = 0x45678901;
        for (i, style) in s.styles.iter_mut().enumerate() {
            for p in breadcrumbs::specs(i) {
                if p.color {
                    style.accent = 0x76543210;
                } else {
                    style.values.insert(p.name.into(), p.from);
                }
            }
        }
        for mode in 0..4 {
            for style in 0..10 {
                for color in tracers::ColorMode::ALL {
                    s.mode = mode;
                    s.style = style;
                    s.color.color_mode = color;
                    for format in [config::JsonFormat::File, config::JsonFormat::Interop] {
                        let (restored, tree) = config::roundtrip(&s, format);
                        assert_eq!(restored, s);
                        let encoded = tree.to_string();
                        assert!(!encoded.contains("liquidbounce.module.storageESP"));
                        assert!(!encoded.contains("liquidbounce.module.tracers"));
                        if format == config::JsonFormat::Interop {
                            let o = named(&tree, "Objects");
                            assert_eq!(o["canBeNone"], true);
                            assert_eq!(o["isOrderSensitive"], false);
                            assert_eq!(
                                o["choices"],
                                json!(["EndCrystals", "Projectiles", "Vehicles"])
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn original_defaults_color_order_and_strict_range() {
        let mut s = Esp::default();
        let mut a = e();
        assert_eq!(s.mode, 3);
        assert_eq!(s.style, 1);
        assert_eq!(s.objects, 0);
        for (i, v) in s.styles.iter().enumerate() {
            assert_eq!(v.get("Fill"), esp::FILLS[i]);
        }
        s.color.color_mode = tracers::ColorMode::Static;
        assert_eq!(s.color(&a, [0.; 3], 0.), 0x64ffffff);
        a.player = true;
        a.tag = Some(0xff123456);
        assert_eq!(s.color(&a, [0.; 3], 0.), 0xff123456);
        a.friend = true;
        assert_eq!(s.color(&a, [0.; 3], 0.), 0xff00ff00);
        s.friends = 0;
        assert_eq!(s.color(&a, [0.; 3], 0.), 0xff123456);
        a.invisible = true;
        assert_eq!(s.color(&a, [0.; 3], 0.), 0xffffa500);
        a.hurt = true;
        assert_eq!(s.color(&a, [0.; 3], 0.), 0xffff0000);
        a.living = false;
        assert_eq!(s.color(&a, [0.; 3], 0.), 0x64ffffff);
        a.position = [128., 0., 0.];
        assert!(!s.within(&a, [0.; 3]));
        a.position = [127.999, 0., 0.];
        assert!(s.within(&a, [0.; 3]));
        a.object = 4;
        s.objects = 4;
        assert!(s.is_object(&a));
        a.own_vehicle = true;
        assert!(!s.is_object(&a));
    }
    #[test]
    fn tick_pose_cache_and_box_alpha_are_source_fixed() {
        let mut s = Esp::default();
        s.mode = 0;
        s.color.color_mode = tracers::ColorMode::Static;
        s.color.color = 0x7f123456;
        s.selected.insert(11);
        let mut a = e();
        s.prepare(&[a], [0.; 3]);
        a.width = 10.;
        a.height = 99.;
        a.interpolated = [10., 20., 30.];
        let b = s.boxes(&[a], [999.; 3], 0.);
        close(b[0].0[0], 9.65);
        close(b[0].0[4], 22.05);
        let m = s.box_mesh(&b);
        assert_eq!(m.quads.len(), 6);
        assert_eq!(m.lines.len(), 12);
        assert_eq!(m.quads[0][0].color[3], 50. / 255.);
        assert_eq!(m.lines[0][0].color[3], 100. / 255.);
        s.outline = false;
        assert!(s.box_mesh(&b).lines.is_empty());
        s.clear();
        assert!(s.prepared.is_empty());
    }
    #[test]
    fn merge_uses_original_intersection_full_argb_and_connected_components() {
        let a = [0., 0., 0., 1., 1., 3.];
        let b = [0., 0., 2., 3., 1., 3.];
        let gap = [2., 0., 0., 3., 1., 1.];
        let v = esp::merge(vec![(a, 7), (b, 7), (gap, 7)]);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, [0., 0., 0., 3., 1., 3.]);
        let touching = [1., 0., 0., 2., 1., 1.];
        assert_eq!(
            esp::merge(vec![([0., 0., 0., 1., 1., 1.], 7), (touching, 7)]).len(),
            2
        );
        assert_eq!(esp::merge(vec![(a, 0xff000007), (a, 0x7f000007)]).len(), 2);
    }
    #[test]
    fn two_d_exact_outline_border_health_and_corner_rectangles() {
        let mut s = Esp::default();
        s.mode = 1;
        let r = s.overlay([10., 20., 30., 60.], 0x12010203, Some((10., 20.)), 2.);
        assert_eq!(r.len(), 11);
        assert_eq!(r[0].bounds, [10., 20., 30., 60.]);
        assert_eq!(r[0].color, 0x32010203);
        assert_eq!(r[1].bounds, [9.25, 19.25, 30.75, 20.75]);
        assert_eq!(r[5].bounds, [9.75, 19.75, 30.25, 20.25]);
        assert_eq!(r[5].color, 0xff010203);
        assert_eq!(r[10].bounds, [7.25, 39.75, 7.75, 60.25]);
        assert_eq!(r[10].color, 0xff7f7f00);
        s.corner = true;
        assert_eq!(s.overlay([0., 0., 20., 40.], 0, None, 2.).len(), 17);
        s.outline_2d = false;
        assert_eq!(
            s.overlay([0., 0., 20., 40.], 0, Some((25., 20.)), 2.).len(),
            3
        );
        assert_eq!(
            s.overlay([0., 0., 20., 40.], 0, Some((25., 20.)), 2.)
                .last()
                .unwrap()
                .color,
            0xff00ff00
        );
        s.outline_2d = true;
        assert!(
            s.overlay([0., 0., 20., 40.], 0, None, 0.)
                .iter()
                .any(|r| r.bounds.iter().any(|v| !v.is_finite()))
        );
    }
    #[test]
    fn legacy_marker_keeps_original_rect_order_yaw_scale_and_height() {
        let mut s = Esp::default();
        s.mode = 2;
        s.selected.insert(11);
        let m = s.legacy_mesh(&[e()], [0.; 3], 0., 0.);
        assert_eq!(m.quads.len(), 16);
        for (a, b) in m.quads[0][0].position.into_iter().zip([1.65, 1.8, 3.]) {
            close(a, b);
        }
        for (a, b) in m.quads[8][0].position.into_iter().zip([0.55, 4.3, 3.]) {
            close(a, b);
        }
        assert_eq!(m.quads[4][0].color, [0., 0., 0., 150. / 255.]);
        let n = s.legacy_mesh(&[e()], [0.; 3], 90., 0.);
        close(n.quads[0][0].position[0], 1.25);
        close(n.quads[0][0].position[2], 3.4);
    }
    fn dispatch(h: &runtime::LiquidBounceHooks, input: &Value) -> Value {
        let mut output = String::new();
        h.esp(
            &mod_api::NullHost,
            &mut mod_api::ClientEspEvent {
                input: &input.to_string(),
                output: &mut output,
            },
        );
        serde_json::from_str(&output).unwrap_or(Value::Null)
    }
    fn fact(id: u64, kind: &str) -> Value {
        json!({"id":id,"identity":id,"kind":kind,"group":0,"position":[1.,0.,0.],"interpolated":[1.25,0.,0.],"width":0.6,"height":2.,"health":20.,"maxHealth":20.,"target":{"kind":kind,"alive":true,"invisible":false,"sleeping":false,"is_local":false,"has_local_passenger":false,"within_border":true}})
    }
    #[test]
    fn runtime_glow_uses_live_filter_box_uses_tick_cache_and_malformed_entry_alignment() {
        let h = runtime::LiquidBounceHooks::default();
        h.state().behaviors.borrow_mut().esp.mode = 0;
        let mut input = json!({"tick":false,"perspective":0,"camera":[0.,0.,0.],"entities":[{},fact(11,"minecraft:zombie"),fact(12,"minecraft:zombie")]});
        assert!(
            dispatch(&h, &input)["plain"]["quads"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        input["tick"] = json!(true);
        dispatch(&h, &input);
        assert_eq!(h.state().behaviors.borrow().esp.selected.len(), 1);
        input["tick"] = json!(false);
        assert_eq!(
            dispatch(&h, &input)["plain"]["quads"]
                .as_array()
                .unwrap()
                .len(),
            6
        );
        h.state().behaviors.borrow_mut().esp.mode = 3;
        assert_eq!(
            dispatch(&h, &input)["glow"]["targets"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        h.state().behaviors.borrow_mut().esp.maximum_distance = 1.;
        assert_eq!(
            dispatch(&h, &input)["glow"]["targets"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(h.state().behaviors.borrow().esp.true_sight.len(), 2);
        h.state().behaviors.borrow_mut().disabled = true;
        assert!(dispatch(&h, &input).is_null());
        assert!(h.state().behaviors.borrow().esp.true_sight.is_empty());
    }
    #[test]
    fn runtime_color_eligibility_is_separate_from_visual_filter_and_own_vehicle() {
        let h = runtime::LiquidBounceHooks::default();
        let input = json!({"tick":false,"perspective":0,"camera":[0.,0.,0.],"entities":[fact(11,"minecraft:cow")]});
        let v = dispatch(&h, &input);
        assert!(v["glow"]["targets"].as_array().unwrap().is_empty());
        assert_eq!(v["glow"]["colors"].as_array().unwrap().len(), 1);
        let a = json!({"id":13,"identity":13,"kind":"minecraft:boat","position":[1.,0.,0.],"interpolated":[1.,0.,0.],"width":1.,"height":1.,"living":false,"object":4,"ownVehicle":true});
        let mut input = json!({"tick":false,"perspective":0,"camera":[0.,0.,0.],"entities":[a]});
        h.state().behaviors.borrow_mut().esp.objects = 4;
        assert!(
            dispatch(&h, &input)["glow"]["targets"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        input["entities"][0]["ownVehicle"] = json!(false);
        assert_eq!(
            dispatch(&h, &input)["glow"]["targets"][0],
            json!([13, 0x64ffffffu32])
        );
    }
    fn frame(targets: Value, colors: Value) -> String {
        json!({"quads":[],"lines":[],"effect":{"style":0,"params":vec![0.;8],"accent":vec![0.;4],"range":6.,"opacity":1.},"targets":targets,"colors":colors}).to_string()
    }
    #[test]
    fn shared_registry_prioritizes_esp_item_storage_and_vanilla_eligibility() {
        use g::Owner::*;
        use gfx::outline_glow as g;
        g::begin_frame();
        g::submit(StorageEsp, &frame(json!([[11, 0xff445566u32]]), json!([])));
        assert_eq!(g::entity_owner(11), Some(StorageEsp));
        g::submit(ItemEsp, &frame(json!([[11, 0xff223344u32]]), json!([])));
        assert_eq!(g::entity_owner(11), Some(ItemEsp));
        g::submit(
            Esp,
            &frame(json!([]), json!([[11, 0xff123456u32], [12, 0xffabcdefu32]])),
        );
        assert_eq!(g::entity_owner(11), Some(Esp));
        assert_eq!(g::entity_color(11), Some(0xff123456));
        assert_eq!(g::entity_owner(12), None);
        g::promote_entity(12);
        assert_eq!(g::entity_color(12), Some(0xffabcdef));
        g::begin_frame();
        assert!(!g::active());
        assert_eq!(g::entity_owner(11), None);
    }
    #[test]
    fn model_capture_retains_positions_uv_and_exact_blue_tag_native_color() {
        use bytemuck::Zeroable;
        use gfx::outline_glow as g;
        g::begin_frame();
        g::submit(
            g::Owner::Esp,
            &frame(json!([[11, 0xff123456u32]]), json!([])),
        );
        let mut q = [mesher::Vertex::zeroed(); 4];
        for (i, v) in q.iter_mut().enumerate() {
            v.pos = [i as f32, 2., 3.];
            v.uv = [0.25, 0.75];
        }
        g::capture_entity(11, &q);
        let v = g::fixture_models();
        assert_eq!(v.len(), 6);
        for (i, p) in [0, 1, 2, 0, 2, 3].into_iter().zip(v) {
            assert_eq!(p.pos, q[i].pos);
            assert_eq!(p.uv, q[i].uv);
            assert_eq!(p.color, 0xff513412);
        }
        g::capture_entity(12, &q);
        assert_eq!(g::fixture_models().len(), 6);
        g::capture_untagged(&q, 0xffabcdef);
        assert_eq!(g::fixture_models()[6].color, 0xffe8cdab);
    }
    #[test]
    fn generated_original_glsl_and_shared_wgsl_validate_without_gpu() {
        for i in 0..14 {
            gfx::outline_owner::fragment_module(i).unwrap();
            gfx::storage_esp::fragment_module(i).unwrap();
        }
        let source = include_str!("outline-glow.wgsl");
        let module = naga::front::wgsl::parse_str(source).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
        let source = include_str!("storage-esp-model.wgsl");
        let module = naga::front::wgsl::parse_str(source).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
    #[test]
    fn both_chinese_dictionaries_cover_every_esp_specific_leaf() {
        let (_, tree) = config::roundtrip(&Esp::default(), config::JsonFormat::Interop);
        fn walk(v: &Value, keys: &mut Vec<String>) {
            match v {
                Value::Array(a) => {
                    for v in a {
                        walk(v, keys)
                    }
                }
                Value::Object(o) => {
                    if let Some(k) = o.get("key").and_then(Value::as_str) {
                        if k.starts_with("liquidbounce.module.esp.")
                            && !k.ends_with(".bind")
                            && !k.ends_with(".hidden")
                        {
                            keys.push(format!("{k}.description"));
                        }
                    }
                    for v in o.values() {
                        walk(v, keys)
                    }
                }
                _ => {}
            }
        }
        let mut keys = vec![];
        walk(&tree, &mut keys);
        for lang in ["zh_cn", "zh_tw"] {
            let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("localization/display_{lang}.json"));
            let data: Value = serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
            for k in &keys {
                assert!(
                    data["descriptions"][k]
                        .as_str()
                        .is_some_and(|s| !s.is_empty()),
                    "{lang} {k}"
                );
            }
        }
    }
}
