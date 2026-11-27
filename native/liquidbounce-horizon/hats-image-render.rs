// SPDX-License-Identifier: GPL-3.0-or-later
#[derive(Clone, Debug, serde::Deserialize)]
pub struct ImageFrame {
    pub path: String,
    pub quads: Vec<[[f64; 3]; 4]>,
    pub color: [f32; 4],
}
thread_local! {static IMAGE_ERROR:RefCell<Option<String>>=const{RefCell::new(None)};}
pub fn take_error() -> Option<String> {
    IMAGE_ERROR.with_borrow_mut(Option::take)
}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVertex {
    pub clip: [f32; 4],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}
pub fn image_vertices(frame: &ImageFrame, projection: Mat4, camera: DVec3) -> Vec<ImageVertex> {
    let mut out = vec![];
    for q in &frame.quads {
        let uv = [[0., 0.], [0., 1.], [1., 1.], [1., 0.]];
        let v: [ImageVertex; 4] = std::array::from_fn(|i| ImageVertex {
            clip: (projection * (DVec3::from_array(q[i]) - camera).as_vec3().extend(1.)).to_array(),
            color: frame.color,
            uv: uv[i],
        });
        for i in [0, 1, 2, 0, 2, 3] {
            out.push(v[i]);
        }
    }
    out
}
struct ImageRenderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    attempted: String,
    loading: Option<(
        String,
        std::sync::mpsc::Receiver<Result<(u32, u32, Vec<u8>), String>>,
    )>,
    texture: Option<(wgpu::Texture, wgpu::BindGroup)>,
    pending: Option<(wgpu::Buffer, u32)>,
}
impl ImageRenderer {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat, linear: bool) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hats Image"),
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
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hats Image"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hats Image"),
            source: wgpu::ShaderSource::Wgsl(include_str!("hats-image.wgsl").into()),
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x2];
        let constants = [(
            "FRAME_LINEAR",
            f64::from(u8::from(
                linear && format == wgpu::TextureFormat::Rgba16Float,
            )),
        )];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hats Image"),
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
        Self {
            pipeline,
            layout,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                ..Default::default()
            }),
            attempted: String::new(),
            loading: None,
            texture: None,
            pending: None,
        }
    }
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: Option<ImageFrame>,
        projection: Mat4,
        camera: DVec3,
        title: bool,
    ) {
        self.pending = None;
        let Some(frame) = frame.filter(|_| !title) else {
            return;
        };
        if frame.quads.is_empty() {
            return;
        }
        // FileValue's state flow filters non-files; decoding errors keep the last good texture.
        let changed = frame.path != self.attempted;
        self.attempted = frame.path.clone();
        if changed && std::path::Path::new(&frame.path).is_file() {
            let path = frame.path.clone();
            let (sender, receiver) = std::sync::mpsc::channel();
            self.loading = Some((path.clone(), receiver));
            std::thread::spawn(move || {
                let decoded = std::fs::read(&path)
                    .map_err(|e| e.to_string())
                    .and_then(|b| {
                        if b.starts_with(&[0xff, 0xd8]) {
                            crate::hud::decode_image_bytes(&b)
                        } else {
                            crate::hud::decode_bytes(&b)
                        }
                        .map_err(|e| e.to_string())
                    });
                let _ = sender.send(decoded);
            });
        }
        let decoded = self.loading.as_ref().and_then(|(path, receiver)| {
            receiver
                .try_recv()
                .ok()
                .map(|result| (path.clone(), result))
        });
        if let Some((path, decoded)) = decoded {
            self.loading = None;
            match decoded {
                Ok((width, height, data)) => {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Hats file texture"),
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
                        label: Some("Hats file texture"),
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
                    self.texture = Some((texture, group));
                }
                Err(error) => IMAGE_ERROR.with_borrow_mut(|e| {
                    *e = Some(format!(
                        "Failed to load texture from '{}' for Image ({error})",
                        std::path::Path::new(&path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    ))
                }),
            }
        }
        if self.texture.is_none() {
            return;
        }
        let v = image_vertices(&frame, projection, camera);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hats image mesh"),
            contents: bytemuck::cast_slice(&v),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.pending = Some((buffer, v.len() as u32));
    }
    fn draw(&mut self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let Some((buffer, count)) = self.pending.take() else {
            return;
        };
        let Some((_, group)) = &self.texture else {
            return;
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hats Image"),
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
        pass.set_bind_group(0, group, &[]);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..count, 0..1);
    }
}
pub struct HatsRenderer {
    shapes: HatsShapeRenderer,
    images: ImageRenderer,
}
impl HatsRenderer {
    #[cfg(test)]
    pub fn image_loading(&self) -> bool {
        self.images.loading.is_some()
    }
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, linear: bool) -> Self {
        Self {
            shapes: HatsShapeRenderer::new(device, format, linear),
            images: ImageRenderer::new(device, format, linear),
        }
    }
    pub fn pending(&self) -> bool {
        self.shapes.pending() || self.images.pending.is_some()
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        projection: Mat4,
        camera: DVec3,
        size: [u32; 2],
        title: bool,
    ) {
        let images = PENDING.with_borrow(|p| p.as_ref().and_then(|f| f.image.clone()));
        self.images
            .prepare(device, queue, images, projection, camera, title);
        self.shapes.prepare(device, projection, camera, size, title);
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
        self.shapes
            .draw(device, encoder, target, game_time, camera, view, fov);
        self.images.draw(encoder, target);
    }
}
