struct Geometry { bounds: vec4<f32> }
@group(0) @binding(0) var browser: texture_2d<f32>;
@group(0) @binding(1) var browser_sampler: sampler;
@group(0) @binding(2) var<uniform> geometry: Geometry;
struct Vertex { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> Vertex {
    let corners = array<vec2<f32>, 6>(vec2(0., 0.), vec2(1., 0.), vec2(0., 1.),
                                     vec2(0., 1.), vec2(1., 0.), vec2(1., 1.));
    let uv = corners[index];
    let at = geometry.bounds.xy + uv * geometry.bounds.zw;
    var out: Vertex;
    out.position = vec4(at.x * 2. - 1., 1. - at.y * 2., 0., 1.);
    out.uv = uv;
    return out;
}
@fragment fn fs_plain(in: Vertex) -> @location(0) vec4<f32> {
    return textureSample(browser, browser_sampler, in.uv);
}
struct Covered { @location(0) color: vec4<f32>, @location(1) coverage: vec4<f32> }
@fragment fn fs_coverage(in: Vertex) -> Covered {
    let color = textureSample(browser, browser_sampler, in.uv);
    var out: Covered;
    out.color = color;
    out.coverage = vec4(color.a);
    return out;
}
