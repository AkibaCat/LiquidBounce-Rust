"""Independent annulus/ordered wireframe meshes and depth-correct frozen ShaderLayer."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'breadcrumbs-render.rs').is_file()else HERE.parents[1]

def source():
 t=(NATIVE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('ModuleBreadcrumbs','ModuleTotemEffect').replace('BreadcrumbRenderer','TotemEffectRenderer').replace('Breadcrumbs','TotemEffect').replace('breadcrumbs','totem-effect')
 a=t.index('#[derive(Clone, Copy, Debug, serde::Deserialize)]');b=t.index('const FORMAT:',a)
 t=t[:a]+(HERE/'totem-effect-vertices.rs').read_text(encoding='utf8')+'\n'+t[b:]
 t=t.replace('views: [wgpu::TextureView; 4]','views: [wgpu::TextureView; 5]')
 t=t.replace('    capture: wgpu::RenderPipeline,','    capture:[wgpu::RenderPipeline;2],').replace('    plain: wgpu::RenderPipeline,','    plain:[wgpu::RenderPipeline;2],')
 t=t.replace('    composite: wgpu::RenderPipeline,','    composite: wgpu::RenderPipeline,\n    flip_capture:wgpu::RenderPipeline,')
 t=t.replace('let attributes = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4];','let attributes=wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x2,3=>Float32x4,4=>Float32x4];')
 t=t.replace('                    blend: bool| {','                    blend: bool,depth:bool| {').replace('array_stride: 32','array_stride: 72')
 t=t.replace('                depth_stencil: None,','                depth_stencil:mesh.then_some(wgpu::DepthStencilState{format:wgpu::TextureFormat::Depth32Float,depth_write_enabled:Some(depth),depth_compare:Some(if depth{wgpu::CompareFunction::GreaterEqual}else{wgpu::CompareFunction::Always}),stencil:Default::default(),bias:Default::default()}),')
 t=t.replace('make("screen", &fragment, "main", FORMAT, false, false)','make("screen",&fragment,"main",FORMAT,false,false,false)')
 t=t.replace('let capture = make("mesh", &shader, "capture", FORMAT, true, true);','let capture=std::array::from_fn(|depth|make("mesh",&shader,"capture",FORMAT,true,true,depth==1));')
 t=t.replace('let plain = make("mesh", &shader, "plain", format, true, true);','let plain=std::array::from_fn(|depth|make("mesh",&shader,"plain",format,true,true,depth==1));')
 t=t.replace('            false,\n            true,\n        );','            false,\n            true,\n            false,\n        );',1)
 t=t.replace('        Self {\n            layout,','        let flip_capture=make("composite_screen",&shader,"flip_capture",FORMAT,false,false,false);\n        Self {\n            layout,').replace('            composite,','            composite,\n            flip_capture,')
 t=t.replace('        target: &wgpu::TextureView,\n        game_time:', '        target: &wgpu::TextureView,\n        scene_depth:&wgpu::TextureView,\n        game_time:',1)
 t=t.replace('        let pass = |encoder:','        let depth=usize::from(frame.depth);\n        let pass = |encoder:')
 t=t.replace('                ..Default::default()\n            });\n            p.set_pipeline(pipeline);','                depth_stencil_attachment:mesh.then_some(wgpu::RenderPassDepthStencilAttachment{view:scene_depth,depth_ops:Some(wgpu::Operations{load:wgpu::LoadOp::Load,store:wgpu::StoreOp::Store}),stencil_ops:None}),\n                ..Default::default()\n            });\n            p.set_pipeline(pipeline);',1)
 t=t.replace('pass(encoder, target, &self.plain, None, true, false);','pass(encoder,target,&self.plain[depth],None,true,false);')
 t=t.replace('pass(encoder, &targets.views[0], &self.capture, None, true, true);','''if frame.depth{
            pass(encoder,&targets.views[4],&self.capture[1],None,true,true);
            let flipped=group(&targets.views[4]);
            pass(encoder,&targets.views[0],&self.flip_capture,Some(&flipped),false,true);
        }else{pass(encoder,&targets.views[0],&self.capture[0],None,true,true);}''')
 return t

def shader():
 t=(NATIVE/'breadcrumbs.wgsl').read_text(encoding='utf8');a=t.index('struct MeshOut');b=t.index('struct ScreenOut')
 t=t[:a]+'''struct MeshOut{@builtin(position)position:vec4<f32>,@location(0)color:vec4<f32>,@location(1)uv:vec2<f32>,@location(2)inner_color:vec4<f32>,@location(3)@interpolate(flat)params:vec4<f32>}
@vertex fn mesh(@location(0)clip:vec4<f32>,@location(1)color:vec4<f32>,@location(2)uv:vec2<f32>,@location(3)inner_color:vec4<f32>,@location(4)params:vec4<f32>)->MeshOut{return MeshOut(clip,color,uv,inner_color,params);}
fn source_color(v:MeshOut)->vec4<f32>{
    let dist=length(v.uv*2.-vec2(1.));let aa=max(fwidth(dist),0.0001);var color=v.color;
    if u32(v.params.x+0.5)==2u{let inner=clamp(v.params.y,0.,1.);let outer_alpha=1.-smoothstep(-aa,0.,dist-1.);var inner_alpha=1.;if inner>0.{inner_alpha=1.-smoothstep(-aa,0.,inner-dist);}let t=clamp((dist-inner)/max(1.-inner,0.0001),0.,1.);color=mix(v.inner_color,v.color,t);color.a*=outer_alpha*inner_alpha;}return color;
}
@fragment fn capture(v:MeshOut)->@location(0)vec4<f32>{let color=source_color(v);if color.a<=0.001{discard;}return color;}
@fragment fn plain(v:MeshOut)->@location(0)vec4<f32>{var color=source_color(v);if color.a<=0.001{discard;}if FRAME_LINEAR{color=vec4(pow(max(color.rgb,vec3(0.)),vec3(2.2)),color.a);}return color;}
'''+t[b:]
 t+='\n@fragment fn flip_capture(i:ScreenOut)->@location(0)vec4<f32>{let size=textureDimensions(styled);let p=clamp(vec2<i32>(i.uv*vec2<f32>(size)),vec2(0),vec2<i32>(size)-1);return textureLoad(styled,p,0);}\n'
 return t
if __name__=='__main__':
 (HERE/'totem-effect-render.rs').write_text(source(),encoding='utf8');(HERE/'totem-effect.wgsl').write_text(shader(),encoding='utf8')
 print('Generated independent TotemEffect renderer and WGSL')
