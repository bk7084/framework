struct Globals {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

struct PushConstants {
    model: mat4x4<f32>,
}

struct VSInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
}

struct VSOutput {
    // Clip space position when the struct is used as vertex stae output.
    // Screen space position when the struct is used as fragment stage input.
    @builtin(position) position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;
var<push_constant> pconsts: PushConstants;

@vertex
fn vs_main(vin: VSInput) -> VSOutput {
    var out: VSOutput;
    out.position = globals.proj * globals.view * pconsts.model * vec4<f32>(vin.position, 1.0);
    return out;
}

@fragment
fn fs_main(vout: VSOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.99110, 0.93869, 0.78354, 1.0);
}