// SPDX-License-Identifier: GPL-3.0-or-later
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[path="behavior_chams.rs"] pub mod behavior_chams;
#[cfg(test)] mod hud;
#[cfg(test)] mod overlay;
#[cfg(test)] mod dynamic_light {pub const MAX:usize=16;pub type Colored=[[f32;4];2];}
#[cfg(test)] mod native_emissive {pub const MAX:usize=128;}
#[cfg(test)] mod gfx {
    #[repr(C)]
    #[derive(Clone,Copy,Debug,bytemuck::Pod,bytemuck::Zeroable)]
    __VERTEX__
    mod sunrise_fan {pub const TRIANGLES:usize=16;}
    #[repr(C)]
    #[derive(Clone,Copy,bytemuck::Pod,bytemuck::Zeroable)]
    __GLOBALS__
    pub const LINEAR_FRAME:wgpu::TextureFormat=wgpu::TextureFormat::Rgba16Float;
    #[allow(dead_code)] mod upload;
    #[allow(dead_code)] mod linear_blend;
    mod test_gpu;
    pub(crate) mod chams;
    use bytemuck::Zeroable;
    use wgpu::util::DeviceExt;

    #[test]
    fn captured_geometry_is_removed_once_and_metadata_offsets_are_rebased() {
        chams::begin_frame(r#"{"mode":"Normal","file":"","mapping":"Repeat","tileWidth":256,"filtering":"Nearest","offset":[0,0]}"#);
        let quad=|layer|[Vertex{layer,..Zeroable::zeroed()};4];
        let mut vertices=Vec::new();vertices.extend(quad(0));vertices.extend(quad(0));vertices.extend(quad(chams::UNSUPPORTED));vertices.extend(quad(36));vertices.extend(quad(14));
        chams::mark_entity(&mut vertices[4..16]);
        let offsets=chams::finish_entities(&mut vertices);
        assert_eq!(vertices.len(),12);
        assert_eq!([offsets[0],offsets[4],offsets[8],offsets[12],offsets[16],offsets[20]],[0,4,4,8,8,12]);
        assert!(vertices.iter().all(|v|v.layer&(chams::CAPTURE|chams::UNSUPPORTED)==0));
        chams::begin_frame("null");assert!(!chams::active());
        assert!(chams::finish_entities(&mut vertices).is_empty());
        assert_eq!(vertices.len(),12);
    }

    fn gpu()->(wgpu::Device,wgpu::Queue) {
        let instance=test_gpu::instance();
        let adapter=pollster::block_on(instance.request_adapter(&Default::default())).expect("GPU adapter required");
        eprintln!("Chams GPU: {:?}",adapter.get_info());
        assert_ne!(adapter.get_info().device_type,wgpu::DeviceType::Cpu);
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor{required_limits:adapter.limits(),..Default::default()})).unwrap()
    }
    fn texture(device:&wgpu::Device,size:[u32;2],format:wgpu::TextureFormat,usage:wgpu::TextureUsages)->wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor{label:None,size:wgpu::Extent3d{width:size[0],height:size[1],depth_or_array_layers:1},
            mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format,usage,view_formats:&[]})
    }
    fn scene(device:&wgpu::Device,encoder:&mut wgpu::CommandEncoder,size:[u32;2])->(wgpu::Texture,wgpu::TextureView,wgpu::TextureView) {
        scene_in(device,encoder,size,wgpu::TextureFormat::Rgba8Unorm,wgpu::Color::BLUE)
    }
    fn scene_in(device:&wgpu::Device,encoder:&mut wgpu::CommandEncoder,size:[u32;2],format:wgpu::TextureFormat,clear:wgpu::Color)->(wgpu::Texture,wgpu::TextureView,wgpu::TextureView) {
        let color=texture(device,size,format,wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC);
        let view=color.create_view(&Default::default());
        let depth=texture(device,size,wgpu::TextureFormat::Depth32Float,wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::TEXTURE_BINDING).create_view(&Default::default());
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:&view,resolve_target:None,depth_slice:None,
                ops:wgpu::Operations{load:wgpu::LoadOp::Clear(clear),store:wgpu::StoreOp::Store}})],
            depth_stencil_attachment:Some(wgpu::RenderPassDepthStencilAttachment{view:&depth,
                depth_ops:Some(wgpu::Operations{load:wgpu::LoadOp::Clear(0.5),store:wgpu::StoreOp::Store}),stencil_ops:None}),..Default::default()});
        (color,view,depth)
    }
    fn read(device:&wgpu::Device,queue:&wgpu::Queue,mut encoder:wgpu::CommandEncoder,color:&wgpu::Texture,uploads:&mut upload::Uploads)->Vec<[u8;4]> {
        let stride=(color.width()*4).next_multiple_of(256);
        let buffer=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:u64::from(stride*color.height()),
            usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
        encoder.copy_texture_to_buffer(color.as_image_copy(),wgpu::TexelCopyBufferInfo{buffer:&buffer,
            layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(stride),rows_per_image:Some(color.height())}},color.size());
        uploads.finish(&encoder);queue.submit([encoder.finish()]);
        buffer.slice(..).map_async(wgpu::MapMode::Read,|r|r.unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data=buffer.slice(..).get_mapped_range().unwrap();
        let mut out=Vec::new();
        for row in data.chunks(stride as usize) {out.extend(row[..color.width() as usize*4].as_chunks::<4>().0.iter().copied());}
        out
    }
    fn entity_shader(device:&wgpu::Device,queue:&wgpu::Queue)->(wgpu::ShaderModule,wgpu::PipelineLayout,wgpu::BindGroup) {
        let source=crate::overlay::wgsl_constants()+include_str!("hdr_light.wgsl")+include_str!("entity.wgsl");
        let shader=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("Actual Horizon entity shader"),source:wgpu::ShaderSource::Wgsl(source.into())});
        let mut globals=Globals::zeroed();globals.view_proj=glam::Mat4::IDENTITY.to_cols_array_2d();globals.inverse_view=globals.view_proj;
        globals.fog=[1000.0;4];globals.fog_ends=[1000.0;4];
        globals.hdr_light=[1.0,1.0,0.0,1.0];globals.hdr_settings=[10.0,4.0,1.0,0.0];globals.hdr_gains=[2.0,3.0,4.0,5.0];
        let uniform=device.create_buffer_init(&wgpu::util::BufferInitDescriptor{label:None,contents:bytemuck::bytes_of(&globals),usage:wgpu::BufferUsages::UNIFORM});
        let tex=device.create_texture_with_data(queue,&wgpu::TextureDescriptor{label:None,size:wgpu::Extent3d{width:1,height:1,depth_or_array_layers:1},mip_level_count:1,sample_count:1,
            dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8Unorm,usage:wgpu::TextureUsages::TEXTURE_BINDING,view_formats:&[]},wgpu::util::TextureDataOrder::LayerMajor,&[255;4]);
        let view=tex.create_view(&Default::default());let sampler=device.create_sampler(&Default::default());
        let mut entries=vec![wgpu::BindGroupLayoutEntry{binding:0,visibility:wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty:wgpu::BindingType::Buffer{ty:wgpu::BufferBindingType::Uniform,has_dynamic_offset:false,min_binding_size:wgpu::BufferSize::new(std::mem::size_of::<Globals>() as u64)},count:None}];
        for binding in 1..8 {entries.push(wgpu::BindGroupLayoutEntry{binding,visibility:wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty:if matches!(binding,2|4|6){wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)}else{wgpu::BindingType::Texture{
                sample_type:wgpu::TextureSampleType::Float{filterable:true},view_dimension:wgpu::TextureViewDimension::D2,multisampled:false}},count:None});}
        let bindings=device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{label:None,entries:&entries});
        let layout=device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{label:None,bind_group_layouts:&[Some(&bindings)],..Default::default()});
        let mut entries=vec![wgpu::BindGroupEntry{binding:0,resource:uniform.as_entire_binding()}];
        for binding in 1..8 {entries.push(wgpu::BindGroupEntry{binding,resource:if matches!(binding,2|4|6){wgpu::BindingResource::Sampler(&sampler)}else{wgpu::BindingResource::TextureView(&view)}});}
        let group=device.create_bind_group(&wgpu::BindGroupDescriptor{label:None,layout:&bindings,entries:&entries});
        (shader,layout,group)
    }
    fn quad(x:[f32;2],depth:f32,color:[u8;4])->[Vertex;4] {
        [[x[0],-1.0,depth],[x[1],-1.0,depth],[x[1],1.0,depth],[x[0],1.0,depth]]
            .map(|pos|Vertex{pos,color:u32::from_le_bytes(color),..Zeroable::zeroed()})
    }
    fn capture(settings:&str,vertices:&[Vertex]) {
        chams::begin_frame(settings);
        let mut source=vertices.to_vec();chams::mark_entity(&mut source);
        chams::finish_entities(&mut source);assert!(source.is_empty());
    }
    fn png(name:&str,size:[u32;2],pixels:&[u8])->String {
        let path=std::env::temp_dir().join(format!("liquidbounce-chams-{}-{name}.png",std::process::id()));
        let mut encoder=png::Encoder::new(std::fs::File::create(&path).unwrap(),size[0],size[1]);
        encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().unwrap().write_image_data(pixels).unwrap();
        path.to_string_lossy().into_owned()
    }
    fn settings(mode:&str,file:&str,mapping:&str,tile:i32,filtering:&str,offset:[f32;2])->String {
        serde_json::json!({"mode":mode,"file":file,"mapping":mapping,"tileWidth":tile,"filtering":filtering,"offset":offset}).to_string()
    }
    #[allow(clippy::too_many_arguments)]
    fn render(device:&wgpu::Device,queue:&wgpu::Queue,renderer:&mut chams::ChamsRenderer,group:&wgpu::BindGroup,
        uploads:&mut upload::Uploads,size:[u32;2])->Vec<[u8;4]> {
        let mut encoder=device.create_command_encoder(&Default::default());
        let (color,view,depth)=scene(device,&mut encoder,size);
        renderer.prepare(device,queue,uploads,&mut encoder,size,&depth,group,false);
        renderer.composite(&mut encoder,&view);
        // The original compositor is dispatched once per prepared frame.
        renderer.composite(&mut encoder,&view);
        read(device,queue,encoder,&color,uploads)
    }
    #[test]
    fn actual_gpu_capture_and_depth_image_composite() {
        let (device,queue)=gpu();
        let (shader,layout,group)=entity_shader(&device,&queue);
        let mut renderer=chams::ChamsRenderer::new(&shader,&layout,wgpu::TextureFormat::Rgba8Unorm,false);
        let mut uploads=upload::Uploads::new(&device);
        let draw=|renderer:&mut chams::ChamsRenderer,uploads:&mut upload::Uploads,size|render(&device,&queue,renderer,&group,uploads,size);
        chams::begin_frame("null");
        assert!(draw(&mut renderer,&mut uploads,[8,4]).iter().all(|c|*c==[0,0,255,255]));
        assert!(!renderer.pending());
        let mut geometry=quad([-1.0,0.0],0.75,[255,0,0,255]).to_vec();
        geometry.extend(quad([0.0,1.0],0.25,[255,0,0,255]));
        capture(&settings("Normal","","Repeat",256,"Nearest",[0.0;2]),&geometry);
        assert!(draw(&mut renderer,&mut uploads,[8,4]).iter().all(|c|*c==[255,0,0,255]));
        capture(&settings("Image","nonexistent-chams-image.png","Repeat",256,"Nearest",[0.0;2]),&geometry);
        assert!(draw(&mut renderer,&mut uploads,[8,4]).iter().all(|c|*c==[255,0,0,255]));
        let green=png("green",[1,1],&[0,255,0,255]);
        capture(&settings("Image",&green,"Repeat",256,"Nearest",[0.0;2]),&geometry);
        let colors=draw(&mut renderer,&mut uploads,[8,4]);
        for (i,pixel) in colors.iter().enumerate(){assert_eq!(*pixel,if i%8<4{[255,0,0,255]}else{[0,255,0,255]},"at {i}");}
        // The same prepared CPU frame may be rendered again for a high-resolution screenshot.
        let colors=draw(&mut renderer,&mut uploads,[16,6]);
        for (i,pixel) in colors.iter().enumerate(){assert_eq!(*pixel,if i%16<8{[255,0,0,255]}else{[0,255,0,255]},"resized at {i}");}
        capture(&settings("Image","nonexistent-chams-image.png","Repeat",256,"Nearest",[0.0;2]),&geometry);
        let colors=draw(&mut renderer,&mut uploads,[8,4]);
        for (i,pixel) in colors.iter().enumerate(){assert_eq!(*pixel,if i%8<4{[255,0,0,255]}else{[0,255,0,255]},"retained at {i}");}
        capture(&settings("Image",&green,"Repeat",256,"Nearest",[0.0;2]),&quad([-1.0,1.0],0.25,[255,0,0,0]));
        assert!(draw(&mut renderer,&mut uploads,[8,4]).iter().all(|c|*c==[0,0,255,255]));
        // A new empty frame cannot retain captured entities from the previous frame.
        chams::begin_frame(&settings("Normal","","Repeat",256,"Nearest",[0.0;2]));
        assert!(draw(&mut renderer,&mut uploads,[8,4]).iter().all(|c|*c==[0,0,255,255]));
        chams::begin_frame("null");
        assert!(draw(&mut renderer,&mut uploads,[8,4]).iter().all(|c|*c==[0,0,255,255]));
        // A disabled module releases render targets but keeps the file property's last image.
        capture(&settings("Image","nonexistent-chams-image.png","Repeat",256,"Nearest",[0.0;2]),&geometry);
        let colors=draw(&mut renderer,&mut uploads,[8,4]);
        for (i,pixel) in colors.iter().enumerate(){assert_eq!(*pixel,if i%8<4{[255,0,0,255]}else{[0,255,0,255]},"reenabled at {i}");}
        // Re-selecting the previous successful path after another value must
        // upload its newly decoded bytes, even when the path itself is the same.
        assert_eq!(png("green",[1,1],&[255,0,0,255]),green);
        capture(&settings("Image",&green,"Repeat",256,"Nearest",[0.0;2]),&geometry);
        assert!(draw(&mut renderer,&mut uploads,[8,4]).iter().all(|c|*c==[255,0,0,255]));
        let jpeg=std::env::temp_dir().join(format!("liquidbounce-chams-{}-green.jpeg",std::process::id()));
        std::fs::write(&jpeg,include_bytes!("chams-green.jpg")).unwrap();
        let jpeg=jpeg.to_string_lossy().into_owned();
        capture(&settings("Image",&jpeg,"Stretch",256,"Nearest",[0.0;2]),&geometry);
        for (i,pixel) in draw(&mut renderer,&mut uploads,[8,4]).iter().enumerate() {
            if i%8<4 {assert_eq!(*pixel,[255,0,0,255]);}
            else {assert!(pixel[0]<=3 && pixel[1]>=252 && pixel[2]<=3 && pixel[3]==255,"JPEG at {i}: {pixel:?}");}
        }
        // A corrupt JPEG preserves the last successfully decoded JPEG.
        let corrupt=std::env::temp_dir().join(format!("liquidbounce-chams-{}-corrupt.jpg",std::process::id()));
        std::fs::write(&corrupt,b"\xff\xd8\xffbroken-jpeg").unwrap();
        capture(&settings("Image",&corrupt.to_string_lossy(),"Stretch",256,"Nearest",[0.0;2]),&geometry);
        for (i,pixel) in draw(&mut renderer,&mut uploads,[8,4]).iter().enumerate() {
            if i%8<4 {assert_eq!(*pixel,[255,0,0,255]);}
            else {assert!(pixel[0]<=3 && pixel[1]>=252 && pixel[2]<=3 && pixel[3]==255,"retained JPEG at {i}: {pixel:?}");}
        }
        std::fs::remove_file(jpeg).unwrap();std::fs::remove_file(corrupt).unwrap();
        std::fs::remove_file(green).unwrap();
    }
    #[test]
    fn actual_gpu_image_mapping_orientation_offsets_and_filtering() {
        let (device,queue)=gpu();let (shader,layout,group)=entity_shader(&device,&queue);
        let mut renderer=chams::ChamsRenderer::new(&shader,&layout,wgpu::TextureFormat::Rgba8Unorm,false);
        let mut uploads=upload::Uploads::new(&device);
        let draw=|renderer:&mut chams::ChamsRenderer,uploads:&mut upload::Uploads,size|render(&device,&queue,renderer,&group,uploads,size);
        let red=[255,0,0,255];let yellow=[255,255,0,255];let green=[0,255,0,255];let magenta=[255,0,255,255];
        let file=png("mapping",[2,2],&[red,yellow,green,magenta].concat());
        let geometry=quad([-1.0,1.0],0.25,[255;4]);
        capture(&settings("Image",&file,"Stretch",2,"Nearest",[0.0;2]),&geometry);
        assert_eq!(draw(&mut renderer,&mut uploads,[4,4]),[green,green,magenta,magenta,green,green,magenta,magenta,red,red,yellow,yellow,red,red,yellow,yellow]);
        capture(&settings("Image",&file,"Repeat",2,"Nearest",[0.0;2]),&geometry);
        assert_eq!(draw(&mut renderer,&mut uploads,[4,4]),[green,magenta,green,magenta,red,yellow,red,yellow,green,magenta,green,magenta,red,yellow,red,yellow]);
        capture(&settings("Image",&file,"Repeat",2,"Nearest",[1.0;2]),&geometry);
        assert_eq!(draw(&mut renderer,&mut uploads,[4,4]),[yellow,red,yellow,red,magenta,green,magenta,green,yellow,red,yellow,red,magenta,green,magenta,green]);
        let blue=[0,0,255,255];let cyan=[0,255,255,255];
        let wide=png("cover",[4,2],&[red,yellow,blue,[255;4],green,magenta,cyan,[0,0,0,255]].concat());
        capture(&settings("Image",&wide,"Cover",2,"Nearest",[0.0;2]),&geometry);
        assert_eq!(draw(&mut renderer,&mut uploads,[4,4]),[magenta,magenta,cyan,cyan,magenta,magenta,cyan,cyan,yellow,yellow,blue,blue,yellow,yellow,blue,blue]);
        let gradient=png("filtering",[2,1],&[red,green].concat());
        capture(&settings("Image",&gradient,"Stretch",2,"Linear",[0.0;2]),&geometry);
        let colors=draw(&mut renderer,&mut uploads,[4,1]);
        for (pixel,expected) in colors.iter().zip([red,[191,64,0,255],[64,191,0,255],green]) {
            for (actual,want) in pixel.iter().zip(expected){assert!((i32::from(*actual)-i32::from(want)).abs()<=1,"{pixel:?} vs {expected:?}");}
        }
        for path in [file,wide,gradient]{std::fs::remove_file(path).unwrap();}
    }
    #[test]
    fn actual_gpu_classic_transparency_preserves_original_phase_replay_and_item_depth() {
        let (device,queue)=gpu();let (shader,layout,group)=entity_shader(&device,&queue);
        let mut renderer=chams::ChamsRenderer::new(&shader,&layout,wgpu::TextureFormat::Rgba8Unorm,false);
        let mut uploads=upload::Uploads::new(&device);
        let draw=|renderer:&mut chams::ChamsRenderer,uploads:&mut upload::Uploads,improved| {
            let mut encoder=device.create_command_encoder(&Default::default());
            let (color,view,depth)=scene(&device,&mut encoder,[1,1]);
            renderer.prepare(&device,&queue,uploads,&mut encoder,[1,1],&depth,&group,improved);
            renderer.composite(&mut encoder,&view);
            read(&device,&queue,encoder,&color,uploads)[0]
        };
        let close=|actual:[u8;4],expected:[u8;4]| {
            for (a,e) in actual.into_iter().zip(expected) {
                assert!((i32::from(a)-i32::from(e)).abs()<=1,"{actual:?} vs {expected:?}");
            }
        };
        let normal=settings("Normal","","Repeat",256,"Nearest",[0.0;2]);
        let mut translucent=quad([-1.0,1.0],0.75,[255,0,0,128]);
        for vertex in &mut translucent {vertex.layer=1<<24;}
        capture(&normal,&translucent);
        close(draw(&mut renderer,&mut uploads,false),[64,0,127,255]);
        // Official 26.3 Java oracle: renderAllFeatures replays this classic
        // translucent phase ten times with the improved option, never OIT.
        capture(&normal,&translucent);
        close(draw(&mut renderer,&mut uploads,true),[255,0,0,255]);
        let mut item=quad([-1.0,1.0],0.75,[255,0,0,128]);
        for vertex in &mut item {vertex.layer=32;}
        let mut geometry=item.to_vec();geometry.extend(quad([-1.0,1.0],0.25,[0,255,0,255]));
        capture(&normal,&geometry);
        close(draw(&mut renderer,&mut uploads,false),[128,127,0,255]);
        // A fully opaque translucent-item texel still uses its pipeline's
        // no-depth-write state. Image mode reads the unchanged Chams depth.
        for vertex in &mut item {vertex.color=u32::from_le_bytes([255,0,0,255]);}
        let image=png("item-depth",[1,1],&[0,255,0,255]);
        capture(&settings("Image",&image,"Stretch",256,"Nearest",[0.0;2]),&item);
        close(draw(&mut renderer,&mut uploads,false),[0,255,0,255]);
        std::fs::remove_file(image).unwrap();
    }
    #[test]
    fn actual_gpu_float_frame_retains_highlights_alpha_and_image_encoding() {
        let (device,queue)=gpu();let (shader,layout,group)=entity_shader(&device,&queue);
        let mut renderer=chams::ChamsRenderer::new(&shader,&layout,wgpu::TextureFormat::Rgba16Float,false);
        let mut uploads=upload::Uploads::new(&device);
        let mut geometry=quad([-1.0,1.0],0.75,[255,0,0,255]);
        for vertex in &mut geometry {vertex.light=240|240<<8;}
        capture(&settings("Normal","","Repeat",256,"Nearest",[0.0;2]),&geometry);
        let mut encoder=device.create_command_encoder(&Default::default());
        let (color,view,depth)=scene_in(&device,&mut encoder,[1,1],wgpu::TextureFormat::Rgba16Float,wgpu::Color{r:0.0,g:0.0,b:1.0,a:0.375});
        renderer.prepare(&device,&queue,&mut uploads,&mut encoder,[1,1],&depth,&group,false);renderer.composite(&mut encoder,&view);
        let pixel=read_float(&device,&queue,encoder,&color,&mut uploads);
        assert!(pixel[0]>1.8 && pixel[0]<2.0,"HDR light clipped: {pixel:?}");
        assert_eq!([pixel[1],pixel[2],pixel[3]],[0.0,0.0,0.375]);
        let image=png("linear",[1,1],&[128,64,32,255]);
        let mut renderer=chams::ChamsRenderer::new(&shader,&layout,wgpu::TextureFormat::Rgba16Float,true);
        capture(&settings("Image",&image,"Stretch",256,"Nearest",[0.0;2]),&quad([-1.0,1.0],0.25,[255;4]));
        let mut encoder=device.create_command_encoder(&Default::default());
        let (color,view,depth)=scene_in(&device,&mut encoder,[1,1],wgpu::TextureFormat::Rgba16Float,wgpu::Color{r:0.0,g:0.0,b:1.0,a:0.375});
        renderer.prepare(&device,&queue,&mut uploads,&mut encoder,[1,1],&depth,&group,false);renderer.composite(&mut encoder,&view);
        let pixel=read_float(&device,&queue,encoder,&color,&mut uploads);
        for (actual,input) in pixel[..3].iter().zip([128.0f32,64.0,32.0]) {
            assert!((*actual-(input/255.0).powf(2.2)).abs()<0.0003,"linear image: {pixel:?}");
        }
        assert_eq!(pixel[3],0.375);std::fs::remove_file(image).unwrap();
    }
    fn read_float(device:&wgpu::Device,queue:&wgpu::Queue,mut encoder:wgpu::CommandEncoder,color:&wgpu::Texture,uploads:&mut upload::Uploads)->[f32;4] {
        let buffer=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:256,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
        encoder.copy_texture_to_buffer(color.as_image_copy(),wgpu::TexelCopyBufferInfo{buffer:&buffer,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(256),rows_per_image:Some(1)}},color.size());
        uploads.finish(&encoder);queue.submit([encoder.finish()]);buffer.slice(..).map_async(wgpu::MapMode::Read,|r|r.unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();let data=buffer.slice(..).get_mapped_range().unwrap();
        std::array::from_fn(|i|half::f16::from_le_bytes([data[i*2],data[i*2+1]]).to_f32())
    }
}
