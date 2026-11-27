use crate::{
    compositor::{Compositor, Target},
    paint::{Pixels, Snapshot},
};
use std::sync::Arc;

#[test]
fn bgra_premultiplication_and_hdr_coverage_preserve_the_underlying_picture() {
    let descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    let instance = wgpu::Instance::new(descriptor.with_env());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .expect("A Vulkan or DX12 adapter is required for the compositor test");
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let size = wgpu::Extent3d {
        width: 3,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = |format| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    };
    let color = texture(wgpu::TextureFormat::Rgba8Unorm);
    let coverage = texture(wgpu::TextureFormat::R8Unorm);
    let color_view = color.create_view(&Default::default());
    let coverage_view = coverage.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    let attachment = |view, clear| wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(clear),
            store: wgpu::StoreOp::Store,
        },
    };
    {
        let attachments = [
            Some(attachment(
                &color_view,
                wgpu::Color {
                    r: 0.,
                    g: 0.,
                    b: 1.,
                    a: 1.,
                },
            )),
            Some(attachment(
                &coverage_view,
                wgpu::Color {
                    r: 0.25,
                    g: 0.,
                    b: 0.,
                    a: 0.,
                },
            )),
        ];
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &attachments,
            ..Default::default()
        });
    }
    // Top-down CEF BGRA: half-transparent red, transparent, opaque green.
    let snapshot = Snapshot {
        revision: 1,
        view: Some(Arc::new(
            Pixels::copy(3, 1, &[0, 0, 128, 128, 0, 0, 0, 0, 0, 255, 0, 255]).unwrap(),
        )),
        popup: None,
    };
    Compositor::new(&device)
        .draw(
            &mut Target {
                device: &device,
                queue: &queue,
                encoder: &mut encoder,
                color: &color_view,
                color_format: wgpu::TextureFormat::Rgba8Unorm,
                coverage: Some(&coverage_view),
                size: [3, 1],
            },
            &snapshot,
        )
        .unwrap();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 512,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    for (image, offset) in [(&color, 0), (&coverage, 256)] {
        encoder.copy_texture_to_buffer(
            image.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset,
                    bytes_per_row: Some(256),
                    rows_per_image: None,
                },
            },
            size,
        );
    }
    queue.submit([encoder.finish()]);
    buffer.map_async(wgpu::MapMode::Read, .., |result| result.unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let bytes = buffer.get_mapped_range(..).unwrap();
    assert_eq!(
        &bytes[..12],
        &[128, 0, 127, 255, 0, 0, 255, 255, 0, 255, 0, 255]
    );
    assert!(bytes[256].abs_diff(160) <= 1);
    assert_eq!(&bytes[257..259], &[64, 255]);
}
