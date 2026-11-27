// SPDX-License-Identifier: GPL-3.0-or-later
// GL full-screen coordinates: source effects use the lower-left UV origin.
struct ScreenOut {@builtin(position) position:vec4<f32>,@location(0) uv:vec2<f32>}
@vertex fn screen(@builtin(vertex_index) i:u32)->ScreenOut {
    let p=array<vec2<f32>,3>(vec2(-1.0,-1.0),vec2(3.0,-1.0),vec2(-1.0,3.0))[i];
    var o:ScreenOut;o.position=vec4(p,0.0,1.0);o.uv=p*0.5+0.5;return o;
}
// Intermediate targets keep the same lower-left texture origin as the flipped
// actual hand capture. Native lightmap rows keep their existing row ordering.
@vertex fn screen_native(@builtin(vertex_index) i:u32)->ScreenOut {
    let p=array<vec2<f32>,3>(vec2(-1.0,-1.0),vec2(3.0,-1.0),vec2(-1.0,3.0))[i];
    var o:ScreenOut;o.position=vec4(p.x,-p.y,0.0,1.0);o.uv=p*0.5+0.5;return o;
}
@group(0) @binding(0) var captured_depth:texture_depth_2d;
// Copy exact reversed depth to sampled R32F. GLSL's sampler2D depthTex then
// retains its source nearest sampling, epsilon and off-screen clamp semantics.
@fragment fn copy_depth(i:ScreenOut)->@location(0) f32 {
    return textureLoad(captured_depth,vec2<i32>(i.position.xy),0);
}
