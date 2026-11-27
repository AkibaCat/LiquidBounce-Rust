#[cfg(test)]
mod smoke {
    use super::*;
    use serde_json::{Value, json};
    use target_renderer::*;
    fn settings(mode: &str) -> TargetRendererSettings {
        let mut s = TargetRendererSettings::default();
        TargetRendererSetting("Mode".into(), json!(mode)).apply(&mut s);
        s
    }
    fn entity() -> Entity {
        Entity {
            id: 2,
            position: [2., 3., 4.],
            interpolated: [1., 2., 3.],
            height: 2.,
            health: 5.,
            max_health: 20.,
            absorption: 3.,
            hurt_time: 0,
            tick_count: 9,
            living: true,
        }
    }
    fn camera() -> Camera {
        Camera {
            partial: 0.5,
            right: [1., 0., 0.],
            up: [0., 1., 0.],
            yaw: 0.,
            pitch: 0.,
        }
    }
    fn same(a: &Value, b: &Value) {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                assert_eq!(a.as_f64().unwrap() as f32, b.as_f64().unwrap() as f32)
            }
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(b) {
                    same(a, b);
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(a.len(), b.len());
                for (k, a) in a {
                    same(a, &b[k]);
                }
            }
            _ => assert_eq!(a, b),
        }
    }
    #[test]
    fn complete_config_roundtrip_metadata_and_partial_validation() {
        let mut s = TargetRendererSettings::default();
        s.values.insert(
            "Mode.Image.Rotate.Curve".into(),
            json!([{"x":0.,"y":-33.25},{"x":1.,"y":71.5}]),
        );
        s.values
            .insert("Mode.Text2D.Text".into(), json!(["TARGET", "目标"]));
        let locales: Value =
            serde_json::from_str(include_str!("localization/display_zh_cn.json")).unwrap();
        fn metadata(v: &Value, d: &Value) {
            if let Some(k) = v["key"].as_str() {
                let key = format!("{k}.description");
                assert!(
                    d["descriptions"]
                        .get(&key)
                        .and_then(Value::as_str)
                        .is_some(),
                    "Missing {key}"
                );
            }
            if let Some(a) = v["value"].as_array() {
                for c in a {
                    metadata(c, d);
                }
            }
            if let Some(c) = v["choices"].as_object() {
                for c in c.values() {
                    metadata(c, d);
                }
            }
        }
        for format in [config::JsonFormat::File, config::JsonFormat::Interop] {
            let tree = config::target_renderer::serialize_target_renderer(
                &s,
                format,
                &str::to_owned,
                "liquidbounce.module.autoBow.bowAimbot.targetRendering",
            )
            .unwrap();
            let (changes, issues) = config::target_renderer::parse_target_renderer(
                &tree,
                "AutoBow.BowAimbot.TargetRendering",
            );
            assert!(issues.is_empty(), "{issues:?}");
            assert_eq!(changes.len(), s.values.len());
            let mut restored = TargetRendererSettings::default();
            for c in changes {
                c.apply(&mut restored);
            }
            for (k, v) in &s.values {
                same(v, &restored.values[k]);
            }
            if format == config::JsonFormat::Interop {
                metadata(&tree, &locales);
                assert_eq!(tree["valueType"], "TOGGLEABLE");
                let mode = &tree["value"][1];
                assert_eq!(mode["choices"].as_object().unwrap().len(), 8);
            }
        }
        let (changes, issues) = config::target_renderer::parse_target_renderer(
            &json!({"name":"TargetRendering","value":[{"name":"Enabled","value":false},{"name":"Mode","active":"Arrow","choices":{"Arrow":{"name":"Arrow","value":[{"name":"Size","value":"3.5"},{"name":"Color","value":"#123456"},{"name":"OutlineColor","value":{}}]}}}]}),
            "Target",
        );
        assert_eq!(issues.len(), 1);
        assert_eq!(changes.len(), 4);
        for c in changes {
            c.apply(&mut s);
        }
        assert_eq!(s.mode(), "Arrow");
        assert!(!s.boolean("Enabled"));
        assert_eq!(s.number("Mode.Arrow.Size"), 3.5);
        assert_eq!(s.color("Mode.Arrow.Color") as u32, 0xff123456);
    }
    #[test]
    fn eight_modes_five_heights_curve_and_actual_damage_state() {
        let (e, c) = (entity(), camera());
        let mut runtime = TargetRendererRuntime::default();
        for mode in MODES {
            let mut s = settings(mode);
            if mode == "Image" {
                s.values
                    .insert("Mode.Image.Source".into(), json!("Builtin"));
            }
            let f = runtime.render(&s, &e, &c, 1000, 500);
            assert!(!f.quads.is_empty() || !f.overlay.is_empty(), "{mode}");
            assert!(
                f.quads
                    .iter()
                    .flat_map(|q| q.corners)
                    .flatten()
                    .all(f64::is_finite)
            );
            let _: render::Frame = serde_json::from_value(f.json()).unwrap();
            match mode {
                "Legacy" => assert_eq!(f.quads.len(), 6),
                "Circle" => assert_eq!(f.quads.len(), 2),
                "GlowingCircle" => assert_eq!(f.quads.len(), 42),
                "Ghost" => {
                    assert_eq!(f.quads.len(), 75);
                    assert!(f.quads.iter().all(|q| q.depth));
                }
                "Hearts" => {
                    assert_eq!(f.quads.len(), 10);
                    assert!(f.quads.iter().any(|q| q.uv[2][0] == 0.5));
                    assert!(f.quads.iter().all(|q| !q.depth));
                }
                "Text2D" => assert_eq!(f.overlay[0].text, ["TARGET"]),
                "Arrow" => assert!(f.overlay[0].arrow),
                _ => {}
            }
        }
        let mut s = settings("Circle");
        let p = "Mode.Circle.HeightMode";
        let expected = [
            0.,
            2.,
            1.,
            0.5,
            f64::from((9.5f32 * 0.18f32)).sin() * f64::from(0.4f32) + f64::from(1.3f32),
        ];
        for (mode, y) in HEIGHT_MODES.into_iter().zip(expected) {
            s.values.insert(p.into(), json!(mode));
            assert!((s.height(p, &e, 0.5, false) - y).abs() < 1e-7);
        }
        s.values.insert(
            "Mode.Image.Rotate.Curve".into(),
            json!([{"x":0.,"y":0.},{"x":1.,"y":90.}]),
        );
        assert_eq!(s.rotation(500), s.rotation(1500));
        assert!(s.rotation(1000) > 89.);
        s.values
            .insert("Mode.Image.Rotate.Symmetric".into(), json!(false));
        assert_eq!(s.rotation(1000), 0.);
        let mut s = settings("Hearts");
        s.values
            .insert("Mode.Hearts.CanBeCovered".into(), json!(true));
        let mut hurt = e;
        hurt.hurt_time = 10;
        let damaged = runtime.render(&s, &hurt, &c, 1100, 600);
        assert!(
            damaged
                .quads
                .iter()
                .all(|q| q.depth && q.colors[0][1] == 0.)
        );
        hurt.id = 3;
        hurt.hurt_time = 0;
        let fresh = runtime.render(&s, &hurt, &c, 1200, 700);
        assert!(fresh.quads[0].colors[0][1] > 0.9);
    }
    #[test]
    fn actual_owner_runtime_selects_snapshots_and_gates() {
        struct Host;
        impl mod_api::Host for Host {}
        let h = runtime::LiquidBounceHooks::default();
        h.state().behaviors.borrow_mut().auto_bow.behavior.target = Some(2);
        let mut output = String::new();
        h.target_renderer(
            &Host,
            &mut mod_api::ClientTargetRendererEvent {
                input: "{\"phase\":\"targets\"}",
                geometry: &mut output,
            },
        );
        assert_eq!(output, "[2]");
        let input=json!({"phase":"frame","partial":0.5,"right":[1.,0.,0.],"up":[0.,1.,0.],"yaw":0.,"pitch":0.,"entities":[{"id":2,"position":[2.,3.,4.],"interpolated":[1.,2.,3.],"height":2.,"health":5.,"maxHealth":20.,"absorption":3.,"hurt":0,"age":9,"living":true}]}).to_string();
        h.target_renderer(
            &Host,
            &mut mod_api::ClientTargetRendererEvent {
                input: &input,
                geometry: &mut output,
            },
        );
        let f: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(f["quads"].as_array().unwrap().len(), 42);
        h.state()
            .behaviors
            .borrow_mut()
            .auto_bow
            .behavior
            .settings
            .aimbot = false;
        h.target_renderer(
            &Host,
            &mut mod_api::ClientTargetRendererEvent {
                input: "{\"phase\":\"targets\"}",
                geometry: &mut output,
            },
        );
        assert_eq!(output, "[]");
    }
    fn gpu_quad(shape: u32, texture: &str, depth: bool) -> Value {
        json!({"corners":[[-0.75,0.75,0.5],[-0.75,-0.75,0.5],[0.75,-0.75,0.5],[0.75,0.75,0.5]],"colors":[[1.,0.7,0.4,1.],[1.,0.7,0.4,1.],[1.,0.7,0.4,1.],[1.,0.7,0.4,1.]],"uv":[[0.,0.],[0.,1.],[1.,1.],[1.,0.]],"shape":shape,"inner":0.25,"inner_color":[0.,0.,1.,1.],"texture":texture,"depth":depth})
    }
    #[test]
    fn gpu_original_images_annulus_partial_heart_and_scene_depth() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut renderer =
            render::TargetRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, false);
        let desc = wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let target = device.create_texture(&desc);
        let view = target.create_view(&Default::default());
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            ..desc
        });
        let depth_view = depth.create_view(&Default::default());
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64 * 256,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut cases = vec![
            (gpu_quad(2, "", false), 0.75, true),
            (gpu_quad(4, "", true), 0.75, false),
            (gpu_quad(4, "", true), 0., true),
        ];
        for texture in ["builtin:Marker1", "builtin:Marker2", "builtin:Glow"] {
            cases.push((gpu_quad(1, texture, false), 0.75, true));
        }
        let mut partial = gpu_quad(4, "", false);
        partial["corners"][2][0] = json!(0.);
        partial["corners"][3][0] = json!(0.);
        partial["uv"][2][0] = json!(0.5);
        partial["uv"][3][0] = json!(0.5);
        cases.push((partial, 0.75, true));
        for (q, z, visible) in cases {
            render::begin_frame(&json!({"quads":[q]}).to_string());
            renderer.prepare(
                &device,
                &queue,
                glam::Mat4::IDENTITY,
                glam::DVec3::ZERO,
                false,
            );
            assert!(renderer.pending());
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(z),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });
            }
            renderer.draw(&device, &mut encoder, &view, &depth_view);
            encoder.copy_texture_to_buffer(
                target.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &read,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: None,
                    },
                },
                target.size(),
            );
            queue.submit([encoder.finish()]);
            read.slice(..)
                .map_async(wgpu::MapMode::Read, |r| r.unwrap());
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            {
                let pixels = read.slice(..).get_mapped_range().unwrap();
                let sum = pixels.chunks(4).map(|p| u64::from(p[3])).sum::<u64>();
                assert_eq!(sum > 0, visible);
            }
            read.unmap();
        }
    }
}
