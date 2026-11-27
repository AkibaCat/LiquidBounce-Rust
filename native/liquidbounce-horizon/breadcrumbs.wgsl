// SPDX-License-Identifier: GPL-3.0-or-later
override FRAME_LINEAR: bool = false;
struct MeshOut {@builtin(position) position:vec4<f32>,@location(0) color:vec4<f32>}
@vertex fn mesh(@location(0) clip:vec4<f32>,@location(1) color:vec4<f32>)->MeshOut{var o:MeshOut;o.position=clip;o.color=color;return o;}
@fragment fn capture(i:MeshOut)->@location(0) vec4<f32>{return i.color;}
@fragment fn plain(i:MeshOut)->@location(0) vec4<f32>{if FRAME_LINEAR{return vec4(pow(max(i.color.rgb,vec3(0.0)),vec3(2.2)),i.color.a);}return i.color;}
struct ScreenOut {@builtin(position) position:vec4<f32>,@location(0) uv:vec2<f32>}
fn screen_vertex(i:u32)->ScreenOut{let p=array<vec2<f32>,3>(vec2(-1.0,-1.0),vec2(3.0,-1.0),vec2(-1.0,3.0))[i];var o:ScreenOut;o.position=vec4(p.x,-p.y,0.0,1.0);o.uv=p*0.5+0.5;return o;}
@vertex fn screen(@builtin(vertex_index) i:u32)->ScreenOut{return screen_vertex(i);}
@vertex fn composite_screen(@builtin(vertex_index) i:u32)->ScreenOut{var o=screen_vertex(i);o.position.y = -o.position.y;return o;}
@group(0) @binding(3) var styled:texture_2d<f32>;
@fragment fn composite(i:ScreenOut)->@location(0) vec4<f32>{let size=textureDimensions(styled);let p=clamp(vec2<i32>(i.uv*vec2<f32>(size)),vec2(0),vec2<i32>(size)-1);var c=textureLoad(styled,p,0);if FRAME_LINEAR{c=vec4(pow(max(c.rgb,vec3(0.0)),vec3(2.2)),c.a);}return c;}
