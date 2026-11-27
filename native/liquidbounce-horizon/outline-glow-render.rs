// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared native OutlineGlow: one tagged silhouette, independent owner styles,
//! original straight-alpha combine, then one framebuffer colour conversion.
use super::outline_owner::{self, OwnerRenderer};
use glam::{DVec3, Mat4};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeSet;
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Owner {
    Esp = 0,
    ItemEsp = 1,
    StorageEsp = 2,
}
const OWNERS: [Owner; 3] = [Owner::Esp, Owner::ItemEsp, Owner::StorageEsp];
impl Owner {
    fn tag(self) -> u32 {
        self as u32 + 1
    }
}
#[derive(Default)]
struct OwnerFrame {
    frame: Option<Value>,
    drawn: bool,
}
#[derive(Default)]
struct Registry {
    owners: [OwnerFrame; 3],
    vanilla: BTreeSet<i32>,
    models: Vec<crate::mesher::Vertex>,
}
thread_local! {static REGISTRY:RefCell<Registry>=RefCell::new(Registry::default());}
pub fn begin_frame() {
    REGISTRY.with_borrow_mut(|r| *r = Registry::default());
}
pub fn submit(owner: Owner, json: &str) {
    let frame = serde_json::from_str::<Value>(json)
        .ok()
        .filter(|v| v["effect"].is_object());
    REGISTRY.with_borrow_mut(|r| r.owners[owner as usize].frame = frame);
}
pub fn active() -> bool {
    REGISTRY.with_borrow(|r| r.owners.iter().any(|o| o.frame.is_some()))
}
fn lookup(frame: &Value, name: &str, id: i32) -> Option<u32> {
    frame[name].as_array()?.iter().find_map(|v| {
        (v[0].as_i64()? == i64::from(id))
            .then(|| v[1].as_u64().map(|c| c as u32))
            .flatten()
    })
}
fn selected(r: &Registry, id: i32) -> Option<(Owner, u32)> {
    if !r.vanilla.contains(&id)
        && !r.owners.iter().any(|o| {
            o.frame
                .as_ref()
                .is_some_and(|f| lookup(f, "targets", id).is_some())
        })
    {
        return None;
    }
    // EntityRenderer's outline eligibility is separate from team colour. ESP's
    // colour wins even when ItemESP supplied the eligible outline for this ID.
    OWNERS.into_iter().find_map(|owner| {
        let f = r.owners[owner as usize].frame.as_ref()?;
        lookup(f, "colors", id)
            .or_else(|| lookup(f, "targets", id))
            .map(|c| (owner, c))
    })
}
pub fn entity_color(id: i32) -> Option<u32> {
    REGISTRY.with_borrow(|r| selected(r, id).map(|v| v.1))
}
pub fn entity_owner(id: i32) -> Option<Owner> {
    REGISTRY.with_borrow(|r| selected(r, id).map(|v| v.0))
}
/// Vanilla glowing supplies outline eligibility before the colour mixin runs.
pub fn promote_entity(id: i32) {
    REGISTRY.with_borrow_mut(|r| {
        r.vanilla.insert(id);
    });
}
fn append(r: &mut Registry, vertices: &[crate::mesher::Vertex], argb: u32, tag: u32) {
    let argb = (argb & !7) | tag;
    let rgba = (argb & 0xff00ff00) | ((argb >> 16) & 255) | ((argb & 255) << 16);
    for q in vertices.chunks_exact(4) {
        for i in [0, 1, 2, 0, 2, 3] {
            let mut v = q[i];
            v.color = rgba;
            r.models.push(v);
        }
    }
}
pub fn capture_entity(id: i32, vertices: &[crate::mesher::Vertex]) {
    REGISTRY.with_borrow_mut(|r| {
        if let Some((owner, color)) = selected(r, id) {
            r.owners[owner as usize].drawn |= !vertices.is_empty();
            append(r, vertices, color, owner.tag());
        }
    });
}
pub fn capture_model(owner: Owner, vertices: &[crate::mesher::Vertex], argb: u32) {
    REGISTRY.with_borrow_mut(|r| {
        if r.owners[owner as usize].frame.is_some() {
            r.owners[owner as usize].drawn |= !vertices.is_empty();
            append(r, vertices, argb, owner.tag());
        }
    });
}
pub fn capture_untagged(vertices: &[crate::mesher::Vertex], argb: u32) {
    REGISTRY.with_borrow_mut(|r| {
        if r.owners.iter().any(|o| o.frame.is_some()) {
            append(r, vertices, argb, 0);
        }
    });
}
fn geometry(frame: &Value, name: &str, tag: u32) -> Vec<Value> {
    frame[name]
        .as_array()
        .into_iter()
        .flatten()
        .cloned()
        .map(|mut q| {
            if let Some(points) = q.as_array_mut() {
                for point in points {
                    if let Some(c) = point["color"].as_array_mut() {
                        if c.len() == 4 {
                            let blue = (c[2].as_f64().unwrap_or(0.) * 255.).round() as u32;
                            c[2] = json!(((blue & !7) | tag) as f64 / 255.);
                        }
                    }
                }
            }
            q
        })
        .collect()
}
pub struct OutlineGlowRenderer {
    capture: OwnerRenderer,
    owners: [OwnerRenderer; 3],
    drawing: Vec<Owner>,
    layout: wgpu::BindGroupLayout,
    composite: wgpu::RenderPipeline,
}
impl OutlineGlowRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        linear: bool,
        entity_layout: &wgpu::PipelineLayout,
    ) -> Self {
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("OutlineGlow original combine"),
            entries: &[
                texture(0),
                texture(1),
                texture(3),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("OutlineGlow original combine"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("OutlineGlow mask/combine"),
            source: wgpu::ShaderSource::Wgsl(include_str!("outline-glow.wgsl").into()),
        });
        let constants = [(
            "FRAME_LINEAR",
            f64::from(u8::from(
                linear && format == wgpu::TextureFormat::Rgba16Float,
            )),
        )];
        let composite = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("OutlineGlow original straight-alpha over"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("composite_screen"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("combine"),
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
        Self {
            capture: OwnerRenderer::new(device, format, linear, entity_layout),
            owners: std::array::from_fn(|_| {
                OwnerRenderer::new(device, format, linear, entity_layout)
            }),
            drawing: vec![],
            layout,
            composite,
        }
    }
    pub fn pending(&self) -> bool {
        !self.drawing.is_empty() && self.capture.pending()
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        projection: Mat4,
        camera: DVec3,
        size: [u32; 2],
        title: bool,
    ) {
        let (frames, models, drawing) = REGISTRY.with_borrow_mut(|r| {
            let frames = r.owners.each_ref().map(|o| o.frame.clone());
            let mut drawing = OWNERS
                .into_iter()
                .filter(|o| {
                    r.owners[*o as usize].frame.is_some()
                        && (r.owners[*o as usize].drawn
                            || ["quads", "lines"].iter().any(|name| {
                                r.owners[*o as usize].frame.as_ref().unwrap()[name]
                                    .as_array()
                                    .is_some_and(|a| !a.is_empty())
                            }))
                })
                .collect::<Vec<_>>();
            if drawing.is_empty() {
                drawing.extend(
                    OWNERS
                        .into_iter()
                        .find(|o| r.owners[*o as usize].frame.is_some()),
                );
            }
            (frames, std::mem::take(&mut r.models), drawing)
        });
        self.drawing = if title { vec![] } else { drawing };
        let mut quads = vec![];
        let mut lines = vec![];
        for owner in OWNERS {
            if let Some(f) = &frames[owner as usize] {
                quads.extend(geometry(f, "quads", owner.tag()));
                lines.extend(geometry(f, "lines", owner.tag()));
            }
        }
        let params = [0f32; 8];
        let accent = [1f32; 4];
        let effect = json!({"style":0,"params":params,"accent":accent,"range":1.,"opacity":1.});
        let frame = if self.drawing.is_empty() {
            Value::Null
        } else {
            json!({"quads":quads,"lines":lines,"effect":effect})
        };
        outline_owner::input(&frame.to_string(), models);
        self.capture
            .prepare(device, projection, camera, size, title);
        for owner in OWNERS {
            let frame = if self.drawing.contains(&owner) {
                let color = [0f32; 4];
                let point = json!({"position":camera.to_array(),"color":color});
                json!({"quads":[[point.clone(),point.clone(),point.clone(),point]],"lines":[],"effect":frames[owner as usize].as_ref().unwrap()["effect"]})
            } else {
                Value::Null
            };
            outline_owner::input(&frame.to_string(), vec![]);
            self.owners[owner as usize].prepare(device, projection, camera, size, title);
        }
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
        entity_group: &wgpu::BindGroup,
    ) {
        if !self.pending() {
            return;
        }
        self.capture.draw(
            device,
            encoder,
            target,
            game_time,
            camera,
            view,
            fov,
            entity_group,
            None,
            true,
        );
        let silhouette = self.capture.silhouette().unwrap();
        // A single owner styles the entire outline target. With multiple owners,
        // original mask.frag assigns untagged outlines to the first drawing owner.
        for (index, owner) in self.drawing.iter().copied().enumerate() {
            self.owners[owner as usize].draw(
                device,
                encoder,
                target,
                game_time,
                camera,
                view,
                fov,
                entity_group,
                Some((
                    silhouette,
                    if self.drawing.len() == 1 {
                        0
                    } else {
                        owner.tag()
                    },
                    index == 0,
                )),
                false,
            );
        }
        let top = self.owners[self.drawing[0] as usize].output().unwrap();
        let below = self
            .drawing
            .get(1)
            .and_then(|o| self.owners[*o as usize].output())
            .unwrap_or(top);
        let values = [0u32, 0, self.drawing.len() as u32, 0];
        let third = self
            .drawing
            .get(2)
            .and_then(|o| self.owners[*o as usize].output())
            .unwrap_or(top);
        let data = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("OutlineGlow count"),
            contents: bytemuck::cast_slice(&values),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("OutlineGlow combine"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(top),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(below),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(third),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: data.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("OutlineGlow original combine"),
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
        pass.set_pipeline(&self.composite);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}
