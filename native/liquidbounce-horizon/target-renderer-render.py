"""Independent TargetRenderer GPU; no frozen renderer mutations."""
from pathlib import Path
import base64
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def textures():
 root=ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce'
 files=[root/'target_renderer/target.png',root/'target_renderer/target2.png',root/'particles/glow.png']
 return 'const PNGS:[&str;3]=[\n'+''.join('"'+base64.b64encode(p.read_bytes()).decode()+'",\n'for p in files)+'];\n'
def source():
 t=(HERE/'particles-render.rs').read_text(encoding='utf8').replace('Particles','TargetRenderer').replace('particles','target_renderer').replace('TargetRendererRenderer','TargetRenderer')
 a=t.index('#[derive(Clone, Debug, serde::Deserialize)]');b=t.index('thread_local!',a)
 t=t[:a]+'''#[derive(Clone,Debug,serde::Deserialize)]
pub struct Quad{pub corners:[[f64;3];4],pub colors:[[f32;4];4],pub uv:[[f32;2];4],pub shape:u32,pub inner:f32,pub inner_color:[f32;4],pub texture:String,pub depth:bool}
#[derive(Default,Debug,serde::Deserialize)]pub struct Frame{pub quads:Vec<Quad>}
'''+t[b:]
 a=t.index('pub struct Vertex {');b=t.index('include!("target_renderer-textures.rs");',a)
 t=t[:a]+'''pub struct Vertex{pub clip:[f32;4],pub color:[f32;4],pub uv:[f32;2],pub inner_color:[f32;4],pub params:[f32;4]}
pub fn vertices(q:&Quad,projection:Mat4,camera:DVec3)->Vec<Vertex>{let v:[Vertex;4]=std::array::from_fn(|i|Vertex{clip:(projection*(DVec3::from_array(q.corners[i])-camera).as_vec3().extend(1.)).to_array(),color:q.colors[i],uv:q.uv[i],inner_color:q.inner_color,params:[q.shape as f32,q.inner,u8::from(q.depth)as f32,0.]});[0,1,2,0,2,3].into_iter().map(|i|v[i]).collect()}
thread_local!{static IMAGE_ERROR:RefCell<Option<String>>=const{RefCell::new(None)};}
pub fn take_error()->Option<String>{IMAGE_ERROR.with_borrow_mut(Option::take)}
'''+t[b:]
 t=t.replace('textures: Vec<Option<(wgpu::Texture, wgpu::BindGroup)>>,','textures:std::collections::BTreeMap<String,(wgpu::Texture,wgpu::BindGroup)>,\n    selected_custom:Option<String>,\n    loading:Option<(String,std::sync::mpsc::Receiver<Result<(u32,u32,Vec<u8>),String>>)>,\n    custom_last:Option<String>,')
 t=t.replace('pending: Vec<(usize, wgpu::Buffer, u32)>','pending: Vec<(String, wgpu::Buffer, u32, bool)>')
 t=t.replace('0=>Float32x4,1=>Float32x4,2=>Float32x2','0=>Float32x4,1=>Float32x4,2=>Float32x2,3=>Float32x4,4=>Float32x4').replace('array_stride: 40','array_stride: 72')
 t=t.replace('textures: (0..PNGS.len()).map(|_| None).collect(),','textures:Default::default(),selected_custom:None,loading:None,custom_last:None,')
 a=t.index('        // Preserve particle insertion order;');b=t.index('    pub fn draw(',a)
 t=t[:a]+'''        if let Some(path)=frame.quads.iter().find(|q|q.shape==1&&!q.texture.starts_with("builtin:")).map(|q|&q.texture){
            if self.selected_custom.as_ref()!=Some(path){self.selected_custom=Some(path.clone());if std::path::Path::new(path).is_file(){let(tx,rx)=std::sync::mpsc::channel();self.loading=Some((path.clone(),rx));let load=path.clone();std::thread::spawn(move||{let decoded=std::fs::read(&load).map_err(|e|e.to_string()).and_then(|b|if b.starts_with(&[0xff,0xd8]){crate::hud::decode_image_bytes(&b)}else{crate::hud::decode_bytes(&b)}.map_err(|e|e.to_string()));let _=tx.send(decoded);});}}
        }
        let ready=self.loading.as_ref().and_then(|(path,rx)|rx.try_recv().ok().map(|v|(path.clone(),v)));
        if let Some((path,result))=ready{self.loading=None;match result{Ok((w,h,data))=>{self.upload(device,queue,path.clone(),w,h,&data);self.custom_last=Some(path);},Err(error)=>IMAGE_ERROR.with_borrow_mut(|v|*v=Some(format!("Failed to load texture from '{}' for TargetRenderer Image ({error})",std::path::Path::new(&path).file_name().unwrap_or_default().to_string_lossy())))}}
        for q in frame.quads{
            let mut path=if q.shape==1{q.texture.clone()}else{"builtin:White".into()};
            if path.starts_with("builtin:")&&!self.textures.contains_key(&path){let decoded=match path.as_str(){"builtin:White"=>Some((1,1,vec![255;4])),_=>{let index=match path.as_str(){"builtin:Marker1"=>0,"builtin:Marker2"=>1,"builtin:Glow"=>2,_=>continue};crate::skins::base64_decode(PNGS[index]).and_then(|b|crate::hud::decode_bytes(&b).ok())}};if let Some((w,h,data))=decoded{self.upload(device,queue,path.clone(),w,h,&data);}}
            if !path.starts_with("builtin:"){if let Some(last)=&self.custom_last{path=last.clone();}else{continue;}}else if !self.textures.contains_key(&path){continue;}
            let v=vertices(&q,projection,camera);let buffer=device.create_buffer_init(&wgpu::util::BufferInitDescriptor{label:Some("TargetRenderer mesh"),contents:bytemuck::cast_slice(&v),usage:wgpu::BufferUsages::VERTEX});self.pending.push((path,buffer,v.len()as u32,q.depth));
        }
    }
    fn upload(&mut self,device:&wgpu::Device,queue:&wgpu::Queue,path:String,width:u32,height:u32,data:&[u8]){
        let texture=device.create_texture(&wgpu::TextureDescriptor{label:Some("TargetRenderer original texture"),size:wgpu::Extent3d{width,height,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8Unorm,usage:wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_DST,view_formats:&[]});
        queue.write_texture(wgpu::TexelCopyTextureInfo{texture:&texture,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},data,wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(width*4),rows_per_image:Some(height)},wgpu::Extent3d{width,height,depth_or_array_layers:1});
        let view=texture.create_view(&Default::default());let group=device.create_bind_group(&wgpu::BindGroupDescriptor{label:Some("TargetRenderer texture"),layout:&self.layout,entries:&[wgpu::BindGroupEntry{binding:0,resource:wgpu::BindingResource::TextureView(&view)},wgpu::BindGroupEntry{binding:1,resource:wgpu::BindingResource::Sampler(&self.sampler)}]});if !path.starts_with("builtin:"){self.textures.retain(|key,_|key.starts_with("builtin:"));}self.textures.insert(path,(texture,group));
    }
'''+t[b:]
 t=t.replace('if let Some((_, group)) = &self.textures[*texture] {','if let Some((_, group)) = self.textures.get(texture) {')
 # Real depth-tested source pipelines write the actual scene depth as well.
 t=t.replace('pipeline: wgpu::RenderPipeline,','pipeline:[wgpu::RenderPipeline;2],').replace('    depth_layout: wgpu::BindGroupLayout,\n','')
 a=t.index('        let depth_layout =');b=t.index('        let pipeline_layout =',a);t=t[:a]+t[b:]
 t=t.replace('bind_group_layouts: &[Some(&layout), Some(&depth_layout)],','bind_group_layouts: &[Some(&layout)],')
 t=t.replace('let pipeline = device.create_render_pipeline(', 'let pipeline=std::array::from_fn(|depth|device.create_render_pipeline(')
 a=t.index('        let pipeline=std::array::from_fn');b=t.index('        let sampler =',a);body=t[a:b];body=body.replace('depth_stencil: None,','depth_stencil:Some(wgpu::DepthStencilState{format:wgpu::TextureFormat::Depth32Float,depth_write_enabled:Some(depth==1),depth_compare:Some(if depth==1{wgpu::CompareFunction::GreaterEqual}else{wgpu::CompareFunction::Always}),stencil:Default::default(),bias:Default::default()}),');body=body.rstrip();assert body.endswith(');');body=body[:-2]+'));\n';t=t[:a]+body+t[b:]
 t=t.replace('            depth_layout,\n','')
 a=t.index('        let depth = device.create_bind_group(');b=t.index('        let mut pass =',a);t=t[:a]+t[b:]
 a=t.index('        let mut pass =');b=t.index('        drop(pass);',a);body=t[a:b]
 body=body.replace('            ..Default::default()','            depth_stencil_attachment:Some(wgpu::RenderPassDepthStencilAttachment{view:depth,depth_ops:Some(wgpu::Operations{load:wgpu::LoadOp::Load,store:wgpu::StoreOp::Store}),stencil_ops:None}),\n            ..Default::default()',1)
 body=body.replace('        pass.set_pipeline(&self.pipeline);\n','').replace('        pass.set_bind_group(1, &depth, &[]);\n','').replace('for (texture, buffer, count) in &self.pending {','for (texture, buffer, count, depth) in &self.pending {\n            pass.set_pipeline(&self.pipeline[usize::from(*depth)]);')
 t=t[:a]+body+t[b:]
 return t
