#[test]
fn six_modes_source_geometry_selection_animation_damage_and_image() {
    use crate::hats::{Entity, Hats};
    let e = Entity {
        local: true,
        friend: false,
        position: [0.; 3],
        interpolated: [0.; 3],
        rotation: [0., 0.],
        height: 1.8,
        eye_height: 1.62,
        armor: false,
        hurt: false,
    };
    let mut s = Hats::default();
    for (mode, count) in [128, 1728, 48, 1920, 1920, 0].into_iter().enumerate() {
        s.mode = mode;
        let mesh = s.mesh(&[e], [0.; 3], true, false, 0, 0);
        assert_eq!(mesh.quads.len(), count, "{}", hats::MODES[mode]);
        assert_eq!(mesh.image_quads.len(), usize::from(mode == 5));
        for q in mesh.quads {
            assert!(q.iter().flat_map(|p| p.position).all(f64::is_finite));
        }
    }
    s.mode = 0;
    let mesh = s.mesh(&[e], [0.; 3], true, false, 0, 0);
    assert!((mesh.quads[0][2].position[1] - 2.3).abs() < 1e-6);
    assert!((mesh.quads[0][0].position[0] - 0.6).abs() < 1e-6);
    let mut hurt = e;
    hurt.hurt = true;
    assert_eq!(
        s.mesh(&[hurt], [0.; 3], true, false, 0, 0).quads[0][0].color,
        [1., 0., 0., 125. / 255.]
    );
    s.render.shader = true;
    assert_eq!(
        s.mesh(&[hurt], [0.; 3], true, false, 0, 0).quads[0][0].color[3],
        1.
    );
    s.render.shader = false;
    let mut armored = e;
    armored.armor = true;
    let q = s.mesh(&[armored], [0.; 3], true, false, 0, 0).quads[0];
    assert!((q[0].position[1] - 2.1).abs() < 1e-6);
    s.modes[0].first_person = false;
    assert!(!s.wears(&e, [0.; 3], true, false));
    assert!(s.wears(&e, [0.; 3], true, true));
    assert!(s.wears(&e, [0.; 3], false, false));
    let mut friend = e;
    friend.local = false;
    friend.friend = true;
    friend.position = [64., 0., 0.];
    assert!(!s.wears(&friend, [0.; 3], true, false));
    friend.position = [63., 0., 0.];
    assert!(s.wears(&friend, [0.; 3], true, false));
    friend.friend = false;
    assert!(!s.wears(&friend, [0.; 3], true, false));
    s.height = vec![[0., 0.], [1., 2.]];
    assert_eq!(s.height(500), 1.);
    assert_eq!(s.height(1500), 1.);
    assert_eq!(s.height(2000), 0.);
    s.symmetric = false;
    assert_eq!(s.height(1500), 1.);
    s.modes[0].first_person = true;
    s.modes[0].follow_rotation = true;
    let mut rotated = e;
    rotated.rotation = [180., 90.];
    let q = s.mesh(&[rotated], [0.; 3], true, false, 0, 0).quads[0];
    assert!(q[2].position[2] < -0.4);
    let mut colors = hats::Colors::default();
    colors.sync = false;
    colors.first = 0x7d000000;
    colors.second = 0x7dffffff;
    assert_eq!(
        colors.at(0., false, 0),
        [127. / 255., 127. / 255., 127. / 255., 125. / 255.]
    );
    assert_ne!(colors.at(0., false, 0), colors.at(0., false, 2500));
    assert_eq!(hats::rotation_angle(180000, -1.), -180.00002);
    s.mode = 5;
    s.render.shader = true;
    assert!(s.effect().is_none());
}
#[test]
fn production_native_game_snapshots_and_engine_bridge() {
    let mut g = crate::game::Game::default();
    g.tick();
    g.lan.as_mut().unwrap().hub.guests[0].yaw = 60.;
    g.frame(0.5);
    let input: serde_json::Value = serde_json::from_str(&g.input.borrow()).unwrap();
    let e = input["entities"].as_array().unwrap();
    assert_eq!(e.len(), 3);
    assert_eq!(e[0]["rotation"], serde_json::json!([10., 5.]));
    assert_eq!(e[1]["rotation"], serde_json::json!([60., 10.]));
    assert_eq!(e[2]["rotation"], serde_json::json!([40., 0.]));
    assert_eq!(e[2]["interpolated"], serde_json::json!([1.5, 0., 0.]));
    assert_eq!(e[1]["armor"], true);
    assert_eq!(e[1]["hurt"], true);
    let mut geometry = String::new();
    g.hooks.hats(
        &g,
        &mut crate::mod_api::ClientHatsEvent {
            input: &g.input.borrow(),
            geometry: &mut geometry,
        },
    );
    let result: serde_json::Value = serde_json::from_str(&geometry).unwrap();
    assert_eq!(result["quads"].as_array().unwrap().len(), 384);
    g.reset();
}
#[test]
fn image_png_jpeg_upload_and_gpu_draw_with_failed_load_retaining_texture() {
    use serde_json::json;
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).expect("GPU adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut renderer = render::HatsRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, false);
    let target = device.create_texture(&wgpu::TextureDescriptor {
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
    let view = target.create_view(&Default::default());
    let read = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 64 * 256,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    for path in [
        concat!(env!("CARGO_MANIFEST_DIR"), "/image.png"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/image.jpg"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/invalid.png"),
    ] {
        let frame = json!({"quads":[],"lines":[],"effect":null,"image":{"path":path,"color":[0.5,1.,0.5,0.5],"quads":[[[-0.5,-0.5,0.5],[-0.5,0.5,0.5],[0.5,0.5,0.5],[0.5,-0.5,0.5]]]}});
        render::begin_frame(&frame.to_string());
        renderer.prepare(
            &device,
            &queue,
            glam::Mat4::IDENTITY,
            glam::DVec3::ZERO,
            [64, 64],
            false,
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while renderer.image_loading() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(1));
            render::begin_frame(&frame.to_string());
            renderer.prepare(
                &device,
                &queue,
                glam::Mat4::IDENTITY,
                glam::DVec3::ZERO,
                [64, 64],
                false,
            );
        }
        assert!(!renderer.image_loading(), "image loading timed out");
        assert!(renderer.pending());
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
            0.,
            glam::DVec3::ZERO,
            glam::Mat4::IDENTITY,
            70.,
        );
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
            let pixel = &pixels[(32 * 64 + 32) * 4..][..4];
            assert!(pixel[3] > 0 && pixel[3] < 180, "{pixel:?}");
            assert!(
                pixel[1] > pixel[0],
                "image color modulator not applied: {pixel:?}"
            );
        }
        read.unmap();
    }
    assert!(render::take_error().unwrap().contains("invalid.png"));
    assert!(render::take_error().is_none());
}
