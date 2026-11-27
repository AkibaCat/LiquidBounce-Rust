// SPDX-License-Identifier: GPL-3.0-or-later
//! Real first-person hand capture, original ItemChams shaders and private history.
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
};
use wgpu::util::DeviceExt;
const RGBA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub lightmap: bool,
    pub effects: bool,
    pub recolor: bool,
    pub light_uniform: Vec<u32>,
    pub effect_uniform: Vec<u32>,
    pub shield: bool,
    pub shield_override: bool,
    pub shield_color: u32,
}
thread_local! {
    static SETTINGS:RefCell<Option<Settings>>=const{RefCell::new(None)};
    static FIRST_PERSON:Cell<bool>=const{Cell::new(false)};
    static SHIELD_BASE:Cell<bool>=const{Cell::new(false)};
    static PACK_HAND:Cell<bool>=const{Cell::new(false)};
    static HAND_LIGHTMAP:RefCell<Option<wgpu::TextureView>>=const{RefCell::new(None)};
}
pub fn begin_frame(json: &str) {
    SETTINGS.with_borrow_mut(|s| {
        *s = serde_json::from_str::<Option<Settings>>(json)
            .ok()
            .flatten()
            .filter(|s| s.light_uniform.len() == 20 && s.effect_uniform.len() == 52)
    });
}
pub struct FirstPersonScope(bool);
pub fn first_person() -> FirstPersonScope {
    FirstPersonScope(FIRST_PERSON.replace(true))
}
impl Drop for FirstPersonScope {
    fn drop(&mut self) {
        FIRST_PERSON.set(self.0);
    }
}
pub struct ShieldBaseScope(bool);
pub fn shield_base() -> ShieldBaseScope {
    ShieldBaseScope(SHIELD_BASE.replace(true))
}
impl Drop for ShieldBaseScope {
    fn drop(&mut self) {
        SHIELD_BASE.set(self.0);
    }
}
pub struct PackHandScope(bool);
pub fn pack_hand() -> PackHandScope {
    PackHandScope(PACK_HAND.replace(true))
}
impl Drop for PackHandScope {
    fn drop(&mut self) {
        PACK_HAND.set(self.0);
    }
}
pub fn pack_lightmap() -> Option<wgpu::TextureView> {
    if PACK_HAND.get() {
        HAND_LIGHTMAP.with_borrow(Clone::clone)
    } else {
        None
    }
}
pub fn shield_active() -> bool {
    FIRST_PERSON.get()
        && SHIELD_BASE.get()
        && SETTINGS.with_borrow(|s| s.as_ref().is_some_and(|s| s.shield))
}
pub fn shield_tint(original: u32) -> u32 {
    if !shield_active() {
        return original;
    }
    SETTINGS.with_borrow(|s| {
        let s = s.as_ref().unwrap();
        if s.shield_override {
            return s.shield_color;
        }
        let a = original.to_be_bytes();
        let b = s.shield_color.to_be_bytes();
        u32::from_be_bytes(std::array::from_fn(|i| {
            (u32::from(a[i]) * u32::from(b[i]) / 255) as u8
        }))
    })
}
const SOURCES: [&str; 4] = [
    include_str!("item-chams-shaders/glow.frag"),
    include_str!("item-chams-shaders/effects.frag"),
    include_str!("item-chams-shaders/trail.frag"),
    include_str!("item-chams-shaders/recolor.frag"),
];
/// Adapt resource bindings only. Four original GLSL formulas remain verbatim.
pub fn fragment_source(index: usize, linear: bool) -> String {
    let mut source = SOURCES[index]
        .lines()
        .filter(|s| !s.starts_with("#version") && !s.starts_with("#extension"))
        .collect::<Vec<_>>()
        .join("\n");
    for name in ["ItemChamsData", "ItemChamsEffectData"] {
        source = source.replace(
            &format!("layout(std140) uniform {name}"),
            &format!("layout(std140,set=0,binding=0) uniform {name}"),
        );
    }
    for (name, texture, sampler) in [
        ("texture0", 1, 2),
        ("image", 5, 6),
        ("capture", 1, 2),
        ("depthTex", 3, 4),
        ("trail", 5, 6),
        ("history", 5, 6),
    ] {
        source=source.replace(&format!("uniform sampler2D {name};"),&format!("layout(set=0,binding={texture}) uniform texture2D {name}Texture;\nlayout(set=0,binding={sampler}) uniform sampler {name}Filter;\n#define {name} sampler2D({name}Texture,{name}Filter)"));
    }
    // The native linear framebuffer stores gamma-decoded RGB, as its other
    // original shader composites do. Capture/history/lightmap remain RGBA8.
    if linear && matches!(index, 1 | 3) {
        source = source.replace("void main() {", "void originalMain() {");
        source.push_str("\nvoid main(){originalMain();fragColor.rgb=pow(max(fragColor.rgb,vec3(0.0)),vec3(2.2));}\n");
    }
    format!("#version 450\n{source}")
}
pub fn fragment_module(index: usize, linear: bool) -> Result<naga::Module, String> {
    let source = fragment_source(index, linear);
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
fn texture(
    device: &wgpu::Device,
    label: &str,
    size: [u32; 2],
    format: wgpu::TextureFormat,
) -> wgpu::TextureView {
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
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
        .create_view(&Default::default())
}
struct Targets {
    size: [u32; 2],
    capture: wgpu::TextureView,
    depth: wgpu::TextureView,
    sampled_depth: wgpu::TextureView,
    history: [wgpu::TextureView; 2],
}
impl Targets {
    fn new(device: &wgpu::Device, size: [u32; 2]) -> Self {
        Self {
            size,
            capture: texture(device, "ItemChams actual hand", size, RGBA),
            depth: texture(device, "ItemChams private reversed depth", size, DEPTH),
            sampled_depth: texture(
                device,
                "ItemChams nearest sampled depth",
                size,
                wgpu::TextureFormat::R32Float,
            ),
            history: std::array::from_fn(|_| texture(device, "ItemChams trail", size, RGBA)),
        }
    }
}
pub struct ItemChamsRenderer {
    layout: wgpu::BindGroupLayout,
    depth_layout: wgpu::BindGroupLayout,
    light_pipeline: wgpu::RenderPipeline,
    effect_pipeline: wgpu::RenderPipeline,
    trail_pipeline: wgpu::RenderPipeline,
    recolor_pipeline: wgpu::RenderPipeline,
    capture_pipeline: wgpu::RenderPipeline,
    depth_pipeline: wgpu::RenderPipeline,
    linear: wgpu::Sampler,
    nearest: wgpu::Sampler,
    lightmap: wgpu::TextureView,
    dummy_depth: wgpu::TextureView,
    settings: Option<Settings>,
    light_buffer: wgpu::Buffer,
    effect_buffer: wgpu::Buffer,
    targets: Option<Targets>,
    trail_fresh: bool,
    trail_index: usize,
    captured: bool,
}
impl ItemChamsRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        linear_frame: bool,
        entity_shader: &wgpu::ShaderModule,
        entity_layout: &wgpu::PipelineLayout,
    ) -> Self {
        let uniform = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampler = |binding, filtering| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(if filtering {
                wgpu::SamplerBindingType::Filtering
            } else {
                wgpu::SamplerBindingType::NonFiltering
            }),
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ItemChams original GLSL bindings"),
            entries: &[
                uniform,
                tex(1, true),
                sampler(2, true),
                tex(3, false),
                sampler(4, false),
                tex(5, true),
                sampler(6, true),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ItemChams effects"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let screen = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ItemChams screen/depth"),
            source: wgpu::ShaderSource::Wgsl(include_str!("item-chams.wgsl").into()),
        });
        let pipeline = |index, target, blend| {
            let module = fragment_module(index, linear_frame).expect("original ItemChams GLSL");
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ItemChams original GLSL"),
                source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ItemChams original pass"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &screen,
                    entry_point: Some(if matches!(index, 0 | 2) {
                        "screen_native"
                    } else {
                        "screen"
                    }),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let light_pipeline = pipeline(0, RGBA, None);
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let effect_pipeline = pipeline(1, format, Some(additive));
        let trail_pipeline = pipeline(2, RGBA, None);
        let recolor_pipeline = pipeline(3, format, Some(wgpu::BlendState::ALPHA_BLENDING));
        let attributes = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x2,2=>Uint32,3=>Unorm8x4,4=>Uint8x4,5=>Snorm8x4,6=>Float32x2];
        let capture_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ItemChams repeat actual prepared hand"),
            layout: Some(entity_layout),
            vertex: wgpu::VertexState {
                module: entity_shader,
                entry_point: Some("vs_item_chams_capture"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<crate::mesher::Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
                })],
            },
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: entity_shader,
                entry_point: Some("fs_entity_unmarked"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[("FRAME_LINEAR", 0.), ("HDR_WRITE_LIGHT", 0.)],
                    ..Default::default()
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: RGBA,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ItemChams private depth conversion"),
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
        let depth_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ItemChams depth"),
            bind_group_layouts: &[Some(&depth_layout)],
            ..Default::default()
        });
        let depth_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ItemChams preserve sampled depth"),
            layout: Some(&depth_pl),
            vertex: wgpu::VertexState {
                module: &screen,
                entry_point: Some("screen"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &screen,
                entry_point: Some("copy_depth"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ItemChams linear clamp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ItemChams nearest clamp"),
            ..Default::default()
        });
        let buffer = |label, size| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: &vec![0; size],
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };
        Self {
            layout,
            depth_layout,
            light_pipeline,
            effect_pipeline,
            trail_pipeline,
            recolor_pipeline,
            capture_pipeline,
            depth_pipeline,
            linear,
            nearest,
            lightmap: texture(
                device,
                "ItemChams independent 16x16 lightmap",
                [16, 16],
                RGBA,
            ),
            dummy_depth: texture(
                device,
                "ItemChams lightmap unused depth",
                [1, 1],
                wgpu::TextureFormat::R32Float,
            ),
            settings: None,
            light_buffer: buffer("ItemChams light uniform", 80),
            effect_buffer: buffer("ItemChams effect uniform", 208),
            targets: None,
            trail_fresh: false,
            trail_index: 0,
            captured: false,
        }
    }
    pub fn lightmap_view(&self) -> &wgpu::TextureView {
        &self.lightmap
    }
    pub fn lightmap_active(&self) -> bool {
        self.settings.as_ref().is_some_and(|s| s.lightmap)
    }
    pub fn pending(&self) -> bool {
        self.settings.as_ref().is_some_and(|s| s.effects)
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source_lightmap: &wgpu::TextureView,
        size: [u32; 2],
        title: bool,
    ) {
        self.captured = false;
        self.settings = if title {
            None
        } else {
            SETTINGS.with_borrow(Clone::clone)
        };
        HAND_LIGHTMAP
            .with_borrow_mut(|v| *v = self.lightmap_active().then(|| self.lightmap.clone()));
        let Some(s) = self.settings.as_ref() else {
            self.targets = None;
            self.trail_fresh = false;
            self.trail_index = 0;
            return;
        };
        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&s.light_uniform),
        );
        queue.write_buffer(
            &self.effect_buffer,
            0,
            bytemuck::cast_slice(&s.effect_uniform),
        );
        if s.lightmap {
            let group = self.group(
                device,
                &self.light_buffer,
                source_lightmap,
                &self.dummy_depth,
                source_lightmap,
            );
            Self::pass(encoder, &self.lightmap, &self.light_pipeline, &group, true);
        }
        if !s.effects || size.contains(&0) {
            self.targets = None;
            self.trail_fresh = false;
            self.trail_index = 0;
            return;
        }
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            self.targets = Some(Targets::new(device, size));
            self.trail_fresh = false;
            self.trail_index = 0;
        }
    }
    fn group(
        &self,
        device: &wgpu::Device,
        uniform: &wgpu::Buffer,
        capture: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        history: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let tex = |binding, view| wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        };
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ItemChams actual textures"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                tex(1, capture),
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.linear),
                },
                tex(3, depth),
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.nearest),
                },
                tex(5, history),
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&self.linear),
                },
            ],
        })
    }
    fn clear(encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ItemChams invalidate trail"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
    fn pass(
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
        group: &wgpu::BindGroup,
        clear: bool,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ItemChams source pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
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
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.draw(0..3, 0..1);
    }
    /// Called before the ordinary hand pass opens. Same native vertex/index range,
    /// hand projection, atlas, glint, lightmap and culling; separate transparent target.
    pub fn capture(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        hand_group: &wgpu::BindGroup,
        indices: &wgpu::Buffer,
        vertices: &wgpu::Buffer,
        quads: u32,
        first: i32,
        real: bool,
    ) {
        let _ = real;
        if !self.pending() || quads == 0 {
            return;
        }
        let Some(t) = self.targets.as_ref() else {
            return;
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ItemChams actual hand capture"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &t.capture,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &t.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(&self.capture_pipeline);
        pass.set_bind_group(0, hand_group, &[]);
        pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.draw_indexed(0..quads * 6, first, 0..1);
        drop(pass);
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ItemChams exact private depth"),
            layout: &self.depth_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&t.depth),
            }],
        });
        Self::pass(
            encoder,
            &t.sampled_depth,
            &self.depth_pipeline,
            &group,
            true,
        );
        self.captured = true;
    }
    /// Called once after the ordinary hand/HDR passes close. Synthetic presentation
    /// frames neither advance nor interrupt history. Missing real captures invalidate it.
    pub fn composite(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        real: bool,
    ) {
        if !std::mem::take(&mut self.captured) {
            if real {
                self.trail_fresh = false;
            }
            return;
        }
        let (Some(s), Some(t)) = (self.settings.as_ref(), self.targets.as_ref()) else {
            return;
        };
        let flags = s.effect_uniform[1];
        let with_trail = flags & 2 != 0;
        if flags & 31 == 0 && !s.recolor {
            return;
        }
        if with_trail && real {
            if !self.trail_fresh {
                Self::clear(encoder, &t.history[0]);
                Self::clear(encoder, &t.history[1]);
                self.trail_fresh = true;
            }
            let group = self.group(
                device,
                &self.effect_buffer,
                &t.capture,
                &t.sampled_depth,
                &t.history[self.trail_index],
            );
            Self::pass(
                encoder,
                &t.history[1 - self.trail_index],
                &self.trail_pipeline,
                &group,
                true,
            );
            self.trail_index = 1 - self.trail_index;
        } else if real {
            self.trail_fresh = false;
        }
        let group = self.group(
            device,
            &self.effect_buffer,
            &t.capture,
            &t.sampled_depth,
            if with_trail && self.trail_fresh {
                &t.history[self.trail_index]
            } else {
                &t.capture
            },
        );
        if flags & 31 != 0 {
            Self::pass(encoder, target, &self.effect_pipeline, &group, false);
        }
        if s.recolor {
            Self::pass(encoder, target, &self.recolor_pipeline, &group, false);
        }
    }
}
