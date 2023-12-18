struct Locals {
    model: mat4x4<f32>,
}

struct PConsts {
    instance_base_index: u32,
    light_index: u32,
}

struct Light {
    /// Direction or position of light. The last component is 0 for directional
    /// light and 1 for point light.
    dir_or_pos: vec4<f32>,
    /// Color/intensity of light.
    color: vec3<f32>,
    /// Matrix transforming from world space to light space.
    world_to_light: mat4x4<f32>,
}

struct LightArray {
    len: u32,
    data: array<Light>,
}

@group(0) @binding(0) var<storage, read> instances: array<Locals>;
@group(1) @binding(0) var<storage, read> lights: LightArray;

var<push_constant> pconsts: PConsts;

struct ShadowMapVSInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
}

@vertex
fn vs_main(vin: ShadowMapVSInput) -> @builtin(position) vec4<f32> {
    let locals = instances[vin.instance_index + pconsts.instance_base_index];
    let light = lights.data[pconsts.light_index];
    return light.world_to_light * locals.model * vec4<f32>(vin.position, 1.0);
}
