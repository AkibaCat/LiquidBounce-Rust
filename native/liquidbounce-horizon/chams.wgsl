// SPDX-License-Identifier: GPL-3.0-or-later
// LiquidBounce ae0eb396 chams.frag, adapted to WGPU screen coordinates.
override FRAME_LINEAR: bool = false;
@group(0) @binding(0) var entity_color: texture_2d<f32>;
@group(0) @binding(1) var entity_depth: texture_depth_2d;
@group(0) @binding(2) var scene_depth: texture_depth_2d;
@group(0) @binding(3) var image: texture_2d<f32>;
@group(0) @binding(4) var nearest_sampler: sampler;
@group(0) @binding(5) var image_sampler: sampler;
struct ChamsData { scale:vec2<f32>, offset:vec2<f32>, mode:f32, width:f32, height:f32, spare:f32 }
@group(0) @binding(6) var<uniform> data:ChamsData;
@vertex fn vs_quad(@builtin(vertex_index) index:u32)->@builtin(position) vec4<f32> {
    let positions=array<vec2<f32>,3>(vec2(-1.0,-1.0),vec2(3.0,-1.0),vec2(-1.0,3.0));
    return vec4(positions[index],0.0,1.0);
}
@fragment fn fs_chams(@builtin(position) position:vec4<f32>)->@location(0) vec4<f32> {
    let pixel=vec2<i32>(position.xy);
    let color=textureLoad(entity_color,pixel,0);
    if color.a==0.0 {discard;}
    if data.mode==0.0 {return color;}
    let entity=textureLoad(entity_depth,pixel,0);
    let scene=textureLoad(scene_depth,pixel,0);
    if entity>scene+0.00001 {return color;}
    // GLSL gl_FragCoord has a lower-left origin. Image bytes retain their source row order.
    let fragment=vec2(position.x,data.height-position.y);
    var result=textureSampleLevel(image,image_sampler,(fragment-data.offset)/data.scale,0.0);
    if FRAME_LINEAR {result=vec4(pow(max(result.rgb,vec3(0.0)),vec3(2.2)),result.a);}
    return result;
}
