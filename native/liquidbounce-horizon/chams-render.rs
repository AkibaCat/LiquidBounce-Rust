// SPDX-License-Identifier: GPL-3.0-or-later
//! LiquidBounce ae0eb396 ModuleChams: isolated entity storage and depth-aware image composite.
use super::Vertex;
use std::cell::RefCell;
use wgpu::util::DeviceExt;

pub const CAPTURE: u32 = 1 << 26;
pub const UNSUPPORTED: u32 = 1 << 25;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    mode: String,
    file: String,
    mapping: String,
    tile_width: i32,
    filtering: String,
    offset: [f32; 2],
}
#[derive(Default)]
struct Pending {
    settings: Option<Settings>,
    vertices: Vec<Vertex>,
}
thread_local! { static PENDING: RefCell<Pending> = RefCell::new(Pending::default()); }

/// Called before all level geometry; the first-person hand never gets an entity scope.
pub fn begin_frame(settings: &str) {
    PENDING.with_borrow_mut(|frame| {
        frame.settings = serde_json::from_str(settings).ok();
        frame.vertices.clear();
    });
}
pub fn active() -> bool {
    PENDING.with_borrow(|frame| frame.settings.is_some())
}

/// The low 16 bits encode Horizon's actual model/item render material. Eyes,
/// emissive and breeze-wind layers are explicitly tagged at their submission.
pub fn supported(vertex: &Vertex) -> bool {
    vertex.layer & UNSUPPORTED == 0 && matches!(vertex.layer & 0xffff, 0 | 1 | 7 | 8 | 16..=33 | 36)
}
pub fn mark_entity(vertices: &mut [Vertex]) {
    for quad in vertices.as_chunks_mut::<4>().0 {
        if supported(&quad[0]) {
            for vertex in quad {
                vertex.layer |= CAPTURE;
            }
        }
    }
}

/// Separate captured quads after all builders finish, rebasing ordinary vertex
/// offsets so frame-generation and shader-pack identities remain correct.
pub fn finish_entities(vertices: &mut Vec<Vertex>) -> Vec<u32> {
    if !active() {
        return Vec::new();
    }
    let mut offsets = Vec::with_capacity(vertices.len() + 1);
    let mut kept = 0usize;
    PENDING.with_borrow_mut(|frame| {
        vertices.retain_mut(|vertex| {
            offsets.push(kept as u32);
            if vertex.layer & CAPTURE != 0 && frame.settings.is_some() {
                vertex.layer &= !(CAPTURE | UNSUPPORTED);
                frame.vertices.push(*vertex);
                false
            } else {
                vertex.layer &= !(CAPTURE | UNSUPPORTED);
                kept += 1;
                true
            }
        });
    });
    offsets.push(kept as u32);
    offsets
}

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
    0 => Float32x3, 1 => Float32x2, 2 => Uint32, 3 => Unorm8x4,
    4 => Uint8x4, 5 => Snorm8x4, 6 => Float32x2
];

struct Targets {
    size: [u32; 2],
    color: wgpu::TextureView,
    depth: wgpu::TextureView,
}
struct Image {
    generation: u64,
    size: [u32; 2],
    view: wgpu::TextureView,
}
struct LoadedImage {
    generation: u64,
    size: [u32; 2],
    pixels: Vec<u8>,
}
#[derive(Default)]
struct ImageCache {
    requested: Option<String>,
    loaded: Option<LoadedImage>,
    generation: u64,
}
impl ImageCache {
    fn refresh(&mut self, settings: &Settings, limit: u32) {
        if settings.mode != "Image" || self.requested.as_deref() == Some(&settings.file) {
            return;
        }
        self.requested = Some(settings.file.clone());
        // FileValue.toTextureProperty filters non-files and retains the previous
        // successful texture after a decode error. It never clears it on failure.
        if let Some((width, height, pixels)) = std::fs::read(&settings.file)
            .ok()
            .and_then(|bytes| crate::hud::decode_image_bytes(&bytes).ok())
            .filter(|(w, h, _)| *w > 0 && *h > 0 && *w <= limit && *h <= limit)
        {
            self.generation = self.generation.wrapping_add(1);
            self.loaded = Some(LoadedImage {
                generation: self.generation,
                size: [width, height],
                pixels,
            });
        }
    }
}
struct Gpu {
    capture: [wgpu::RenderPipeline; 2],
    composite: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    nearest: wgpu::Sampler,
    sampler: wgpu::Sampler,
    sampling: (bool, bool),
    uniform: wgpu::Buffer,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    capacity: usize,
    count: u32,
    materials: [u32; 3],
    targets: Option<Targets>,
    image: Option<Image>,
    blank: wgpu::TextureView,
    group: Option<wgpu::BindGroup>,
}

