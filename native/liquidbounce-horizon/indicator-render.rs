// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, RefCell};
use wgpu::util::DeviceExt;
thread_local! {static PENDING:RefCell<Vec<Vertex>>=const{RefCell::new(Vec::new())};}
thread_local! {static LAYER_END:Cell<u32>=const{Cell::new(0)};}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub local: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub notch: f32,
}
pub fn clear() {
    PENDING.with_borrow_mut(Vec::clear);
    LAYER_END.set(0);
}
pub fn finish_layer(vertices: usize) {
    LAYER_END.set(vertices as u32);
}
pub fn submit(
    origin: [f32; 2],
    angle: f32,
    size: [f32; 2],
    notch: f32,
    argb: u32,
    viewport: [f32; 2],
) {
    if size[0] <= 0. || size[1] <= 0. || notch >= size[1] || argb >> 24 == 0 {
        return;
    }
    let color = [
        ((argb >> 16) & 255) as f32 / 255.,
        ((argb >> 8) & 255) as f32 / 255.,
        (argb & 255) as f32 / 255.,
        (argb >> 24) as f32 / 255.,
    ];
    let local = [
        [-size[0] * 0.5, 0.],
        [-size[0] * 0.5, size[1]],
        [size[0] * 0.5, size[1]],
        [size[0] * 0.5, 0.],
    ];
    let quantized = size.map(|v| ((v * 32.).clamp(0., 32767.) + 0.5).floor() / 32.);
    let (s, c) = (f64::from(angle).sin() as f32, f64::from(angle).cos() as f32);
    let vertices: [Vertex; 4] = std::array::from_fn(|i| {
        let p = local[i];
        let at = [
            origin[0] + c * p[0] - s * p[1],
            origin[1] + s * p[0] + c * p[1],
        ];
        Vertex {
            pos: [at[0] / viewport[0] * 2. - 1., 1. - at[1] / viewport[1] * 2.],
            local: p,
            size: quantized,
            color,
            notch: notch.max(0.),
        }
    });
    PENDING.with_borrow_mut(|out| out.extend([0, 1, 2, 0, 2, 3].map(|i| vertices[i])));
}
pub struct IndicatorRenderer {
    pipeline: super::hud_coverage::Pipeline,
    pending: Option<(wgpu::Buffer, u32, u32)>,
}
impl IndicatorRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, coverage: bool) -> Self {
        let source = wgpu::ShaderSource::Wgsl(include_str!("indicator.wgsl").into());
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LiquidBounce indicator SDF"),
            source: source.clone(),
        });
        let attributes = wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2,2=>Float32x2,3=>Float32x4,4=>Float32];
        let buffers = [Some(wgpu::VertexBufferLayout {
            array_stride: 44,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attributes,
        })];
        let desc = wgpu::RenderPipelineDescriptor {
            label: Some("LiquidBounce indicator SDF"),
            layout: None,
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
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        };
        Self {
            pipeline: super::hud_coverage::Pipeline::new(device, &desc, source, coverage)
                .expect("indicator location0 permits HUD coverage"),
            pending: None,
        }
    }
    pub fn prepare(&mut self, device: &wgpu::Device, title: bool) {
        self.pending = None;
        let data = PENDING.with_borrow_mut(std::mem::take);
        if title || data.is_empty() {
            return;
        }
        self.pending = Some((
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("LiquidBounce indicator arrows"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            data.len() as u32,
            LAYER_END.get(),
        ));
    }
    pub fn layer_end(&self) -> Option<u32> {
        self.pending.as_ref().map(|(_, _, end)| *end)
    }
    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        coverage: Option<&wgpu::TextureView>,
    ) {
        let Some((buffer, count, _)) = self.pending.as_ref() else {
            return;
        };
        let mut attachments = vec![Some(super::hud_coverage::attachment(target))];
        if let Some(view) = coverage {
            attachments.push(Some(super::hud_coverage::attachment(view)));
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("LiquidBounce indicator arrows"),
            color_attachments: &attachments,
            ..Default::default()
        });
        pass.set_pipeline(self.pipeline.get(coverage.is_some()));
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..*count, 0..1);
    }
}
