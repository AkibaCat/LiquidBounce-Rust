// SPDX-License-Identifier: GPL-3.0-or-later
// Native binding adaptation of original effect/mask.frag and effect/combine.frag.
override FRAME_LINEAR: bool = false;
struct ScreenOut { @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32> }
fn screen_vertex(i:u32)->ScreenOut {
    let p=array<vec2<f32>,3>(vec2(-1.,-1.),vec2(3.,-1.),vec2(-1.,3.))[i];
    var o:ScreenOut; o.position=vec4(p.x,-p.y,0.,1.); o.uv=p*.5+.5; return o;
}
@vertex fn screen(@builtin(vertex_index) i:u32)->ScreenOut {return screen_vertex(i);}
@vertex fn composite_screen(@builtin(vertex_index) i:u32)->ScreenOut {
    var o=screen_vertex(i); o.position.y = -o.position.y; return o;
}
@group(0) @binding(0) var in0:texture_2d<f32>;
@group(0) @binding(1) var in1:texture_2d<f32>;
@group(0) @binding(3) var in2:texture_2d<f32>;
struct Data {tag:u32,untagged:u32,count:u32,pad:u32}
@group(0) @binding(2) var<uniform> data:Data;
@fragment fn mask(i:ScreenOut)->@location(0) vec4<f32> {
    let p=vec2<i32>(i.position.xy); let texel=textureLoad(in0,p,0);
    let tag=u32(texel.b*255.+.5)&7u;
    let kept=texel.a>0. && (data.tag==0u || tag==data.tag || (data.untagged>0u && tag==0u));
    return select(vec4(0.),texel,kept);
}
fn over(top:vec4<f32>,below:vec4<f32>)->vec4<f32> {
    let alpha=top.a+below.a*(1.-top.a);
    if alpha<=0. {return vec4(0.);}
    return vec4((top.rgb*top.a+below.rgb*below.a*(1.-top.a))/alpha,alpha);
}
@fragment fn combine(i:ScreenOut)->@location(0) vec4<f32> {
    let size=textureDimensions(in0); let p=clamp(vec2<i32>(i.uv*vec2<f32>(size)),vec2(0),vec2<i32>(size)-1);
    var color=vec4(0.);
    if data.count>2u {color=textureLoad(in2,p,0);}
    if data.count>1u {color=over(textureLoad(in1,p,0),color);}
    color=over(textureLoad(in0,p,0),color);
    if FRAME_LINEAR {color=vec4(pow(max(color.rgb,vec3(0.)),vec3(2.2)),color.a);}
    return color;
}
