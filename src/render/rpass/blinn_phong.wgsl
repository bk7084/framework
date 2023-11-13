/// Camera data.
struct Globals {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

struct PushConstants {
    model: mat4x4<f32>,
}

/// Data for a directional light.
struct DirectionalLight {
    /// View/Projection of light.
    view_proj: mat4x4<f32>,
    /// Color/intensity of light.
    color: vec3<f32>,
    /// Direction of light.
    direction: vec3<f32>,
}

/// Data for multiple directional lights.
struct DirectionalLightArray {
     len: u32,
     data: array<DirectionalLight>,
}

struct Material {
    ka: vec3<f32>,
    // -- 16 bytes --
    kd: vec3<f32>,
    // -- 32 bytes --
    ks: vec3<f32>,
    // -- 48 bytes --
    ns: f32,
    ni: f32,
    d: f32,
    illum: u32,
//    // -- 64 bytes --
//    map_ka: u32,
//    map_kd: u32,
//    map_ks: u32,
//    map_ns: u32,
//    // -- 80 bytes --
    // map_d: u32,
//    map_bump: u32,
//    map_disp: u32,
//    map_decal: u32,
//    // -- 96 bytes --
//    map_norm: u32,
//    vertex_color: u32, // Whether to use vertex color directly.
//    padding: vec2<u32>,
}

/// Vertex shader input.
struct VSInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
//    @location(1) normal: vec3<f32>,
//    @location(2) uv0: vec2<f32>,
//    @location(3) uv1: vec2<f32>,
//    @location(4) tangent: vec4<f32>,
//    @location(5) color: vec4<f32>,
}

struct VSOutput {
    // Clip space position when the struct is used as vertex stae output.
    // Screen space position when the struct is used as fragment stage input.
    @builtin(position) position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;
@group(1) @binding(0)
var<uniform> material: Material;
// var<storage> materials: array<Material>;

@group(2) @binding(0)
var textures: binding_array<texture_2d<f32>>;

var<push_constant> pconsts: PushConstants;

@vertex
fn vs_main(vin: VSInput) -> VSOutput {
    var out: VSOutput;
    out.position = globals.proj * globals.view * pconsts.model * vec4<f32>(vin.position, 1.0);
    return out;
}

@fragment
fn fs_main(vout: VSOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(material.kd, 1.0);
}
