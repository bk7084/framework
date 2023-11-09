/// Camera data.
struct Globals {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

/// Vertex shader input.
struct VSInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) uv1: vec2<f32>,
    @location(4) tangent: vec4<f32>,
    @location(5) color: vec4<f32>,
}