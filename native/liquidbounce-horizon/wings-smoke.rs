#[cfg(test)]
mod smoke {
    use super::*;
    use serde_json::json;
    use wings::{Entity, Wings};
    fn entity() -> Entity {
        Entity {
            local: true,
            friend: false,
            position: [0.; 3],
            interpolated: [0.; 3],
            body_yaw: 0.,
            scale: 1.,
            height: 1.8,
            chest: false,
            hurt: false,
        }
    }
    #[test]
    fn source_geometry_fade_selection_scaling_damage_and_solid_defaults() {
        let mut s = Wings::default();
        let e = entity();
        let mesh = s.mesh(&[e.clone()], [0.; 3], false, false, false);
        assert_eq!(mesh.len(), 32);
        assert!(
            mesh.iter()
                .flatten()
                .all(|v| v.position.iter().all(|x| x.is_finite()))
        );
        assert_eq!(mesh[1][1].color[3], 0.);
        assert_eq!(mesh[0][1].color[3], 1.);
        s.angles = [0, 0];
        s.count = 1;
        let mesh = s.mesh(&[e.clone()], [0.; 3], false, false, false);
        assert_eq!(mesh.len(), 8);
        for (a, b) in mesh[0][0].position.into_iter().zip([0.1, 1.325, -0.25]) {
            assert!((a - b).abs() < 1e-6, "{a} != {b}");
        }
        assert!(s.mesh(&[e.clone()], [0.; 3], true, false, false).is_empty());
        assert!(!s.mesh(&[e.clone()], [0.; 3], true, true, false).is_empty());
        let mut friend = e.clone();
        friend.local = false;
        friend.friend = true;
        friend.position = [64., 0., 0.];
        assert!(!s.selected(&friend, [0.; 3], false, false));
        friend.position = [63.99, 0., 0.];
        assert!(s.selected(&friend, [0.; 3], false, false));
        let mut hurt = e.clone();
        hurt.hurt = true;
        s.color = 0x334677ff;
        let m = s.mesh(&[hurt.clone()], [0.; 3], false, false, false);
        assert_eq!(m[0][0].color, [1., 0., 0., 0x33 as f32 / 255.]);
        s.render.shader = true;
        let m = s.mesh(&[hurt], [0.; 3], false, false, false);
        assert_eq!(m[0][0].color, [1., 0., 0., 1.]);
        assert_eq!(m[1][1].color[3], 0.);
        assert!(s.render.styles.iter().all(|style| style.get("Fill") == 1.));
        let mut scaled = e.clone();
        scaled.scale = 2.;
        scaled.height = 3.6;
        scaled.chest = true;
        s.render.shader = false;
        let m = s.mesh(&[scaled], [0.; 3], false, false, false);
        assert!((m[0][0].position[1] - 2.35).abs() < 1e-6);
        assert!((m[0][0].position[2] + 0.7).abs() < 1e-6);
        let standing = s.mesh(&[e.clone()], [0.; 3], false, false, false);
        let shifted = s.mesh(&[e], [0.; 3], false, false, true);
        assert_ne!(standing[0][0].position, shifted[0][0].position);
        s.count = 12;
        s.length = 0.1;
        assert_eq!(s.mesh(&[entity()], [0.; 3], false, false, false).len(), 96);
    }
    #[test]
    fn original_all_style_gpu_draw_and_scene_depth_clip() {
        let instance = wgpu::Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&Default::default())).expect("GPU");
        eprintln!("Wings adapter {:?}", adapter.get_info());
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut renderer =
            render::WingsRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, false);
        let size = wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth.create_view(&Default::default());
        let depth_shader=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:None,source:wgpu::ShaderSource::Wgsl("@vertex fn vertex(@builtin(vertex_index)i:u32)->@builtin(position)vec4<f32>{let p=array<vec2<f32>,3>(vec2(-1.,-1.),vec2(3.,-1.),vec2(-1.,3.));return vec4(p[i],0.75,1.);}".into())});
        let depth_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: &depth_shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: None,
            multiview_mask: None,
            cache: None,
        });
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64 * 256,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        for style in 0usize..10 {
            let mut settings = breadcrumbs::StyleSettings::new(style.saturating_sub(1));
            settings.values.insert("Fill".into(), 1.);
            let effect = if style == 0 {
                serde_json::Value::Null
            } else {
                let e = settings.effect(style - 1);
                json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity})
            };
            let p = |x, y| json!({"position":[x,y,0.5],"color":[0.2,0.5,1.,1.]});
            render::begin_frame(&json!({"quads":[[p(-0.8,-0.8),p(0.8,-0.8),p(0.8,0.8),p(-0.8,0.8)]],"lines":[],"effect":effect}).to_string());
            renderer.prepare(
                &device,
                glam::Mat4::IDENTITY,
                glam::DVec3::ZERO,
                [64, 64],
                false,
            );
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
                    ..Default::default()
                });
            }
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });
                pass.set_pipeline(&depth_pipe);
                pass.set_scissor_rect(0, 0, 64, 32);
                pass.draw(0..3, 0..1);
            }
            renderer.draw(
                &device,
                &mut encoder,
                &view,
                &depth_view,
                0.1,
                glam::DVec3::ZERO,
                glam::Mat4::IDENTITY,
                70.,
            );
            encoder.copy_texture_to_buffer(
                texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &read,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: None,
                    },
                },
                size,
            );
            queue.submit([encoder.finish()]);
            read.slice(..)
                .map_async(wgpu::MapMode::Read, |r| r.unwrap());
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            {
                let b = read.slice(..).get_mapped_range().unwrap();
                assert!(
                    b.chunks_exact(4).any(|p| p[3] != 0),
                    "style {style} drew nothing"
                );
                if style <= 1 {
                    assert_eq!(
                        b[16 * 256 + 32 * 4 + 3],
                        0,
                        "style {style} ignored scene depth"
                    );
                    assert!(
                        b[48 * 256 + 32 * 4 + 3] > 0,
                        "style {style} clipped the visible half"
                    );
                }
            }
            read.unmap();
        }
    }
}
