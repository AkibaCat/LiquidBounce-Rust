// SPDX-License-Identifier: GPL-3.0-or-later
override FRAME_LINEAR:bool=false;
struct Vertex{@builtin(position)clip:vec4<f32>,@location(0)color:vec4<f32>,@location(1)uv:vec2<f32>}
@vertex fn mesh(@location(0)clip:vec4<f32>,@location(1)color:vec4<f32>,@location(2)uv:vec2<f32>)->Vertex{var v:Vertex;v.clip=clip;v.color=color;v.uv=uv;return v;}
@group(0)@binding(0)var image:texture_2d<f32>;
@group(0)@binding(1)var image_sampler:sampler;
@group(1)@binding(0)var world_depth:texture_depth_2d;
@fragment fn image_fragment(v:Vertex)->@location(0)vec4<f32>{
    var color=textureSample(image,image_sampler,v.uv)*v.color;
    let size=textureDimensions(world_depth);let p=clamp(vec2<i32>(v.clip.xy),vec2(0),vec2<i32>(size)-1);
    if v.clip.z<textureLoad(world_depth,p,0){discard;}
    if FRAME_LINEAR{color=vec4(pow(color.rgb,vec3(2.2)),color.a);}
    return color;
}
