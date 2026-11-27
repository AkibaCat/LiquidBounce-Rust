// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleBreadcrumbs: floating endpoint gradients, then original ShaderLayer's RGBA8
//! silhouettes -> nearest rows -> Euclidean field -> style -> straight-alpha scene blend.
use glam::{DVec3, Mat4, Vec2, Vec4};
use std::{borrow::Cow, cell::RefCell};
use wgpu::util::DeviceExt;
#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub struct ColoredPoint {
    pub position: [f64; 3],
    pub color: [f32; 4],
}
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Effect {
    pub style: usize,
    pub params: [f32; 8],
    pub accent: [f32; 4],
    pub range: f32,
    pub opacity: f32,
}
#[derive(Default, Debug, serde::Deserialize)]
pub struct Frame {
    pub quads: Vec<[ColoredPoint; 4]>,
    pub lines: Vec<[ColoredPoint; 2]>,
    pub effect: Option<Effect>,
}
thread_local! {static PENDING:RefCell<Option<Frame>>=const{RefCell::new(None)};}
pub fn begin_frame(json: &str) {
    PENDING.with_borrow_mut(|f| *f = serde_json::from_str(json).ok());
}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub clip: [f32; 4],
    pub color: [f32; 4],
}
/// The original plain GL line list is one physical pixel wide; endpoint colours retain
/// Float precision. Clip in homogeneous space before expanding, including endpoint alpha.
pub fn vertices(frame: &Frame, projection: Mat4, camera: DVec3, size: [u32; 2]) -> Vec<Vertex> {
    let project = |p: ColoredPoint| Vertex {
        clip: (projection
            * (DVec3::from_array(p.position) - camera)
                .as_vec3()
                .extend(1.))
        .to_array(),
        color: p.color,
    };
    let mut out = vec![];
    for q in &frame.quads {
        let q = q.map(project);
        for i in [0, 1, 2, 0, 2, 3] {
            out.push(q[i]);
        }
    }
    for line in &frame.lines {
        let mut v = line.map(project);
        let mut a = Vec4::from_array(v[0].clip);
        let mut b = Vec4::from_array(v[1].clip);
        for plane in 0..2 {
            let distance = |p: Vec4| if plane == 0 { p.w - 0.00001 } else { p.w - p.z };
            let da = distance(a);
            let db = distance(b);
            if da < 0. && db < 0. {
                a = Vec4::ZERO;
                b = Vec4::ZERO;
                break;
            }
            if (da < 0.) != (db < 0.) {
                let t = da / (da - db);
                let c =
                    std::array::from_fn(|i| v[0].color[i] + (v[1].color[i] - v[0].color[i]) * t);
                let p = a.lerp(b, t);
                if da < 0. {
                    a = p;
                    v[0].color = c;
                } else {
                    b = p;
                    v[1].color = c;
                }
            }
        }
        if a.w <= 0. || b.w <= 0. {
            continue;
        }
        let screen = Vec2::new(size[0] as f32, size[1] as f32);
        let d = (b.truncate().truncate() / b.w - a.truncate().truncate() / a.w) * screen;
        if d.length_squared() == 0. {
            continue;
        }
        let d = d.normalize();
        let n = Vec2::new(-d.y, d.x) / screen;
        let mut q = [v[0], v[0], v[1], v[1]];
        for (i, (p, sign)) in [(a, 1.), (a, -1.), (b, -1.), (b, 1.)]
            .into_iter()
            .enumerate()
        {
            q[i].clip = (p + Vec4::new(n.x * sign * p.w, n.y * sign * p.w, 0., 0.)).to_array();
        }
        for i in [0, 1, 2, 0, 2, 3] {
            out.push(q[i]);
        }
    }
    if frame.effect.is_some() {
        for v in &mut out {
            v.clip[1] = -v.clip[1];
        }
    }
    out
}
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const COMMON: &str = include_str!("breadcrumbs-shaders/effect.glsl");
const SOURCES: [&str; 11] = [
    include_str!("breadcrumbs-shaders/field_rows.frag"),
    include_str!("breadcrumbs-shaders/field_columns.frag"),
    include_str!("breadcrumbs-shaders/smooth.frag"),
    include_str!("breadcrumbs-shaders/outline.frag"),
    include_str!("breadcrumbs-shaders/neon.frag"),
    include_str!("breadcrumbs-shaders/rainbow.frag"),
    include_str!("breadcrumbs-shaders/ripple.frag"),
    include_str!("breadcrumbs-shaders/hologram.frag"),
    include_str!("breadcrumbs-shaders/halftone.frag"),
    include_str!("breadcrumbs-shaders/chromatic.frag"),
    include_str!("breadcrumbs-shaders/aura.frag"),
];
/// Only resource bindings and version change. Original field scans, RGB565 packing,
/// 8-bit distance encoding, noise functions and all nine style formulas remain verbatim.
pub fn fragment_source(index: usize) -> String {
    let globals = "layout(std140,set=0,binding=1) uniform Globals {vec4 Clock;ivec4 CameraBlock;vec4 CameraFrac;};\n#define GameTime Clock.x\n#define CameraBlockPos CameraBlock.xyz\n#define CameraOffset CameraFrac.xyz\n";
    let common = COMMON
        .replace("#include <minecraft:globals.glsl>", globals)
        .replace(
            "layout(std140) uniform EffectData",
            "layout(std140,set=0,binding=0) uniform EffectData",
        );
    let source=SOURCES[index].lines().filter(|l|!l.starts_with("#version")&&!l.starts_with("#extension")).collect::<Vec<_>>().join("\n").replace("#include <liquidbounce:effect/effect.glsl>",&common).replace("layout(std140) uniform EffectView","layout(std140,set=0,binding=2) uniform EffectView").replace("uniform sampler2D InSampler;","layout(set=0,binding=3) uniform texture2D InTexture;\nlayout(set=0,binding=4) uniform sampler InFilter;\n#define InSampler sampler2D(InTexture,InFilter)");
    format!("#version 450\n{source}")
}
pub fn fragment_module(index: usize) -> Result<naga::Module, String> {
    let source = fragment_source(index);
    let module = naga::front::glsl::Frontend::default()
        .parse(
            &naga::front::glsl::Options::from(naga::ShaderStage::Fragment),
            &source,
        )
        .map_err(|e| e.emit_to_string(&source))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|e| e.to_string())?;
    Ok(module)
}
struct Targets {
    size: [u32; 2],
    views: [wgpu::TextureView; 4],
}
pub struct BreadcrumbRenderer {
    layout: wgpu::BindGroupLayout,
    pipelines: [wgpu::RenderPipeline; 11],
    capture: wgpu::RenderPipeline,
    plain: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    targets: Option<Targets>,
    pending: Option<(Frame, wgpu::Buffer, u32)>,
}
impl BreadcrumbRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, linear: bool) -> Self {
        let uniform = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("LiquidBounce ShaderLayer"),
            entries: &[
                uniform(0),
                uniform(1),
                uniform(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("LiquidBounce ShaderLayer"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let empty = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Breadcrumbs mesh"),
            bind_group_layouts: &[],
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Breadcrumbs"),
            source: wgpu::ShaderSource::Wgsl(include_str!("breadcrumbs.wgsl").into()),
        });
        let attributes = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4];
        let constants = [(
            "FRAME_LINEAR",
            f64::from(u8::from(
                linear && format == wgpu::TextureFormat::Rgba16Float,
            )),
        )];
        let make = |vertex: &str,
                    fragment: &wgpu::ShaderModule,
                    entry: &str,
                    target: wgpu::TextureFormat,
                    mesh: bool,
                    blend: bool| {
            let buffers = [Some(wgpu::VertexBufferLayout {
                array_stride: 32,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attributes,
            })];
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("LiquidBounce Breadcrumbs"),
                layout: Some(if mesh { &empty } else { &pipeline_layout }),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vertex),
                    compilation_options: Default::default(),
                    buffers: if mesh { &buffers } else { &[] },
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: fragment,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: if entry == "plain" || entry == "composite" {
                            &constants
                        } else {
                            &[]
                        },
                        ..Default::default()
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target,
                        blend: blend.then_some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pipelines = std::array::from_fn(|i| {
            let module =
                fragment_module(i).unwrap_or_else(|e| panic!("LiquidBounce style {i}: {e}"));
            let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("LiquidBounce original GLSL"),
                source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
            });
            make("screen", &fragment, "main", FORMAT, false, false)
        });
        let capture = make("mesh", &shader, "capture", FORMAT, true, true);
        let plain = make("mesh", &shader, "plain", format, true, true);
        let composite = make(
            "composite_screen",
            &shader,
            "composite",
            format,
            false,
            true,
        );
        Self {
            layout,
            pipelines,
            capture,
            plain,
            composite,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor::default()),
            targets: None,
            pending: None,
        }
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        projection: Mat4,
        camera: DVec3,
        size: [u32; 2],
        title: bool,
    ) {
        self.pending = None;
        let frame = PENDING.with_borrow_mut(Option::take);
        let Some(frame) = frame.filter(|_| !title) else {
            return;
        };
        let mesh = vertices(&frame, projection, camera, size);
        if mesh.is_empty() {
            return;
        }
        let count = mesh.len() as u32;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Breadcrumbs mesh"),
            contents: bytemuck::cast_slice(&mesh),
            usage: wgpu::BufferUsages::VERTEX,
        });
        if frame.effect.is_some() && self.targets.as_ref().is_none_or(|t| t.size != size) {
            let views = std::array::from_fn(|_| {
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some("LiquidBounce RGBA8 style target"),
                        size: wgpu::Extent3d {
                            width: size[0],
                            height: size[1],
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: FORMAT,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    })
                    .create_view(&Default::default())
            });
            self.targets = Some(Targets { size, views });
        }
        self.pending = Some((frame, buffer, count));
    }
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        game_time: f32,
        camera: DVec3,
        view: Mat4,
        fov: f32,
    ) {
        let Some((frame, buffer, count)) = self.pending.take() else {
            return;
        };
        let pass = |encoder: &mut wgpu::CommandEncoder,
                    target: &wgpu::TextureView,
                    pipeline: &wgpu::RenderPipeline,
                    group: Option<&wgpu::BindGroup>,
                    mesh: bool,
                    clear: bool| {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("LiquidBounce Breadcrumbs"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if clear {
                            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            p.set_pipeline(pipeline);
            if let Some(g) = group {
                p.set_bind_group(0, g, &[]);
            }
            if mesh {
                p.set_vertex_buffer(0, buffer.slice(..));
                p.draw(0..count, 0..1);
            } else {
                p.draw(0..3, 0..1);
            }
        };
        let Some(effect) = frame.effect else {
            pass(encoder, target, &self.plain, None, true, false);
            return;
        };
        let targets = self.targets.as_ref().unwrap();
        let mut data = [0f32; 16];
        data[..8].copy_from_slice(&effect.params);
        data[8..12].copy_from_slice(&effect.accent);
        data[12] = effect.range;
        data[13] = effect.opacity;
        let effect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("EffectData"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let blocks = camera.floor().as_ivec3();
        let offset = blocks.as_dvec3() - camera;
        let mut globals = [0u32; 12];
        globals[0] = game_time.to_bits();
        globals[4] = blocks.x as u32;
        globals[5] = blocks.y as u32;
        globals[6] = blocks.z as u32;
        globals[8] = (offset.x as f32).to_bits();
        globals[9] = (offset.y as f32).to_bits();
        globals[10] = (offset.z as f32).to_bits();
        let globals = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals"),
            contents: bytemuck::cast_slice(&globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let inverse = view.inverse();
        let right = inverse.x_axis;
        let up = inverse.y_axis;
        let forward = -inverse.z_axis;
        let tangent = (fov.to_radians() / 2.).tan();
        let axes = [
            right.to_array(),
            up.to_array(),
            forward.to_array(),
            [
                tangent * targets.size[0] as f32 / targets.size[1] as f32,
                tangent,
                0.,
                0.,
            ],
        ];
        let axes = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("EffectView"),
            contents: bytemuck::cast_slice(&axes),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let group = |source: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("LiquidBounce ShaderLayer"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: effect_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: globals.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: axes.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            })
        };
        pass(encoder, &targets.views[0], &self.capture, None, true, true);
        for (i, pipeline) in [0, 1, effect.style + 2].into_iter().enumerate() {
            let group = group(&targets.views[i]);
            pass(
                encoder,
                &targets.views[i + 1],
                &self.pipelines[pipeline],
                Some(&group),
                false,
                true,
            );
        }
        let group = group(&targets.views[3]);
        pass(encoder, target, &self.composite, Some(&group), false, false);
    }
}
