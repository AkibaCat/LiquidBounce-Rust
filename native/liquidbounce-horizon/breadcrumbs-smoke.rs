#[cfg(test)]
mod tests {
    use super::*;
    use breadcrumbs::{Breadcrumbs, Entity};
    use serde_json::json;
    fn entity(id: u64, own: bool, alive: bool, x: f64) -> Entity {
        Entity {
            identity: id,
            own,
            alive,
            position: [x, 0., 0.],
            interpolated: [x + 0.25, 0., 0.],
        }
    }
    #[test]
    fn source_lifetimes_fade_and_detached_dead_trails() {
        let mut s = Breadcrumbs::default();
        s.only_own = false;
        s.tick(
            0,
            &[
                entity(1, true, true, 0.),
                entity(2, false, true, 0.),
                entity(3, false, true, 0.),
            ],
        );
        s.tick(
            100,
            &[
                entity(1, true, true, 1.),
                entity(2, false, true, 1.),
                entity(3, false, true, 1.),
            ],
        );
        s.only_own = true;
        s.tick(
            200,
            &[
                entity(1, true, true, 2.),
                entity(2, false, true, 2.),
                entity(3, false, false, 2.),
            ],
        );
        let g = s.render(300, 0., &[entity(1, true, true, 2.)]);
        assert_eq!(g.quads.len(), 3);
        assert_eq!(g.quads[1][0].position, [2.25, 0., 0.]);
        assert!((g.quads[0][0].color[3] - (120. / 255.) * (1. - 200. / 900.)).abs() < 1e-6);
        s.height = 0.;
        assert_eq!(
            s.render(1000, 0., &[entity(1, true, true, 2.)]).lines.len(),
            1
        );
        assert!(s.render(1001, 0., &[]).lines.is_empty());
        s.temporary = false;
        s.tick(2000, &[entity(1, true, true, 3.)]);
        s.tick(2001, &[entity(1, true, true, 4.)]);
        assert!(!s.render(99999, 0., &[]).lines.is_empty());
        s.clear();
        assert!(s.render(100000, 0., &[]).lines.is_empty());
    }
    #[test]
    fn engine_emits_actual_geometry_and_all_style_uniforms() {
        let hooks = runtime::LiquidBounceHooks::default();
        let host = mod_api::NullHost;
        let mut geometry = String::new();
        for x in [0., 1.] {
            let input=json!({"tick":true,"entities":[{"identity":1,"own":true,"alive":true,"position":[x,0.,0.],"interpolated":[x,0.,0.]}]}).to_string();
            hooks.breadcrumbs(
                &host,
                &mut mod_api::ClientBreadcrumbsEvent {
                    input: &input,
                    geometry: &mut geometry,
                },
            );
        }
        let input=json!({"entities":[{"identity":1,"own":true,"alive":true,"position":[1.,0.,0.],"interpolated":[1.25,0.,0.]}]}).to_string();
        hooks.breadcrumbs(
            &host,
            &mut mod_api::ClientBreadcrumbsEvent {
                input: &input,
                geometry: &mut geometry,
            },
        );
        let output: serde_json::Value = serde_json::from_str(&geometry).unwrap();
        assert_eq!(output["quads"][0][0]["position"][0], 1.25);
        assert!(output["effect"].is_null());
        let mut s = Breadcrumbs::default();
        s.shader = true;
        for i in 0..9 {
            s.style = i;
            let e = s.effect().unwrap();
            assert!(e.range >= 1. && e.range <= 64.);
            assert_eq!(s.styles[i].get("Fill"), 0.15);
        }
        assert_eq!(breadcrumbs::rainbow(0.), [1., 0., 0., 1.]);
    }
    #[test]
    fn original_styles_compile_and_draw_on_gpu() {
        for i in 0..11 {
            render::fragment_module(i).unwrap();
        }
        let instance = wgpu::Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&Default::default())).expect("GPU adapter");
        eprintln!("Breadcrumbs adapter: {:?}", adapter.get_info());
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut renderer =
            render::BreadcrumbRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, false);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64 * 256,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        for style in 0usize..10 {
            let settings = breadcrumbs::StyleSettings::new(style.saturating_sub(1));
            let effect = if style == 0 {
                serde_json::Value::Null
            } else {
                let e = settings.effect(style - 1);
                json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity})
            };
            let p = |x, y, a| json!({"position":[x,y,0.5],"color":[0.2,0.5,1.,a]});
            let frame = json!({"quads":[[p(-0.5,-0.5,0.3),p(0.5,-0.5,1.),p(0.5,0.5,1.),p(-0.5,0.5,0.3)]],"lines":[[p(-0.8,-0.7,0.2),p(0.8,-0.7,1.)]],"effect":effect});
            render::begin_frame(&frame.to_string());
            renderer.prepare(
                &device,
                glam::Mat4::IDENTITY,
                glam::DVec3::ZERO,
                [64, 64],
                false,
            );
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let _p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
            }
            renderer.draw(
                &device,
                &mut encoder,
                &view,
                0.1,
                glam::DVec3::ZERO,
                glam::Mat4::IDENTITY,
                70.,
            );
            encoder.copy_texture_to_buffer(
                texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &read,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: None,
                    },
                },
                texture.size(),
            );
            queue.submit([encoder.finish()]);
            read.slice(..)
                .map_async(wgpu::MapMode::Read, |r| r.unwrap());
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            {
                let bytes = read.slice(..).get_mapped_range().unwrap();
                assert!(
                    bytes.chunks_exact(4).any(|p| p[3] != 0),
                    "style {style} drew no pixels"
                );
            }
            read.unmap();
        }
    }
}
