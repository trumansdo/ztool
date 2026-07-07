@group(0) @binding(0)
var<uniform> vp_matrix: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> rotation_matrix: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) clip_position:vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vx_main(@location(0) position: vec3<f32>, @location(1) color: vec3<f32>) -> VertexOutput {
    var pos = vp_matrix * rotation_matrix * vec4<f32>(position, 1.0);
    return VertexOutput(pos, color);
}

@fragment
fn fg_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}