"""Independent copies of frozen ShaderLayer renderer, with pinned radial circle fill."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def source():
 t=(HERE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('BreadcrumbRenderer','BlinkRenderer').replace('ModuleBreadcrumbs','BlinkManager').replace('include_str!("breadcrumbs.wgsl")','include_str!("blink_esp.wgsl")')
 t=t.replace('pub struct Frame {','''pub struct Ring {pub position:[f64;3],pub radius:f32,pub color:[f32;4],#[serde(default)]pub outline:bool}
#[derive(Default, Debug, serde::Deserialize)]
pub struct Frame {''').replace('    pub effect: Option<Effect>,','    pub effect: Option<Effect>,\n    #[serde(default)] pub rings:Vec<Ring>,')
 t=t.replace('    pub color: [f32; 4],\n}\n/// The original','    pub color: [f32; 4],\n    pub radial:[f32;4],\n}\n/// The original')
 t=t.replace('        color: p.color,','        color: p.color,\n        radial:[0.;4],')
 # Radial coordinates interpolate perspectively over the same ground-plane quad as LiquidBounce.
 anchor='    if frame.effect.is_some() {\n        for v in &mut out {'
 assert t.count(anchor)==1
 ring='''    for ring in &frame.rings {
        let corners=[[-1.,-1.],[1.,-1.],[1.,1.],[-1.,1.]];
        for i in [0,1,2,0,2,3] {
            let uv=corners[i];
            let mut vertex=project(ColoredPoint{position:[ring.position[0]+f64::from(uv[0]*ring.radius),ring.position[1],ring.position[2]+f64::from(uv[1]*ring.radius)],color:ring.color});
            vertex.radial=[uv[0],uv[1],if ring.outline{2.}else{1.},0.];out.push(vertex);
        }
    }
'''
 t=t.replace(anchor,ring+anchor).replace('0=>Float32x4,1=>Float32x4','0=>Float32x4,1=>Float32x4,2=>Float32x4').replace('array_stride: 32,','array_stride: 48,')
 # Every allocation uses size_of, so the extra coordinates cannot truncate uploads.
 return t
def shader():
 t=(HERE/'breadcrumbs.wgsl').read_text(encoding='utf8')
 t=t.replace('@location(0) color:vec4<f32>}','@location(0) color:vec4<f32>,@location(1) radial:vec4<f32>}')
 t=t.replace('@location(1) color:vec4<f32>)->MeshOut','@location(1) color:vec4<f32>,@location(2) radial:vec4<f32>)->MeshOut').replace('o.color=color;return o;','o.color=color;o.radial=radial;return o;')
 old='@fragment fn plain(i:MeshOut)->@location(0) vec4<f32>{if FRAME_LINEAR{return vec4(pow(max(i.color.rgb,vec3(0.0)),vec3(2.2)),i.color.a);}return i.color;}'
 new='''@fragment fn plain(i:MeshOut)->@location(0) vec4<f32>{
    var c=i.color;
    let dist=length(i.radial.xy);let dd=fwidth(dist);let aa=max(dd,0.0001);
    // circle/rounded_rect.fsh with CornerRadius=(1,1), StrokeWidth=2 pixels.
    if i.radial.z>1.5 {let d=dist-1.0;let w=2.0*dd;let edge=1.0-smoothstep(-dd,0.0,d);let band=smoothstep(-w-dd,-w+dd,d);c.a*=edge*band;if c.a<0.001{discard;}}
    // gradient_circle.fsh: inner radius 0, transparent inner colour.
    else if i.radial.z>0.5 {let coverage=1.0-smoothstep(-aa,0.0,dist-1.0);if coverage<=0.001{discard;}c=c*clamp(dist,0.0,1.0);c.a*=coverage;if c.a<=0.001{discard;}}
    if FRAME_LINEAR{return vec4(pow(max(c.rgb,vec3(0.0)),vec3(2.2)),c.a);}return c;
}'''
 assert t.count(old)==1
 return t.replace(old,new)
