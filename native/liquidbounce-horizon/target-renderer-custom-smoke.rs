#[cfg(test)]
mod custom_file_smoke {
    use super::*;
    #[test]
    fn async_custom_png_failed_load_retains_last_and_path_change_retries() {
        let instance=wgpu::Instance::default();let adapter=pollster::block_on(instance.request_adapter(&Default::default())).unwrap();let(device,queue)=pollster::block_on(adapter.request_device(&Default::default())).unwrap();let mut r=TargetRenderer::new(&device,wgpu::TextureFormat::Rgba8Unorm,false);
        let original=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("custom-original.png");let broken=original.with_file_name("custom-broken.png");std::fs::write(&broken,b"invalid png").unwrap();
        fn frame(r:&mut TargetRenderer,d:&wgpu::Device,q:&wgpu::Queue,p:&std::path::Path){begin_frame(&serde_json::json!({"quads":[{"corners":[[0.,0.,0.5],[0.,1.,0.5],[1.,1.,0.5],[1.,0.,0.5]],"colors":[[1.,1.,1.,1.],[1.,1.,1.,1.],[1.,1.,1.,1.],[1.,1.,1.,1.]],"uv":[[0.,0.],[0.,1.],[1.,1.],[1.,0.]],"shape":1,"inner":0.,"inner_color":[1.,1.,1.,1.],"texture":p.to_string_lossy(),"depth":false}]}).to_string());r.prepare(d,q,Mat4::IDENTITY,DVec3::ZERO,false);}
        for _ in 0..200{frame(&mut r,&device,&queue,&original);if r.loading.is_none(){break;}std::thread::sleep(std::time::Duration::from_millis(5));}assert_eq!(r.custom_last.as_deref(),original.to_str());assert!(r.pending());
        for _ in 0..200{frame(&mut r,&device,&queue,&broken);if r.loading.is_none(){break;}std::thread::sleep(std::time::Duration::from_millis(5));}assert!(take_error().unwrap().contains("custom-broken.png"));assert_eq!(r.custom_last.as_deref(),original.to_str());assert!(r.pending());
        frame(&mut r,&device,&queue,std::path::Path::new(""));assert_eq!(r.custom_last.as_deref(),original.to_str());assert!(r.pending());std::fs::copy(&original,&broken).unwrap();
        for _ in 0..200{frame(&mut r,&device,&queue,&broken);if r.loading.is_none(){break;}std::thread::sleep(std::time::Duration::from_millis(5));}assert_eq!(r.custom_last.as_deref(),broken.to_str());assert!(take_error().is_none());assert!(r.pending());
    }
}
