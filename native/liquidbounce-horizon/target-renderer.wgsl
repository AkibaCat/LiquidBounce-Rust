// SPDX-License-Identifier: GPL-3.0-or-later
// Source-derived heart.fsh, gradient_circle.fsh and rounded_rect.fsh.
override FRAME_LINEAR:bool=false;
struct Vertex{@builtin(position)clip:vec4<f32>,@location(0)color:vec4<f32>,@location(1)uv:vec2<f32>,@location(2)inner_color:vec4<f32>,@location(3)@interpolate(flat)params:vec4<f32>}
@vertex fn mesh(@location(0)clip:vec4<f32>,@location(1)color:vec4<f32>,@location(2)uv:vec2<f32>,@location(3)inner_color:vec4<f32>,@location(4)params:vec4<f32>)->Vertex{return Vertex(clip,color,uv,inner_color,params);}
@group(0)@binding(0)var image:texture_2d<f32>;
@group(0)@binding(1)var image_sampler:sampler;
fn sd_heart(input:vec2<f32>)->f32{let p=vec2(abs(input.x),input.y);if p.x+p.y>1.{let q=p-vec2(0.25,0.75);return sqrt(dot(q,q))-sqrt(2.)/4.;}let a=p-vec2(0.,1.);let b=p-vec2(0.5*max(p.x+p.y,0.));return sqrt(min(dot(a,a),dot(b,b)))*sign(p.x-p.y);}
@fragment fn image_fragment(v:Vertex)->@location(0)vec4<f32>{
 var color=v.color;
 // Sample unconditionally: implicit texture derivatives remain uniform.
 let texel=textureSample(image,image_sampler,v.uv);
 let shape=u32(v.params.x+0.5);
 let dist=length(v.uv*2.-vec2(1.));let aa=max(fwidth(dist),0.0001);
 let heart=sd_heart((v.uv*2.-vec2(1.))*vec2(0.6035534,0.5517767)*1.02+vec2(0.,0.5517767));let heart_aa=max(fwidth(heart),0.0001)*1.5;
 if shape==1u{color*=texel;}
 if shape==2u{let inner=clamp(v.params.y,0.,1.);let outer_alpha=1.-smoothstep(-aa,0.,dist-1.);var inner_alpha=1.;if inner>0.{inner_alpha=1.-smoothstep(-aa,0.,inner-dist);}let t=clamp((dist-inner)/max(1.-inner,0.0001),0.,1.);color=mix(v.inner_color,v.color,t);color.a*=outer_alpha*inner_alpha;}
 if shape==3u{let d=dist-1.;let width=2.*aa;color.a*=(1.-smoothstep(-aa,0.,d))*smoothstep(-width-aa,-width+aa,d);}
 if shape==4u{color.a*=1.-smoothstep(-heart_aa,0.,heart);}
 if color.a<=0.001{discard;}
 if FRAME_LINEAR{color=vec4(pow(color.rgb,vec3(2.2)),color.a);}return color;
}
