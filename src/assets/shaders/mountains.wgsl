// Draw mountains with gradients in SRGB
#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(100)
var<uniform> color_start: vec4<f32>;

@group(2) @binding(101)
var<uniform> color_end: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Gradient along the Y axis
    let t = 1.0 - clamp(in.uv.y * 3., 0.0, 1.0);
    let color = mix(color_start, color_end, t);
    return vec4<f32>(srgb_to_linear(color.rgb), color.a);
}

// Convert sRGB to linear color space because we interpolate in sRGB space.
fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    let a = 0.055;
    let srgb_low = srgb / 12.92;
    let srgb_high = pow((srgb + a) / (1.0 + a), vec3<f32>(2.4, 2.4, 2.4));
    let linear = mix(srgb_low, srgb_high, step(vec3<f32>(0.04045, 0.04045, 0.04045), srgb));
    return linear;
}
