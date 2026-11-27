// SPDX-License-Identifier: GPL-3.0-or-later
override FRAME_LINEAR: bool = false;
struct Vertex { @builtin(position) clip: vec4<f32>, @location(0) color: vec4<f32>, @location(1) uv: vec2<f32> }
@vertex fn mesh(@location(0) clip:vec4<f32>,@location(1) color:vec4<f32>,@location(2) uv:vec2<f32>)->Vertex{return Vertex(clip,color,uv);}
@group(0) @binding(0) var image:texture_2d<f32>;
@group(0) @binding(1) var image_sampler:sampler;
@fragment fn image_fragment(v:Vertex)->@location(0) vec4<f32>{
    var color=textureSample(image,image_sampler,v.uv)*v.color;
    if FRAME_LINEAR {color=vec4<f32>(pow(color.rgb,vec3<f32>(2.2)),color.a);}
    return color;
}
