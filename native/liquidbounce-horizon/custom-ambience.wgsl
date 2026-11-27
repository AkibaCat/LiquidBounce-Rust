// SPDX-License-Identifier: GPL-3.0-or-later
override FRAME_LINEAR:bool=false;
struct Out{@builtin(position)position:vec4<f32>,@location(0)color:vec4<f32>,@location(1)uv:vec2<f32>}
@vertex fn vertex(@location(0)clip:vec4<f32>,@location(1)color:vec4<f32>,@location(2)uv:vec2<f32>)->Out{return Out(clip,color,uv);}
@group(0)@binding(0)var image:texture_2d<f32>;
@group(0)@binding(1)var image_sampler:sampler;
@fragment fn fragment(v:Out)->@location(0)vec4<f32>{var color=textureSample(image,image_sampler,v.uv)*v.color;if color.a==0.0{discard;}if FRAME_LINEAR{color=vec4(pow(max(color.rgb,vec3(0.)),vec3(2.2)),color.a);}return color;}
