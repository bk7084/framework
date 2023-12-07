struct Locals {
    model: mat4x4<f32>,
    model_view_it: mat4x4<f32>,
    material_index: vec4<u32>,
}

// Projection and view matrix of the light source.
@group(0) @binding(0) var light_matrix: mat4x4<f32>;

struct ShadowMapVSInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
}

@group(1) @binding(0) var<storage> instances : array<Locals>;

fn vs_main(vin: ShadowMapVSInput) -> @builtin(position) vec4<f32> {
    let locals = instances[vin.instance_index + pconsts.instance_base_index];
    return light_matrix * locals.model * vec4<f32>(vin.position, 1.0);
}