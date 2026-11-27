// SPDX-License-Identifier: GPL-3.0-or-later
// Native fs_outline's alpha test with the original model atlas and exact projected vertices.
@group(0) @binding(1) var atlas:texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler:sampler;
struct Out{@builtin(position) clip:vec4<f32>,@location(0) uv:vec2<f32>,@location(1) color:vec4<f32>}
@vertex fn model(@location(0) clip:vec4<f32>,@location(1) uv:vec2<f32>,@location(2) color:vec4<f32>)->Out{var o:Out;o.clip=clip;o.uv=uv;o.color=color;return o;}
@fragment fn capture(i:Out)->@location(0) vec4<f32>{if textureSample(atlas,atlas_sampler,i.uv).a==0.0{discard;}return vec4(i.color.rgb,1.0);}
