use crate::paint::{Pixels, Snapshot};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct Target<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub color: &'a wgpu::TextureView,
    pub color_format: wgpu::TextureFormat,
    pub coverage: Option<&'a wgpu::TextureView>,
    pub size: [u32; 2],
}

struct Image {
    texture: wgpu::Texture,
    geometry: wgpu::Buffer,
    group: wgpu::BindGroup,
}

/// GPU resources are owned by the render thread. CEF only publishes immutable CPU images.
pub struct Compositor {
    device: wgpu::Device,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    shader: wgpu::ShaderModule,
    pipelines: HashMap<(wgpu::TextureFormat, bool), wgpu::RenderPipeline>,
    revision: Option<u64>,
    view: Option<Image>,
    popup: Option<Image>,
}

impl Compositor {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("LiquidBounce CEF image layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("LiquidBounce CEF sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LiquidBounce CEF compositor"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compositor.wgsl").into()),
        });
        Self {
            device: device.clone(),
            layout,
            sampler,
            shader,
            pipelines: HashMap::new(),
            revision: None,
            view: None,
            popup: None,
        }
    }

    pub fn uses_device(&self, device: &wgpu::Device) -> bool {
        self.device == *device
    }

    fn image(&self, pixels: &Pixels, bounds: [f32; 4]) -> Image {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("LiquidBounce independent CEF BGRA surface"),
            size: wgpu::Extent3d {
                width: pixels.width,
                height: pixels.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // CEF pixels are gamma encoded and premultiplied. An sRGB view would decode twice.
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let geometry = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("LiquidBounce CEF image bounds"),
                contents: bytemuck::cast_slice(&bounds),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("LiquidBounce CEF image"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: geometry.as_entire_binding(),
                },
            ],
        });
        Image {
            texture,
            geometry,
            group,
        }
    }

    fn upload(
        &self,
        queue: &wgpu::Queue,
        image: &mut Option<Image>,
        pixels: &Pixels,
        bounds: [f32; 4],
    ) {
        if image.as_ref().is_none_or(|i| {
            i.texture.width() != pixels.width || i.texture.height() != pixels.height
        }) {
            *image = Some(self.image(pixels, bounds));
        }
        let image = image.as_ref().unwrap();
        queue.write_buffer(&image.geometry, 0, bytemuck::cast_slice(&bounds));
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &image.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels.bgra,
            // Queue writes accept tight rows; command-buffer buffer copies require 256 alignment.
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pixels.width * 4),
                rows_per_image: Some(pixels.height),
            },
            wgpu::Extent3d {
                width: pixels.width,
                height: pixels.height,
                depth_or_array_layers: 1,
            },
        );
    }

    fn pipeline(&mut self, format: wgpu::TextureFormat, covered: bool) {
        let key = (format, covered);
        if self.pipelines.contains_key(&key) {
            return;
        }
        let over = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        };
        let mut targets = vec![Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState {
                color: over,
                alpha: over,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        if covered {
            let coverage_over = wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                operation: wgpu::BlendOperation::Add,
            };
            targets.push(Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R8Unorm,
                blend: Some(wgpu::BlendState {
                    color: coverage_over,
                    alpha: coverage_over,
                }),
                write_mask: wgpu::ColorWrites::RED,
            }));
        }
        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("LiquidBounce CEF pipeline layout"),
                bind_group_layouts: &[Some(&self.layout)],
                immediate_size: 0,
            });
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("LiquidBounce final GUI composition"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some(if covered { "fs_coverage" } else { "fs_plain" }),
                    compilation_options: Default::default(),
                    targets: &targets,
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            });
        self.pipelines.insert(key, pipeline);
    }

    pub fn draw(
        &mut self,
        target: &mut Target<'_>,
        snapshot: &Snapshot,
    ) -> Result<(), &'static str> {
        if !self.uses_device(target.device) {
            return Err("CEF compositor belongs to a previous GPU device");
        }
        if target.size.contains(&0) {
            return Ok(());
        }
        let limit = target.device.limits().max_texture_dimension_2d;
        for pixels in snapshot
            .view
            .iter()
            .chain(snapshot.popup.iter().map(|(_, p)| p))
        {
            if pixels.width > limit || pixels.height > limit {
                return Err("CEF surface exceeds GPU texture size limits");
            }
        }
        if self.revision != Some(snapshot.revision) {
            let mut view = self.view.take();
            let mut popup = self.popup.take();
            if let Some(pixels) = &snapshot.view {
                self.upload(target.queue, &mut view, pixels, [0., 0., 1., 1.]);
                if let Some((rect, pixels_popup)) = &snapshot.popup {
                    let w = pixels.width as f32;
                    let h = pixels.height as f32;
                    self.upload(
                        target.queue,
                        &mut popup,
                        pixels_popup,
                        [
                            rect.x as f32 / w,
                            rect.y as f32 / h,
                            rect.width as f32 / w,
                            rect.height as f32 / h,
                        ],
                    );
                } else {
                    popup = None;
                }
            } else {
                view = None;
                popup = None;
            }
            self.view = view;
            self.popup = popup;
            self.revision = Some(snapshot.revision);
        }
        if self.view.is_none() {
            return Ok(());
        }
        self.pipeline(target.color_format, target.coverage.is_some());
        let attachment = |view| wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        };
        let mut attachments = vec![Some(attachment(target.color))];
        if let Some(coverage) = target.coverage {
            attachments.push(Some(attachment(coverage)));
        }
        let mut pass = target
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("LiquidBounce CEF over final HUD"),
                color_attachments: &attachments,
                ..Default::default()
            });
        pass.set_viewport(0., 0., target.size[0] as f32, target.size[1] as f32, 0., 1.);
        pass.set_scissor_rect(0, 0, target.size[0], target.size[1]);
        pass.set_pipeline(&self.pipelines[&(target.color_format, target.coverage.is_some())]);
        for image in self.view.iter().chain(self.popup.iter()) {
            pass.set_bind_group(0, &image.group, &[]);
            pass.draw(0..6, 0..1);
        }
        Ok(())
    }
}
