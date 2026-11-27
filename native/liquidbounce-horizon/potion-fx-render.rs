// SPDX-License-Identifier: GPL-3.0-or-later
use glam::{DVec3, Mat4};
use std::{cell::RefCell, collections::BTreeMap};
use wgpu::util::DeviceExt;
#[derive(Debug, serde::Deserialize)]
pub struct Quad {
    pub corners: [[f64; 3]; 4],
    pub color: [f32; 4],
    pub texture: usize,
}
#[derive(Debug, serde::Deserialize)]
pub struct Group {
    pub slot: usize,
    pub textures: [String; 2],
    pub allow_load: [bool; 2],
    pub depth: bool,
    pub quads: Vec<Quad>,
}
#[derive(Debug, serde::Deserialize)]
pub struct Frame {
    pub groups: Vec<Group>,
}
thread_local! {static PENDING:RefCell<Option<Frame>>=const{RefCell::new(None)};static ERRORS:RefCell<Vec<String>>=const{RefCell::new(Vec::new())};}
pub fn begin_frame(text: &str) {
    PENDING.with_borrow_mut(|s| *s = serde_json::from_str(text).ok());
}
pub fn take_errors() -> Vec<String> {
    ERRORS.with_borrow_mut(std::mem::take)
}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub clip: [f32; 4],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}
pub fn vertices(q: &Quad, projection: Mat4, camera: DVec3) -> [Vertex; 6] {
    let uv = [[0., 0.], [0., 1.], [1., 1.], [1., 0.]];
    let v: [Vertex; 4] = std::array::from_fn(|i| Vertex {
        clip: (projection
            * (DVec3::from_array(q.corners[i]) - camera)
                .as_vec3()
                .extend(1.))
        .to_array(),
        color: q.color,
        uv: uv[i],
    });
    [0, 1, 2, 0, 2, 3].map(|i| v[i])
}
struct Draw {
    texture: String,
    depth: bool,
    start: u32,
}
pub struct PotionFxRenderer {
    images: Images,
    pipelines: [wgpu::RenderPipeline; 2],
    pending: Option<(wgpu::Buffer, Vec<Draw>)>,
}
impl PotionFxRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, linear: bool) -> Self {
        let images = Images::new(device);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PotionFX textures"),
            bind_group_layouts: &[Some(&images.layout)],
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PotionFX"),
            source: wgpu::ShaderSource::Wgsl(include_str!("potion-fx.wgsl").into()),
        });
        let attributes = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x2];
        let buffers = [Some(wgpu::VertexBufferLayout {
            array_stride: 40,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attributes,
        })];
        let constants = [(
            "FRAME_LINEAR",
            f64::from(u8::from(
                linear && format == wgpu::TextureFormat::Rgba16Float,
            )),
        )];
        let pipelines = std::array::from_fn(|i| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("PotionFX depth/textured"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &buffers,
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(i == 1),
                    depth_compare: Some(if i == 1 {
                        wgpu::CompareFunction::GreaterEqual
                    } else {
                        wgpu::CompareFunction::Always
                    }),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment"),
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
            })
        });
        Self {
            images,
            pipelines,
            pending: None,
        }
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        projection: Mat4,
        camera: DVec3,
        title: bool,
    ) {
        self.pending = None;
        let Some(frame) = PENDING.with_borrow_mut(Option::take).filter(|_| !title) else {
            return;
        };
        let mut data = vec![];
        let mut draws = vec![];
        for group in frame.groups {
            if group.slot >= 3 {
                continue;
            }
            let resolved: [Option<String>; 2] = std::array::from_fn(|i| {
                self.images.prepare(
                    device,
                    queue,
                    group.slot * 2 + i,
                    &group.textures[i],
                    group.allow_load[i],
                )
            });
            let [Some(main), Some(secondary)] = resolved else {
                continue;
            };
            for q in group.quads {
                let texture = match q.texture {
                    0 => main.clone(),
                    1 => secondary.clone(),
                    2 => {
                        let Some(path) =
                            self.images.prepare(device, queue, 0, "builtin:Glow", true)
                        else {
                            continue;
                        };
                        path
                    }
                    _ => continue,
                };
                let start = data.len() as u32;
                data.extend(vertices(&q, projection, camera));
                draws.push(Draw {
                    texture,
                    depth: group.depth,
                    start,
                });
            }
        }
        if !data.is_empty() {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("PotionFX quads"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::VERTEX,
            });
            self.pending = Some((buffer, draws));
        }
    }
    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
    ) {
        let Some((buffer, draws)) = self.pending.take() else {
            return;
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("LiquidBounce PotionFX"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_vertex_buffer(0, buffer.slice(..));
        for draw in draws {
            if let Some((_, group)) = self.images.textures.get(&draw.texture) {
                pass.set_pipeline(&self.pipelines[usize::from(draw.depth)]);
                pass.set_bind_group(0, group, &[]);
                pass.draw(draw.start..draw.start + 6, 0..1);
            }
        }
    }
}
include!("potion-fx-textures.rs");
#[derive(Default)]
struct Slot {
    selected: Option<String>,
    last: Option<String>,
    loading: Option<(
        String,
        std::sync::mpsc::Receiver<Result<(u32, u32, Vec<u8>), String>>,
    )>,
}
struct Images {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: BTreeMap<String, (wgpu::Texture, wgpu::BindGroup)>,
    slots: [Slot; 6],
}
impl Images {
    fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PotionFX images"),
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
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            layout,
            sampler,
            textures: Default::default(),
            slots: std::array::from_fn(|_| Slot::default()),
        }
    }
    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: String,
        w: u32,
        h: u32,
        data: &[u8],
    ) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PotionFX source texture"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
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
            texture.as_image_copy(),
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            texture.size(),
        );
        let view = texture.create_view(&Default::default());
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PotionFX source"),
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
        self.textures.insert(path, (texture, group));
    }
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        slot: usize,
        path: &str,
        load: bool,
    ) -> Option<String> {
        if let Some(name) = path.strip_prefix("builtin:") {
            if !self.textures.contains_key(path) {
                let data = PNGS
                    .iter()
                    .find(|(n, _)| *n == name)
                    .and_then(|(_, b)| crate::skins::base64_decode(b))
                    .and_then(|b| crate::hud::decode_bytes(&b).ok());
                if let Some((w, h, data)) = data {
                    self.upload(device, queue, path.into(), w, h, &data);
                }
            }
            return self.textures.contains_key(path).then(|| path.into());
        }
        if load && self.slots[slot].selected.as_deref() != Some(path) {
            self.slots[slot].selected = Some(path.into());
            if std::path::Path::new(path).is_file() {
                let path = path.to_owned();
                let (tx, rx) = std::sync::mpsc::channel();
                self.slots[slot].loading = Some((path.clone(), rx));
                std::thread::spawn(move || {
                    let result = std::fs::read(path)
                        .map_err(|e| e.to_string())
                        .and_then(|b| {
                            if b.starts_with(&[0xff, 0xd8]) {
                                crate::hud::decode_image_bytes(&b)
                            } else {
                                crate::hud::decode_bytes(&b)
                            }
                            .map_err(|e| e.to_string())
                        });
                    let _ = tx.send(result);
                });
            }
        }
        let ready = self.slots[slot]
            .loading
            .as_ref()
            .and_then(|(p, rx)| rx.try_recv().ok().map(|r| (p.clone(), r)));
        if let Some((p, result)) = ready {
            self.slots[slot].loading = None;
            match result {
                Ok((w, h, data)) => {
                    let key = format!("custom:{slot}:{p}");
                    self.upload(device, queue, key.clone(), w, h, &data);
                    self.slots[slot].last = Some(key);
                    let keep = self
                        .slots
                        .iter()
                        .filter_map(|s| s.last.as_deref())
                        .collect::<Vec<_>>();
                    self.textures
                        .retain(|p, _| p.starts_with("builtin:") || keep.contains(&p.as_str()));
                }
                Err(e) => ERRORS.with_borrow_mut(|errors| {
                    errors.push(format!(
                        "Failed to load texture from '{}' for Custom ({e})",
                        std::path::Path::new(&p)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    ))
                }),
            }
        }
        self.slots[slot].last.clone()
    }
}
