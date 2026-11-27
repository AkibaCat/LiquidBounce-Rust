// SPDX-License-Identifier: GPL-3.0-or-later
//! The frozen Hats texture pipeline, with immutable builtin PNGs and scene depth.
use glam::{DVec3, Mat4};
use std::cell::RefCell;
use wgpu::util::DeviceExt;
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Image {
    pub texture: usize,
    pub corners: [[f64; 3]; 4],
    pub color: [f32; 4],
}
#[derive(Default, Debug, serde::Deserialize)]
pub struct Frame {
    pub images: Vec<Image>,
}
thread_local! {static PENDING:RefCell<Option<Frame>>=const{RefCell::new(None)};}
pub fn begin_frame(input: &str) {
    PENDING.with_borrow_mut(|p| *p = serde_json::from_str(input).ok());
}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub clip: [f32; 4],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}
pub fn vertices(image: &Image, projection: Mat4, camera: DVec3) -> Vec<Vertex> {
    let uv = [[0., 0.], [0., 1.], [1., 1.], [1., 0.]];
    let v: [Vertex; 4] = std::array::from_fn(|i| Vertex {
        clip: (projection
            * (DVec3::from_array(image.corners[i]) - camera)
                .as_vec3()
                .extend(1.))
        .to_array(),
        color: image.color,
        uv: uv[i],
    });
    [0, 1, 2, 0, 2, 3].into_iter().map(|i| v[i]).collect()
}
include!("particles-textures.rs");
pub struct ParticlesRenderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    depth_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: Vec<Option<(wgpu::Texture, wgpu::BindGroup)>>,
    pending: Vec<(usize, wgpu::Buffer, u32)>,
}
impl ParticlesRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, linear: bool) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particles textures"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particles scene depth"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particles"),
            bind_group_layouts: &[Some(&layout), Some(&depth_layout)],
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particles"),
            source: wgpu::ShaderSource::Wgsl(include_str!("particles.wgsl").into()),
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x2];
        let constants = [(
            "FRAME_LINEAR",
            f64::from(u8::from(
                linear && format == wgpu::TextureFormat::Rgba16Float,
            )),
        )];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particles"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("mesh"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 40,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                })],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("image_fragment"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &constants,
                    ..Default::default()
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            depth_layout,
            sampler,
            textures: (0..PNGS.len()).map(|_| None).collect(),
            pending: vec![],
        }
    }
    pub fn pending(&self) -> bool {
        !self.pending.is_empty()
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        projection: Mat4,
        camera: DVec3,
        title: bool,
    ) {
        self.pending.clear();
        let Some(frame) = PENDING.with_borrow_mut(Option::take).filter(|_| !title) else {
            return;
        };
        // Preserve particle insertion order; differently textured transparent sprites overlap.
        for image in frame.images {
            let Some(encoded) = PNGS.get(image.texture) else {
                continue;
            };
            if self.textures[image.texture].is_none() {
                let Some(bytes) = crate::skins::base64_decode(encoded) else {
                    continue;
                };
                let Ok((width, height, data)) = crate::hud::decode_bytes(&bytes) else {
                    continue;
                };
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Particles builtin PNG"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(width * 4),
                        rows_per_image: Some(height),
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
                let view = texture.create_view(&Default::default());
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Particles builtin PNG"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                });
                self.textures[image.texture] = Some((texture, group));
            }
            let v = vertices(&image, projection, camera);
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Particles mesh"),
                contents: bytemuck::cast_slice(&v),
                usage: wgpu::BufferUsages::VERTEX,
            });
            self.pending.push((image.texture, buffer, v.len() as u32));
        }
    }
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
    ) {
        if self.pending.is_empty() {
            return;
        }
        let depth = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particles scene depth"),
            layout: &self.depth_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(depth),
            }],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Particles"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(1, &depth, &[]);
        for (texture, buffer, count) in &self.pending {
            if let Some((_, group)) = &self.textures[*texture] {
                pass.set_bind_group(0, group, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..*count, 0..1);
            }
        }
        drop(pass);
        self.pending.clear();
    }
}
