// SPDX-License-Identifier: GPL-3.0-or-later
// Image file flow: new valid paths replace pending decode; failure/nonfile retains last success.
include!("jump-effect-textures.rs");
thread_local! {static IMAGE_ERROR:RefCell<Option<String>>=const{RefCell::new(None)};}
pub fn take_error() -> Option<String> {
    IMAGE_ERROR.with_borrow_mut(Option::take)
}
struct Images {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: std::collections::BTreeMap<String, (wgpu::Texture, wgpu::BindGroup)>,
    selected: Option<String>,
    loading: Option<(
        String,
        std::sync::mpsc::Receiver<Result<(u32, u32, Vec<u8>), String>>,
    )>,
    last: Option<String>,
}
impl Images {
    fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("JumpEffect texture"),
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
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            layout,
            sampler,
            textures: Default::default(),
            selected: None,
            loading: None,
            last: None,
        }
    }
    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: String,
        width: u32,
        height: u32,
        data: &[u8],
    ) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("JumpEffect image"),
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
            data,
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
            label: Some("JumpEffect texture"),
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
        if !path.starts_with("builtin:") {
            self.textures.retain(|p, _| p.starts_with("builtin:"));
        }
        self.textures.insert(path, (texture, group));
    }
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
    ) -> Option<String> {
        if !path.starts_with("builtin:") && self.selected.as_deref() != Some(path) {
            self.selected = Some(path.into());
            if std::path::Path::new(path).is_file() {
                let path = path.to_owned();
                let (tx, rx) = std::sync::mpsc::channel();
                self.loading = Some((path.clone(), rx));
                std::thread::spawn(move || {
                    let result = std::fs::read(&path)
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
        let ready = self
            .loading
            .as_ref()
            .and_then(|(path, rx)| rx.try_recv().ok().map(|result| (path.clone(), result)));
        if let Some((path, result)) = ready {
            self.loading = None;
            match result {
                Ok((w, h, data)) => {
                    self.upload(device, queue, path.clone(), w, h, &data);
                    self.last = Some(path);
                }
                Err(e) => IMAGE_ERROR.with_borrow_mut(|v| {
                    *v = Some(format!(
                        "Failed to load texture from '{}' for JumpEffect Image ({e})",
                        std::path::Path::new(&path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    ))
                }),
            }
        }
        if !path.starts_with("builtin:") {
            return self.last.clone();
        }
        if !self.textures.contains_key(path) {
            let decoded = if path == "builtin:White" {
                Some((1, 1, vec![255; 4]))
            } else {
                let index = match path {
                    "builtin:LiquidBounce" => 0,
                    "builtin:LiquidBounceWithLogo" => 1,
                    _ => return None,
                };
                crate::skins::base64_decode(PNGS[index])
                    .and_then(|b| crate::hud::decode_bytes(&b).ok())
            };
            if let Some((w, h, data)) = decoded {
                self.upload(device, queue, path.into(), w, h, &data);
            }
        }
        self.textures.contains_key(path).then(|| path.into())
    }
}
