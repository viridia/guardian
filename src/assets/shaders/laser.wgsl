// Draw laser beam effect
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals

@group(2) @binding(100)
var<uniform> color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    let ya = 1.0 - abs(in.uv.y - 0.5) * 2.0;
    let xa = smoothstep(0., 0.02, 0.5 - abs(in.uv.x - 0.5));
    let xb = smoothstep(-0.9, 0.1,
        sin(in.uv.x * 20. + t * 6.) +
        sin(in.uv.x * 45. - t * 8.) +
        sin(in.uv.x * 93. + t * 10.) +
        sin(in.uv.x * 267. - t * 14.));
    return vec4<f32>(color.rgb, color.a * ya * xa * xb);
}
