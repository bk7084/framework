/// Camera data.
struct Globals {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

struct PushConstants {
    /// Visible to the vertex shader.
    model: mat4x4<f32>,
    /// Visible to the fragment shader.
    material_index: u32,
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
    ka: vec4<f32>,
    kd: vec4<f32>,
    ks: vec4<f32>,
    ns: f32,
    ni: f32,
    d: f32,
    illum: u32,
    map_ka: u32, // Texture index. 0xFFFFFFFF if no texture.
    map_kd: u32,
    map_ks: u32,
    map_ns: u32,
    map_d: u32,
    map_bump: u32,
    map_disp: u32,
    map_decal: u32,
    map_norm: u32,
    // padding: vec3<u32>,
}

/// Vertex shader input.
struct VSInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv0: vec2<f32>,
//    @location(3) uv1: vec2<f32>,
//    @location(4) tangent: vec4<f32>,
//    @location(5) color: vec4<f32>,
}

struct VSOutput {
    // Clip space position when the struct is used as vertex stae output.
    // Screen space position when the struct is used as fragment stage input.
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var<storage> materials: array<Material>;

@group(2) @binding(0)
var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1)
var samplers: binding_array<sampler>;

var<push_constant> pconsts: PushConstants;

@vertex
fn vs_main(vin: VSInput) -> VSOutput {
    var out: VSOutput;
    out.position = globals.proj * globals.view * pconsts.model * vec4<f32>(vin.position, 1.0);
    out.uv = vin.uv0;
    out.normal = vin.normal;
    return out;
}

@fragment
fn fs_main(vout: VSOutput) -> @location(0) vec4<f32> {
    var material = materials[pconsts.material_index];

    var color = material.kd.rgb;

    // Output UV as color.
    //color = vec4<f32>(vout.uv, 0.0, 1.0);

    // Output normal as color.
    // color = vout.normal;

    if (material.map_kd != 0xffffffffu) {
        color = textureSample(textures[material.map_kd], samplers[material.map_kd], vout.uv).rgb;
    }

//    if (material.map_ka != 0xffffffffu) {
//        color = textureSample(textures[material.map_ka], samplers[material.map_ka], vout.uv).rgb;
//    }

//    if (material.illum == 0u) {
//        color = vec3<f32>(1.0, 1.0, 1.0);
//    } else if (material.illum == 1u) {
//        color = vec3<f32>(1.0, 0.0, 1.0);
//    } else if (material.illum == 2u) {
//        color = vec3<f32>(0.0, 1.0, 1.0);
//    } else {
//        color = vec3<f32>(1.0, 1.0, 0.0);
//    }

    return vec4<f32>(color, 1.0);
}