/// Keeps references to the renderer's ordinary entity shader/layout. All Chams
/// pipelines, buffers, images and frame targets are released when disabled.
pub(super) struct ChamsRenderer {
    shader: wgpu::ShaderModule,
    entity_layout: wgpu::PipelineLayout,
    frame_format: wgpu::TextureFormat,
    linear: bool,
    gpu: Option<Gpu>,
    images: ImageCache,
}
impl ChamsRenderer {
    pub(super) fn new(
        shader: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        frame_format: wgpu::TextureFormat,
        linear: bool,
    ) -> Self {
        Self {
            shader: shader.clone(),
            entity_layout: layout.clone(),
            frame_format,
            linear,
            gpu: None,
            images: ImageCache::default(),
        }
    }
    pub(super) fn pending(&self) -> bool {
        self.gpu.as_ref().is_some_and(|gpu| gpu.count > 0)
            || PENDING.with_borrow(|frame| !frame.vertices.is_empty())
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uploads: &mut super::upload::Uploads,
        encoder: &mut wgpu::CommandEncoder,
        size: [u32; 2],
        scene_depth: &wgpu::TextureView,
        entity_group: &wgpu::BindGroup,
        improved_transparency: bool,
    ) {
        let pending = PENDING.with_borrow(|frame| Pending {
            settings: frame.settings.clone(),
            vertices: frame.vertices.clone(),
        });
        let Some(settings) = pending.settings else {
            self.gpu = None;
            return;
        };
        self.images
            .refresh(&settings, device.limits().max_texture_dimension_2d);
        if pending.vertices.is_empty() {
            if let Some(gpu) = &mut self.gpu {
                gpu.count = 0;
                gpu.group = None;
            }
            return;
        }
        let gpu = self.gpu.get_or_insert_with(|| {
            Gpu::new(
                device,
                &self.shader,
                &self.entity_layout,
                self.frame_format,
                self.linear,
            )
        });
        gpu.count = (pending.vertices.len() / 4) as u32;
        gpu.group = None;
        gpu.prepare(
            device,
            queue,
            uploads,
            encoder,
            &settings,
            &pending.vertices,
            size,
            scene_depth,
            self.images.loaded.as_ref(),
        );
        let Some(target) = &gpu.targets else {
            return;
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("LiquidBounce Chams capture"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.color,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_bind_group(0, entity_group, &[]);
        pass.set_vertex_buffer(0, gpu.vertices.slice(..));
        pass.set_index_buffer(gpu.indices.slice(..), wgpu::IndexFormat::Uint32);
        let [solid, models, items] = gpu.materials;
        pass.set_pipeline(&gpu.capture[0]);
        pass.draw_indexed(0..solid * 6, 0, 0..1);
        // Pinned ModuleChams uses renderAllFeatures, not executeOit. Vanilla
        // aliases ten classic translucent phase fields when the option is on,
        // so it replays that same phase ten times with a null OitStage. The
        // official-JAR dispatch oracle records this upstream quirk explicitly.
        for _ in 0..if improved_transparency { 10 } else { 1 } {
            pass.set_pipeline(&gpu.capture[0]);
            pass.draw_indexed(0..models * 6, (solid * 4) as i32, 0..1);
            // ITEM_TRANSLUCENT / translucent glint items do not write depth.
            pass.set_pipeline(&gpu.capture[1]);
            pass.draw_indexed(0..items * 6, ((solid + models) * 4) as i32, 0..1);
        }
    }
    pub(super) fn composite(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let Some(gpu) = &mut self.gpu else {
            return;
        };
        let Some(group) = gpu.group.take() else {
            return;
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("LiquidBounce Chams composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&gpu.composite);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
        gpu.count = 0;
    }
}
impl Gpu {
    fn new(
        device: &wgpu::Device,
        entity_shader: &wgpu::ShaderModule,
        entity_layout: &wgpu::PipelineLayout,
        frame_format: wgpu::TextureFormat,
        linear: bool,
    ) -> Self {
        let capture = |depth_write| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("LiquidBounce Chams entity capture"),
                layout: Some(entity_layout),
                vertex: wgpu::VertexState {
                    module: entity_shader,
                    entry_point: Some("vs_entity"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &ATTRIBUTES,
                    })],
                },
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: entity_shader,
                    entry_point: Some("fs_entity"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &super::linear_blend::frame_constants(frame_format, linear),
                        ..Default::default()
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format: FORMAT,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let texture = |binding, depth| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: if depth {
                    wgpu::TextureSampleType::Depth
                } else {
                    wgpu::TextureSampleType::Float { filterable: true }
                },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampler = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("LiquidBounce Chams bindings"),
            entries: &[
                texture(0, false),
                texture(1, true),
                texture(2, true),
                texture(3, false),
                sampler(4),
                sampler(5),
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LiquidBounce Chams composite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("chams.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("LiquidBounce Chams composite layout"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let composite = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("LiquidBounce Chams composite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_quad"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_chams"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[("FRAME_LINEAR", f64::from(u8::from(linear)))],
                    ..Default::default()
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: frame_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::COLOR,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let (vertices, indices) = Self::buffers(device, 4);
        let blank = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("LiquidBounce Chams image fallback"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        Self {
            capture: [capture(true), capture(false)],
            composite,
            layout,
            nearest: device.create_sampler(&Default::default()),
            sampler: Self::sampler(device, false, false),
            sampling: (false, false),
            uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("LiquidBounce Chams mapping"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            vertices,
            indices,
            capacity: 4,
            count: 0,
            materials: [0; 3],
            targets: None,
            image: None,
            blank,
            group: None,
        }
    }
    fn sampler(device: &wgpu::Device, repeats: bool, linear: bool) -> wgpu::Sampler {
        let address = if repeats {
            wgpu::AddressMode::Repeat
        } else {
            wgpu::AddressMode::ClampToEdge
        };
        let filter = if linear {
            wgpu::FilterMode::Linear
        } else {
            wgpu::FilterMode::Nearest
        };
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("LiquidBounce Chams image sampler"),
            address_mode_u: address,
            address_mode_v: address,
            mag_filter: filter,
            min_filter: filter,
            ..Default::default()
        })
    }
    fn buffers(device: &wgpu::Device, capacity: usize) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("LiquidBounce Chams captured vertices"),
            size: (capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let indices: Vec<u32> = (0..capacity as u32 / 4)
            .flat_map(|q| [q * 4, q * 4 + 1, q * 4 + 2, q * 4, q * 4 + 2, q * 4 + 3])
            .collect();
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("LiquidBounce Chams quad indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        (vertices, indices)
    }
    #[allow(clippy::too_many_arguments)]
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uploads: &mut super::upload::Uploads,
        encoder: &mut wgpu::CommandEncoder,
        settings: &Settings,
        vertices: &[Vertex],
        size: [u32; 2],
        scene_depth: &wgpu::TextureView,
        loaded: Option<&LoadedImage>,
    ) {
        if vertices.len() > self.capacity {
            self.capacity = vertices.len().next_power_of_two();
            (self.vertices, self.indices) = Self::buffers(device, self.capacity);
        }
        let material = |vertex: &Vertex| match vertex.layer & 0xffff {
            32 | 33 => 2,
            _ if vertex.layer & (1 << 24) != 0 => 1,
            _ => 0,
        };
        let quads = vertices.as_chunks::<4>().0;
        let mut grouped = Vec::with_capacity(vertices.len());
        self.materials = [0; 3];
        for kind in 0..3 {
            for quad in quads.iter().filter(|quad| material(&quad[0]) == kind) {
                grouped.extend_from_slice(quad);
                self.materials[kind] += 1;
            }
        }
        uploads.buffer(encoder, &self.vertices, 0, bytemuck::cast_slice(&grouped));
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            let target = |label, format| {
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some(label),
                        size: wgpu::Extent3d {
                            width: size[0],
                            height: size[1],
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    })
                    .create_view(&Default::default())
            };
            self.targets = Some(Targets {
                size,
                color: target("LiquidBounce Chams color", FORMAT),
                depth: target(
                    "LiquidBounce Chams depth",
                    wgpu::TextureFormat::Depth32Float,
                ),
            });
        }
        if let Some(loaded) = loaded
            && self
                .image
                .as_ref()
                .is_none_or(|image| image.generation != loaded.generation)
        {
            let texture = device.create_texture_with_data(
                queue,
                &wgpu::TextureDescriptor {
                    label: Some("LiquidBounce Chams image"),
                    size: wgpu::Extent3d {
                        width: loaded.size[0],
                        height: loaded.size[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &loaded.pixels,
            );
            self.image = Some(Image {
                generation: loaded.generation,
                size: loaded.size,
                view: texture.create_view(&Default::default()),
            });
        }
        let sampling = (settings.mapping == "Repeat", settings.filtering == "Linear");
        if self.sampling != sampling {
            self.sampling = sampling;
            self.sampler = Self::sampler(device, sampling.0, sampling.1);
        }
        let image = self.image.as_ref();
        let image_size = image.map_or([1, 1], |image| image.size);
        let mut uniform = mapping_uniform(settings, size, image_size);
        if image.is_none() {
            uniform[4] = 0.0;
        }
        uploads.buffer(encoder, &self.uniform, 0, bytemuck::cast_slice(&uniform));
        let targets = self.targets.as_ref().expect("prepared Chams target");
        let entry = |binding, view| wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        };
        self.group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("LiquidBounce Chams frame"),
            layout: &self.layout,
            entries: &[
                entry(0, &targets.color),
                entry(1, &targets.depth),
                entry(2, scene_depth),
                entry(3, image.map_or(&self.blank, |image| &image.view)),
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.nearest),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.uniform.as_entire_binding(),
                },
            ],
        }));
    }
}
fn mapping_uniform(settings: &Settings, target: [u32; 2], image: [u32; 2]) -> [f32; 8] {
    let (scale, mut offset) = match settings.mapping.as_str() {
        "Stretch" => (target.map(|v| v as f32), [0.0; 2]),
        "Cover" => {
            let scale =
                (target[0] as f32 / image[0] as f32).max(target[1] as f32 / image[1] as f32);
            let width = image[0] as f32 * scale;
            let height = image[1] as f32 * scale;
            (
                [width, height],
                [
                    (target[0] as f32 - width) * 0.5,
                    (target[1] as f32 - height) * 0.5,
                ],
            )
        }
        _ => {
            let width = settings.tile_width as f32;
            ([width, width * image[1] as f32 / image[0] as f32], [0.0; 2])
        }
    };
    for (v, configured) in offset.iter_mut().zip(settings.offset) {
        *v += configured;
    }
    [
        scale[0],
        scale[1],
        offset[0],
        offset[1],
        f32::from(u8::from(settings.mode == "Image")),
        target[0] as f32,
        target[1] as f32,
        0.0,
    ]
}
