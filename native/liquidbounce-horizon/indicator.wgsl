// SPDX-License-Identifier: GPL-3.0-or-later
// Original shaders/gui/arrow.fsh SDF, LiquidBounce ae0eb396.
struct Out{@builtin(position)pos:vec4<f32>,@location(0)local:vec2<f32>,@location(1)size:vec2<f32>,@location(2)color:vec4<f32>,@location(3)notch:f32}
@vertex fn vertex(@location(0)pos:vec2<f32>,@location(1)local:vec2<f32>,@location(2)size:vec2<f32>,@location(3)color:vec4<f32>,@location(4)notch:f32)->Out{return Out(vec4(pos,0.,1.),local,size,color,notch);}
fn polygon_distance(p:vec2<f32>,a:vec2<f32>,b:vec2<f32>,c:vec2<f32>,d:vec2<f32>)->f32{
 let vertices=array<vec2<f32>,4>(a,b,c,d);var closest=dot(p-a,p-a);var inside=1.;
 for(var i=0u;i<4u;i++){
  let previous=(i+3u)&3u;let edge=vertices[previous]-vertices[i];let offset=p-vertices[i];
  let to_edge=offset-edge*clamp(dot(offset,edge)/dot(edge,edge),0.,1.);closest=min(closest,dot(to_edge,to_edge));
  let crosses=vec3<bool>((p.y >= vertices[i].y),(p.y < vertices[previous].y),(edge.x*offset.y > edge.y*offset.x));
  if all(crosses)||all(!crosses){inside=-inside;}
 }
 return inside*sqrt(closest);
}
@fragment fn fragment(v:Out)->@location(0)vec4<f32>{
 let half_width=v.size.x*0.5;let distance=polygon_distance(v.local,vec2(-half_width,0.),vec2(0.,v.size.y),vec2(half_width,0.),vec2(0.,v.notch));
 let aa=max(fwidth(distance),0.0001);var color=v.color;color.a*=1.-smoothstep(-aa,0.,distance);if color.a<=0.001{discard;}return color;
}
