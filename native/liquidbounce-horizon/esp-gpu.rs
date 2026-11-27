// SPDX-License-Identifier: GPL-3.0-or-later
// Explicitly ignored: the parent runs this once after the grouped native build.
#[cfg(test)]
mod outline_gpu {
    use crate::{gfx, mesher};
    use bytemuck::Zeroable;
    use glam::{DVec3, Mat4};
    use serde_json::{Value, json};
    use wgpu::util::DeviceExt;

    struct Gpu {
        device: wgpu::Device,
        queue: wgpu::Queue,
        layout: wgpu::BindGroupLayout,
        pipeline: wgpu::PipelineLayout,
        atlas: wgpu::Texture,
        group: wgpu::BindGroup,
        target: wgpu::Texture,
        view: wgpu::TextureView,
        read: wgpu::Buffer,
        adapter: String,
    }
    impl Gpu {
        fn new() -> Self {
            let instance = wgpu::Instance::default();
            let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
                .expect("GPU adapter required for the explicit OutlineGlow check");
            let name = format!("{:?}", adapter.get_info());
            let (device, queue) =
                pollster::block_on(adapter.request_device(&Default::default())).unwrap();
            let texture = |binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            };
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("real model atlas bindings"),
                entries: &[
                    texture(1),
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
            let pipeline = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&layout)],
                ..Default::default()
            });
            let atlas = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("atlas alpha probe"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let av = atlas.create_view(&Default::default());
            let sampler = device.create_sampler(&Default::default());
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&av),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("OutlineGlow focused readback"),
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
            });
            let view = target.create_view(&Default::default());
            let read = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 64 * 256,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            Self {
                device,
                queue,
                layout,
                pipeline,
                atlas,
                group,
                target,
                view,
                read,
                adapter: name,
            }
        }
        fn atlas_alpha(&self, alpha: u8) {
            self.queue.write_texture(
                self.atlas.as_image_copy(),
                &[255, 255, 255, alpha],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: None,
                },
                self.atlas.size(),
            );
        }
        fn clear(&self) -> wgpu::CommandEncoder {
            let mut encoder = self.device.create_command_encoder(&Default::default());
            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.view,
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
            encoder
        }
        fn pixels(&self, mut encoder: wgpu::CommandEncoder) -> Vec<[u8; 4]> {
            encoder.copy_texture_to_buffer(
                self.target.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &self.read,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: None,
                    },
                },
                self.target.size(),
            );
            self.queue.submit([encoder.finish()]);
            self.read
                .slice(..)
                .map_async(wgpu::MapMode::Read, |r| r.unwrap());
            self.device
                .poll(wgpu::PollType::wait_indefinitely())
                .unwrap();
            let result = self
                .read
                .slice(..)
                .get_mapped_range()
                .unwrap()
                .chunks_exact(4)
                .map(|p| [p[0], p[1], p[2], p[3]])
                .collect();
            self.read.unmap();
            result
        }
        fn common(&self, renderer: &mut gfx::outline_glow::OutlineGlowRenderer) -> Vec<[u8; 4]> {
            renderer.prepare(&self.device, Mat4::IDENTITY, DVec3::ZERO, [64, 64], false);
            assert!(renderer.pending());
            let mut encoder = self.clear();
            renderer.draw(
                &self.device,
                &mut encoder,
                &self.view,
                0.1,
                DVec3::ZERO,
                Mat4::IDENTITY,
                70.,
                &self.group,
            );
            self.pixels(encoder)
        }
        fn probe(
            &self,
            entry: &str,
            data: [u32; 4],
            colors: [[u8; 4]; 3],
            linear: bool,
        ) -> [u8; 4] {
            let shader = self
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("production mask/combine probe"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("outline-glow.wgsl").into()),
                });
            let mut entries = vec![wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }];
            for binding in if entry == "mask" {
                vec![0]
            } else {
                vec![0, 1, 3]
            } {
                entries.push(wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                });
            }
            let layout = self
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &entries,
                });
            let pl = self
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[Some(&layout)],
                    ..Default::default()
                });
            let constants = [("FRAME_LINEAR", f64::from(u8::from(linear)))];
            let pipeline = self
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("screen"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(entry),
                        compilation_options: wgpu::PipelineCompilationOptions {
                            constants: &constants,
                            ..Default::default()
                        },
                        targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                    }),
                    multiview_mask: None,
                    cache: None,
                });
            let textures = colors.map(|color| {
                let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: self.target.size(),
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                self.queue.write_texture(
                    texture.as_image_copy(),
                    &color.repeat(64 * 64),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: None,
                    },
                    texture.size(),
                );
                texture.create_view(&Default::default())
            });
            let uniform = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&data),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
            let mut bindings = vec![wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform.as_entire_binding(),
            }];
            for (binding, index) in if entry == "mask" {
                vec![(0, 0)]
            } else {
                vec![(0, 0), (1, 1), (3, 2)]
            } {
                bindings.push(wgpu::BindGroupEntry {
                    binding,
                    resource: wgpu::BindingResource::TextureView(&textures[index]),
                });
            }
            let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &bindings,
            });
            let mut encoder = self.clear();
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.draw(0..3, 0..1);
            }
            self.pixels(encoder)[32 * 64 + 32]
        }
    }
    fn quad(x: f32, y: f32, half: f32) -> [mesher::Vertex; 4] {
        [
            [x - half, y - half, 0.5],
            [x + half, y - half, 0.5],
            [x + half, y + half, 0.5],
            [x - half, y + half, 0.5],
        ]
        .map(|pos| {
            let mut vertex = mesher::Vertex::zeroed();
            vertex.pos = pos;
            vertex.uv = [0.5; 2];
            vertex
        })
    }
    fn frame(style: usize, id: i32, color: u32) -> String {
        let mut s = crate::esp::Esp::default();
        s.style = style;
        let e = s.effect().unwrap();
        json!({"quads":[],"lines":[],"effect":{"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity},"targets":[[id,color]]}).to_string()
    }
    fn nonzero(p: &[[u8; 4]]) -> usize {
        p.iter().filter(|c| c[3] > 0).count()
    }
    fn blurred_outline(p: &[[u8; 4]]) {
        assert_eq!(
            p[32 * 64 + 32][3],
            0,
            "Vanilla keeps the silhouette interior clear"
        );
        assert!(
            (12..16).any(|x| p[32 * 64 + x][3] > 0),
            "horizontal blur extends outside Sobel's one-pixel edge"
        );
        assert!(
            (12..16).any(|y| p[y * 64 + 32][3] > 0),
            "vertical blur extends outside Sobel's one-pixel edge"
        );
    }
    #[test]
    #[ignore = "Run once explicitly after the parent's grouped native build; requires GPU"]
    fn focused_gpu_esp_item_storage_masks_atlas_and_vanilla() {
        use g::Owner::{Esp, ItemEsp, StorageEsp};
        use gfx::outline_glow as g;
        let gpu = Gpu::new();
        let mut renderer = g::OutlineGlowRenderer::new(
            &gpu.device,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            &gpu.pipeline,
        );
        let mut cases = vec![];
        for alpha in [255, 0] {
            gpu.atlas_alpha(alpha);
            g::begin_frame();
            for (owner, id, color, x) in [
                (Esp, 1, 0xffff0000, -0.65),
                (ItemEsp, 2, 0xff00ff00, 0.),
                (StorageEsp, 3, 0xff0000f8, 0.65),
            ] {
                g::submit(owner, &frame(1, id, color));
                g::capture_entity(id, &quad(x, 0., 0.15));
            }
            g::capture_untagged(&quad(-0.65, 0.65, 0.15), 0xffffff00);
            let pixels = gpu.common(&mut renderer);
            assert_eq!(
                nonzero(&pixels) > 0,
                alpha > 0,
                "actual atlas alpha controls all owners"
            );
            if alpha > 0 {
                for (x, channel) in [(11, 0), (32, 1), (53, 2)] {
                    let p = pixels[32 * 64 + x];
                    assert!(
                        p[3] > 0
                            && p[channel] > p[(channel + 1) % 3]
                            && p[channel] > p[(channel + 2) % 3],
                        "owner x={x}: {p:?}"
                    );
                }
                assert!(
                    [11, 53].into_iter().any(|y| {
                        let p = pixels[y * 64 + 11];
                        p[3] > 0 && p[0] > p[2] && p[1] > p[2]
                    }),
                    "untagged silhouette reaches first drawing owner"
                );
            }
            cases.push(json!({"case":"three owners plus untagged atlas silhouette","atlasAlpha":alpha,"nonzero":nonzero(&pixels)}));
        }
        gpu.atlas_alpha(255);
        g::begin_frame();
        for owner in [StorageEsp, ItemEsp, Esp] {
            g::submit(
                owner,
                &frame(1, 7, if owner == Esp { 0xffff0000 } else { 0xff00ff00 }),
            );
        }
        g::capture_entity(7, &quad(0., 0., 0.5));
        let pixels = gpu.common(&mut renderer);
        let p = pixels[32 * 64 + 32];
        assert!(
            p[0] > p[1] && p[3] > 0,
            "ESP color/eligibility priority: {p:?}"
        );
        cases.push(json!({"case":"shared entity priority","center":p}));
        g::begin_frame();
        g::submit(Esp, &frame(0, 8, 0xff2080f8));
        g::capture_entity(8, &quad(0., 0., 0.5));
        let pixels = gpu.common(&mut renderer);
        blurred_outline(&pixels);
        cases.push(json!({"case":"common Vanilla Sobel H V","nonzero":nonzero(&pixels)}));
        // Storage block mesh retains its independent renderer and the same complete Vanilla chain.
        let mut storage = gfx::storage_esp::StorageRenderer::new(
            &gpu.device,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            &gpu.pipeline,
        );
        let mut block: Value = serde_json::from_str(&frame(0, 0, 0)).unwrap();
        block["quads"] =
            json!([quad(0., 0., 0.5).map(|v| json!({"position":v.pos,"color":[0.1,0.5,1.,1.]}))]);
        gfx::storage_esp::begin_frame(&block.to_string());
        storage.prepare(&gpu.device, Mat4::IDENTITY, DVec3::ZERO, [64, 64], false);
        let mut encoder = gpu.clear();
        storage.draw(
            &gpu.device,
            &mut encoder,
            &gpu.view,
            0.1,
            DVec3::ZERO,
            Mat4::IDENTITY,
            70.,
            &gpu.group,
        );
        let pixels = gpu.pixels(encoder);
        blurred_outline(&pixels);
        cases.push(json!({"case":"independent Storage block Vanilla Sobel H V","nonzero":nonzero(&pixels)}));
        // Execute the production mask WGSL for each owner and the source untagged rule.
        for tag in 0u32..=3 {
            for requested in 1u32..=3 {
                for untagged in [0, 1] {
                    let pixel = gpu.probe(
                        "mask",
                        [requested, untagged, 0, 0],
                        [[64, 128, tag as u8, 255]; 3],
                        false,
                    );
                    assert_eq!(
                        pixel[3] > 0,
                        tag == requested || (tag == 0 && untagged == 1)
                    );
                }
            }
        }
        assert_eq!(
            gpu.probe("mask", [0, 0, 0, 0], [[64, 128, 3, 255]; 3], false)[3],
            255
        );
        cases.push(json!({"case":"tag mask and first-owner untagged","combinations":25}));
        let inputs = [[220, 20, 10, 91], [10, 210, 20, 123], [20, 10, 200, 77]];
        let over = |a: [f32; 4], b: [f32; 4]| {
            let alpha = a[3] + b[3] * (1. - a[3]);
            std::array::from_fn::<_, 4, _>(|i| {
                if i == 3 {
                    alpha
                } else {
                    (a[i] * a[3] + b[i] * b[3] * (1. - a[3])) / alpha
                }
            })
        };
        let normalized = inputs.map(|c| c.map(|v| f32::from(v) / 255.));
        for linear in [false, true] {
            for count in 1..=3 {
                let mut expected = normalized[(count - 1) as usize];
                for i in (0..count - 1).rev() {
                    expected = over(normalized[i as usize], expected);
                }
                if linear {
                    for v in &mut expected[..3] {
                        *v = v.powf(2.2);
                    }
                }
                let pixel = gpu.probe("combine", [0, 0, count, 0], inputs, linear);
                for i in 0..4 {
                    assert!(
                        (f32::from(pixel[i]) - expected[i] * 255.).abs() <= 2.,
                        "straight alpha once-gamma count={count} linear={linear}: {pixel:?} vs {expected:?}"
                    );
                }
            }
        }
        cases.push(
            json!({"case":"straight alpha owner order and one gamma conversion","combinations":6}),
        );
        let report = json!({"passed":true,"adapter":gpu.adapter,"scope":"Focused shared owner/model/atlas/mask/combine and complete Vanilla chain; frozen nine-style GPU matrix is excluded.","cases":cases});
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("gpu-report.json");
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .unwrap();
        file.write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes())
            .unwrap();
        eprintln!("{report}");
    }
}
