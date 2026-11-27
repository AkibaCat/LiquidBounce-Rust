"""Independent exact gradient circle/image capture plus frozen original ShaderLayer passes."""
from pathlib import Path
import base64
HERE=Path(__file__).resolve().parent
NATIVE=HERE if (HERE/'breadcrumbs-render.rs').is_file() else HERE.parents[1]
ROOT=NATIVE.parents[1]
def textures():
 root=ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/jump_effect'
 return 'const PNGS:[&str;2]=[\n'+''.join('"'+base64.b64encode((root/n).read_bytes()).decode()+'",\n'for n in ['liquidbounce.png','liquidbounce_with_logo.png'])+'];\n'
def source():
 t=(NATIVE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('ModuleBreadcrumbs','ModuleJumpEffect').replace('BreadcrumbRenderer','JumpEffectRenderer').replace('Breadcrumbs','JumpEffect').replace('breadcrumbs','jump-effect')
 a=t.index('#[derive(Clone, Copy, Debug, serde::Deserialize)]');b=t.index('const FORMAT:',a)
 t=t[:a]+'''#[derive(Clone,Debug,serde::Deserialize)]
pub struct Quad{pub corners:[[f64;3];4],pub colors:[[f32;4];4],pub uv:[[f32;2];4],pub inner_color:[f32;4],pub inner:f32,pub shape:u32}
#[derive(Clone,Debug,serde::Deserialize)]pub struct Effect{pub style:usize,pub params:[f32;8],pub accent:[f32;4],pub range:f32,pub opacity:f32}
#[derive(Default,Debug,serde::Deserialize)]pub struct Frame{pub quads:Vec<Quad>,pub effect:Option<Effect>,pub depth:bool,pub texture:String}
thread_local!{static PENDING:RefCell<Option<Frame>>=const{RefCell::new(None)};}
pub fn begin_frame(json:&str){PENDING.with_borrow_mut(|f|*f=serde_json::from_str(json).ok());}
#[repr(C)]#[derive(Clone,Copy,Debug,bytemuck::Pod,bytemuck::Zeroable)]
pub struct Vertex{pub clip:[f32;4],pub color:[f32;4],pub uv:[f32;2],pub inner_color:[f32;4],pub params:[f32;4]}
pub fn vertices(frame:&Frame,projection:Mat4,camera:DVec3,_size:[u32;2])->Vec<Vertex>{
    let mut out:Vec<Vertex>=frame.quads.iter().flat_map(|q|{let v:[Vertex;4]=std::array::from_fn(|i|Vertex{clip:(projection*(DVec3::from_array(q.corners[i])-camera).as_vec3().extend(1.)).to_array(),color:q.colors[i],uv:q.uv[i],inner_color:q.inner_color,params:[q.shape as f32,q.inner,0.,0.]});[0,1,2,0,2,3].map(|i|v[i])}).collect();
    if frame.effect.is_some()&&!frame.depth{for v in &mut out{v.clip[1]=-v.clip[1];}}
    out
}
'''+t[b:]
 t=t.replace('    capture: wgpu::RenderPipeline,','    capture:[wgpu::RenderPipeline;2],').replace('    plain: wgpu::RenderPipeline,','    plain:[wgpu::RenderPipeline;2],\n    images:Images,')
 t=t.replace('        let empty = device.create_pipeline_layout', '        let images=Images::new(device);\n        let empty = device.create_pipeline_layout')
 t=t.replace('            bind_group_layouts: &[],','            bind_group_layouts: &[Some(&images.layout)],')
 t=t.replace('let attributes = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4];','let attributes=wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x2,3=>Float32x4,4=>Float32x4];')
 t=t.replace('                    blend: bool| {','                    blend: bool,\n                    depth:bool| {')
 t=t.replace('array_stride: 32','array_stride: 72')
 t=t.replace('                depth_stencil: None,','                depth_stencil:mesh.then_some(wgpu::DepthStencilState{format:wgpu::TextureFormat::Depth32Float,depth_write_enabled:Some(depth),depth_compare:Some(if depth{wgpu::CompareFunction::GreaterEqual}else{wgpu::CompareFunction::Always}),stencil:Default::default(),bias:Default::default()}),')
 t=t.replace('make("screen", &fragment, "main", FORMAT, false, false)','make("screen", &fragment, "main", FORMAT, false, false,false)')
 t=t.replace('let capture = make("mesh", &shader, "capture", FORMAT, true, true);','let capture=std::array::from_fn(|depth|make("mesh",&shader,"capture",FORMAT,true,true,depth==1));')
 t=t.replace('let plain = make("mesh", &shader, "plain", format, true, true);','let plain=std::array::from_fn(|depth|make("mesh",&shader,"plain",format,true,true,depth==1));')
 t=t.replace('            false,\n            true,\n        );','            false,\n            true,\n            false,\n        );',1)
 t=t.replace('            capture,\n            plain,','            capture,\n            plain,\n            images,')
 t=t.replace('        let frame = PENDING.with_borrow_mut(Option::take);\n        let Some(frame)', '        let frame = PENDING.with_borrow_mut(Option::take);\n        let Some(mut frame)')
 t=t.replace('        projection: Mat4,\n        camera: DVec3,\n        size:', '        queue:&wgpu::Queue,\n        projection: Mat4,\n        camera: DVec3,\n        size:',1)
 t=t.replace('        let mesh = vertices(&frame, projection, camera, size);','        let Some(texture)=self.images.prepare(device,queue,&frame.texture)else{return;};\n        frame.texture=texture;\n        let mesh = vertices(&frame, projection, camera, size);')
 t=t.replace('        target: &wgpu::TextureView,\n        game_time:', '        target: &wgpu::TextureView,\n        scene_depth:&wgpu::TextureView,\n        game_time:',1)
 t=t.replace('        let pass = |encoder:', '        let depth=usize::from(frame.depth);\n        let Some((_,image_group))=self.images.textures.get(&frame.texture)else{return;};\n        let pass = |encoder:')
 t=t.replace('                ..Default::default()\n            });\n            p.set_pipeline(pipeline);','                depth_stencil_attachment:mesh.then_some(wgpu::RenderPassDepthStencilAttachment{view:scene_depth,depth_ops:Some(wgpu::Operations{load:wgpu::LoadOp::Load,store:wgpu::StoreOp::Store}),stencil_ops:None}),\n                ..Default::default()\n            });\n            p.set_pipeline(pipeline);',1)
 t=t.replace('pass(encoder, target, &self.plain, None, true, false);','pass(encoder,target,&self.plain[depth],Some(image_group),true,false);')
 t=t.replace('pass(encoder, &targets.views[0], &self.capture, None, true, true);','pass(encoder,&targets.views[0],&self.capture[depth],Some(image_group),true,true);')
 # A covered Shader capture must test the normally projected mesh against scene depth.
 # Flip its RGBA8 image afterward; the original ShaderLayer continues to receive GL-oriented input.
 t=t.replace('views: [wgpu::TextureView; 4]','views: [wgpu::TextureView; 5]')
 t=t.replace('    composite: wgpu::RenderPipeline,','    composite: wgpu::RenderPipeline,\n    flip_capture:wgpu::RenderPipeline,')
 t=t.replace('        Self {\n            layout,','        let flip_capture=make("composite_screen",&shader,"flip_capture",FORMAT,false,false,false);\n        Self {\n            layout,')
 t=t.replace('            composite,','            composite,\n            flip_capture,')
 t=t.replace('        pass(encoder,&targets.views[0],&self.capture[depth],Some(image_group),true,true);','''        if frame.depth {
            pass(encoder,&targets.views[4],&self.capture[1],Some(image_group),true,true);
            let flipped=group(&targets.views[4]);
            pass(encoder,&targets.views[0],&self.flip_capture,Some(&flipped),false,true);
        }else{pass(encoder,&targets.views[0],&self.capture[0],Some(image_group),true,true);}''')
 t+='\n'+(HERE/'jump-effect-images.rs').read_text(encoding='utf8')
 return t
def shader():
 t=(NATIVE/'breadcrumbs.wgsl').read_text(encoding='utf8');a=t.index('struct MeshOut');b=t.index('struct ScreenOut');
 t=t[:a]+'''struct MeshOut{@builtin(position)position:vec4<f32>,@location(0)color:vec4<f32>,@location(1)uv:vec2<f32>,@location(2)inner_color:vec4<f32>,@location(3)@interpolate(flat)params:vec4<f32>}
@vertex fn mesh(@location(0)clip:vec4<f32>,@location(1)color:vec4<f32>,@location(2)uv:vec2<f32>,@location(3)inner_color:vec4<f32>,@location(4)params:vec4<f32>)->MeshOut{return MeshOut(clip,color,uv,inner_color,params);}
@group(0)@binding(0)var image:texture_2d<f32>;
@group(0)@binding(1)var image_sampler:sampler;
fn source_color(v:MeshOut)->vec4<f32>{
    var color=v.color;let texel=textureSample(image,image_sampler,v.uv);let dist=length(v.uv*2.-vec2(1.));let aa=max(fwidth(dist),0.0001);
    if u32(v.params.x+0.5)==1u{color*=texel;}else{
        let inner=clamp(v.params.y,0.,1.);let outer_alpha=1.-smoothstep(-aa,0.,dist-1.);var inner_alpha=1.;if inner>0.{inner_alpha=1.-smoothstep(-aa,0.,inner-dist);}
        let t=clamp((dist-inner)/max(1.-inner,0.0001),0.,1.);color=mix(v.inner_color,v.color,t);color.a*=outer_alpha*inner_alpha;
    }return color;
}
@fragment fn capture(v:MeshOut)->@location(0)vec4<f32>{let color=source_color(v);if color.a<=0.001{discard;}return color;}
@fragment fn plain(v:MeshOut)->@location(0)vec4<f32>{var color=source_color(v);if color.a<=0.001{discard;}if FRAME_LINEAR{color=vec4(pow(max(color.rgb,vec3(0.)),vec3(2.2)),color.a);}return color;}
'''+t[b:]
 t+='\n@fragment fn flip_capture(i:ScreenOut)->@location(0)vec4<f32>{let size=textureDimensions(styled);let p=clamp(vec2<i32>(i.uv*vec2<f32>(size)),vec2(0),vec2<i32>(size)-1);return textureLoad(styled,p,0); }\n'
 return t
if __name__=='__main__':
 (HERE/'jump-effect-render.rs').write_text(source(),encoding='utf8')
 (HERE/'jump-effect.wgsl').write_text(shader(),encoding='utf8')
 (HERE/'jump-effect-textures.rs').write_text(textures(),encoding='utf8')
 print('Generated independent JumpEffect renderer, WGSL, original texture bytes')
