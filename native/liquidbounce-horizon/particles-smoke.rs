#[cfg(test)]
mod smoke {
    use super::*;
    use particles::{Particle, Particles};
    use serde_json::{Value, json};
    fn particle() -> Particle {
        Particle {
            position: [0.5, 0.5, 0.5],
            previous: [0.5; 3],
            velocity: [0.1, 0.1, 0.1],
            alpha: 1.,
            rotation: 0.,
            texture: 1,
            spawned: 1_000,
            collision: None,
        }
    }
    #[test]
    fn attack_strict_chronometer_batches_and_retained_clock() {
        let mut p = Particles::default();
        p.settings.count = [2, 2];
        assert_eq!(p.attack(1_000, [0.; 3], 3., [0., 0., 1.]), 2);
        assert_eq!(p.particles[0].position, [0., 0., 3.]);
        assert_eq!(p.attack(1230, [0.; 3], 0., [0.; 3]), 0);
        assert_eq!(p.attack(1231, [0.; 3], 0., [0.; 3]), 2);
        for e in &p.particles {
            assert_eq!(e.texture, 1);
            assert!((-0.01..=0.01).contains(&e.velocity[0]));
            assert!((0.01..=0.02).contains(&e.velocity[1]));
            assert!((0.0..360.).contains(&e.rotation));
        }
        p.clear();
        assert_eq!(p.last_attack, 1231);
        assert_eq!(p.attack(1400, [0.; 3], 0., [0.; 3]), 0);
        p.settings.textures = (1 << 11) - 1;
        for now in (2000..12000).step_by(231) {
            p.attack(now, [0.; 3], 0., [0.; 3]);
        }
        assert_eq!(
            p.particles
                .iter()
                .map(|p| p.texture)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            11
        );
    }
    #[test]
    fn point_cell_collision_priority_drag_gravity_and_deferred_removal() {
        let mut p = Particles::default();
        p.settings.gravity = 0.;
        p.settings.motion = 10.;
        p.particles = vec![particle()];
        p.tick(1000, [0.; 3], |c| c[0] >= 1 || c[1] >= 1 || c[2] >= 1);
        assert!(p.particles[0].velocity[0] < 0.);
        assert_eq!(p.particles[0].velocity[1], 0.1);
        assert_eq!(p.particles[0].velocity[2], 0.1);
        assert_eq!(p.particles[0].collision, Some(1000));
        p.tick(4000, [0.; 3], |_| false);
        assert_eq!(p.particles.len(), 1);
        assert_eq!(p.particles[0].alpha, 0.);
        p.tick(4001, [0.; 3], |_| false);
        assert!(p.particles.is_empty());
        let mut e = particle();
        e.position[1] = 0.99;
        e.velocity = [0.01, 0.1, 0.01];
        p.particles = vec![e];
        p.tick(2000, [0.; 3], |c| c[1] >= 1);
        let e = &p.particles[0];
        assert!((e.velocity[0] - 0.01 * f64::from(p.settings.drag)).abs() < 1e-12);
        assert!(e.velocity[1] < 0.);
        p.particles = vec![particle()];
        p.settings.gravity = 0.8;
        p.tick(2000, [0.; 3], |_| false);
        assert_eq!(p.particles[0].velocity[0], 0.1);
        assert!((p.particles[0].velocity[1] - (0.1 - f64::from(0.8f32) * 0.03125)).abs() < 1e-12);
        p.particles[0].position = [30., 0., 0.];
        p.tick(2001, [0.; 3], |_| false);
        assert!(p.particles.is_empty());
    }
    #[test]
    fn billboard_uv_interpolation_negative_size_and_alpha_truncation() {
        let mut p = Particles::default();
        let mut e = particle();
        e.previous = [0.; 3];
        e.position = [2., 4., 6.];
        e.alpha = 0.5;
        p.particles = vec![e];
        p.settings.color = 0x7f4677ff;
        p.settings.rotate = false;
        let v = p.geometry(1000, 0.5, [1., 0., 0.], [0., 1., 0.]);
        let image: render::Image = serde_json::from_value(v["images"][0].clone()).unwrap();
        assert!((image.color[3] - 63. / 255.).abs() < 1e-6);
        let vertices = render::vertices(&image, glam::Mat4::IDENTITY, glam::DVec3::ZERO);
        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].uv, [0., 0.]);
        assert_eq!(vertices[1].uv, [0., 1.]);
        assert_eq!(vertices[2].uv, [1., 1.]);
        assert_eq!(vertices[0].clip[2], 3.);
        let late = p.geometry(25000, 0.5, [1., 0., 0.], [0., 1., 0.]);
        assert_ne!(v["images"][0]["corners"], late["images"][0]["corners"]);
        assert!(
            late["images"][0]["corners"][1][0].as_f64().unwrap()
                > late["images"][0]["corners"][0][0].as_f64().unwrap()
        );
    }
    #[test]
    fn actual_runtime_target_rotation_and_native_camera_bridge() {
        let mut g = game::Game::default();
        g.entities.insert(2, [3., 0., 4.]);
        g.hooks.particles_attack(&g, 2);
        assert_eq!(
            g.hooks.state().behaviors.borrow().particles.particles.len(),
            2
        );
        let context: Value = serde_json::from_str(&g.native_client_particles_context(2)).unwrap();
        assert_eq!(context["distance"], 5.);
        assert_eq!(context["eye"][1], f64::from(1.62f32));
        {
            let state = g.hooks.state();
            let mut m = state.behaviors.borrow_mut();
            m.particles.clear();
            m.particles.last_attack = 0;
            m.rotations.lock().unwrap().current = Some(runtime::Rotation {
                yaw: 90.,
                pitch: 0.,
            });
        }
        g.hooks.particles_attack(&g, 2);
        let p = g.hooks.state().behaviors.borrow().particles.particles[0].position;
        assert!(p[0] < -4.99);
        assert!(p[2].abs() < 0.01);
        g.frame(0.5);
        assert!(g.geometry.borrow()["images"].as_array().unwrap().len() > 0);
        g.tick();
        {
            let mut state = g.hooks.state();
            state.friends.names.insert("friend".into(), None);
            state.behaviors.borrow_mut().particles.clear();
            state.behaviors.borrow_mut().particles.last_attack = 0;
        }
        g.player_name = Some("friend".into());
        g.hooks.particles_attack(&g, 2);
        assert!(
            g.hooks
                .state()
                .behaviors
                .borrow()
                .particles
                .particles
                .is_empty()
        );
        g.hooks.particles(
            &g,
            &mut mod_api::ClientParticlesEvent {
                input: "{\"action\":\"reset\"}",
                geometry: &mut String::new(),
            },
        );
    }
    #[test]
    fn builtin_png_upload_with_real_depth_occlusion() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut renderer =
            render::ParticlesRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, false);
        let desc = wgpu::TextureDescriptor {
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
        };
        let target = device.create_texture(&desc);
        let view = target.create_view(&Default::default());
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ..desc
        });
        let depth_view = depth.create_view(&Default::default());
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64 * 256,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let images=(0..11).map(|texture|json!({"texture":texture,"corners":[[-0.5,0.5,0.5],[-0.5,-0.5,0.5],[0.5,-0.5,0.5],[0.5,0.5,0.5]],"color":[0.5,1.,0.5,0.5]})).collect::<Vec<_>>();
        for scene_depth in [0.75, 0.] {
            render::begin_frame(&json!({"images":images}).to_string());
            renderer.prepare(
                &device,
                &queue,
                glam::Mat4::IDENTITY,
                glam::DVec3::ZERO,
                false,
            );
            assert!(renderer.pending());
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(scene_depth),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });
            }
            renderer.draw(&device, &mut encoder, &view, &depth_view);
            encoder.copy_texture_to_buffer(
                target.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &read,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: None,
                    },
                },
                target.size(),
            );
            queue.submit([encoder.finish()]);
            read.slice(..)
                .map_async(wgpu::MapMode::Read, |r| r.unwrap());
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            {
                let pixels = read.slice(..).get_mapped_range().unwrap();
                let alpha = pixels.chunks(4).map(|p| u64::from(p[3])).sum::<u64>();
                if scene_depth > 0. {
                    assert_eq!(alpha, 0);
                } else {
                    assert!(alpha > 0);
                }
            }
            read.unmap();
        }
    }
}
