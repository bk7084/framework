    // illum 0: Color on, ambient off
    // illum 1: Color on, ambient on
    // illum 2: Highlight on
    // illum 3: Reflection on and ray trace on
    // illum 4: Transparency: Glass on, Reflection: Ray trace on
    // illum 5: Reflection: Fresnel on and Ray trace on
    // illum 6: Transparency: Refraction on, Reflection: Fresnel off and Ray trace on
    // illum 7: Transparency: Refraction on, Reflection: Fresnel on and Ray trace on
    // illum 8: Reflection on and Ray trace off
    // illum 9: Transparency: Glass on, Reflection: Ray trace off
    // illum 10: Casts shadows onto invisible surfaces
    // Self defined:
    // - 11: kd as color, no lighting

/// Camera data.
struct Globals {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

struct PushConstants {
    /// Visible to the vertex shader.
    model: mat4x4<f32>,
    model_view_inv: mat4x4<f32>, // Inverse of the product of model and view matrix.
    /// Visible to the fragment shader.
    material_index: u32,
}

/// Data for a directional light.
struct DirectionalLight {
    /// Direction of light.
    direction: vec3<f32>,
    /// Color/intensity of light.
    color: vec3<f32>,
}

struct PointLight {
    /// Position of light.
    position: vec3<f32>,
    /// Color/intensity of light.
    color: vec3<f32>,
}

/// Data for multiple directional lights.
struct DirectionalLightArray {
     len: u32,
     data: array<DirectionalLight>,
}

struct PointLightArray {
     len: u32,
     data: array<PointLight>,
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
    @location(0) pos_eye_space: vec3<f32>,
    @location(1) normal_eye_space: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) view_mat_x: vec4<f32>, // View matrix.
    @location(4) view_mat_y: vec4<f32>,
    @location(5) view_mat_z: vec4<f32>,
    @location(6) view_mat_w: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var<storage> materials: array<Material>;

@group(2) @binding(0)
var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1)
var samplers: binding_array<sampler>;

@group(3) @binding(0)
var<storage> directional_lights: DirectionalLightArray;
@group(3) @binding(1)
var<storage> point_lights: PointLightArray;

var<push_constant> pconsts: PushConstants;

@vertex
fn vs_main(vin: VSInput) -> VSOutput {
    var out: VSOutput;
    let pos_eye_space = globals.view * pconsts.model * vec4<f32>(vin.position, 1.0);

    out.position = globals.proj * pos_eye_space;
    out.pos_eye_space = pos_eye_space.xyz / pos_eye_space.w;

    out.uv = vin.uv0;
    var transformed_normal = transpose(pconsts.model_view_inv) * vec4<f32>(vin.normal, 0.0);
    out.normal_eye_space = normalize(transformed_normal.xyz);
    out.view_mat_x = globals.view.x;
    out.view_mat_y = globals.view.y;
    out.view_mat_z = globals.view.z;
    out.view_mat_w = globals.view.w;
    return out;
}

const INVALID_INDEX: u32 = 0xffffffffu;

/// Blinn-Phong BRDF in camera space.
fn blinn_phong_brdf(wi: vec3<f32>, wo: vec3<f32>, n: vec3<f32>, kd: vec3<f32>, ks: vec3<f32>, ns: f32) -> vec3<f32> {
    if ((dot(n, wo) < 0.0) || (dot(n, wi) < 0.0)) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    var dot_n_wi = max(0.0, dot(n, wi));
    var diffuse = kd * dot_n_wi;

    var h = normalize(wi + wo);
    var dot_n_h = max(0.0, dot(n, h));
    var specular = ks * pow(dot_n_h, ns);
    return diffuse + specular;
}

fn blinn_phong_shading(view_mat: mat4x4<f32>, pos_eye_space: vec3<f32>, n: vec3<f32>, kd: vec3<f32>, ks: vec3<f32>, ns: f32) -> vec3<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);

    // View direction in camera space.
    let wo = normalize(-pos_eye_space);

    for (var i: u32 = 0u; i < directional_lights.len; i++) {
        var light = directional_lights.data[i];
        // Light direction in view space.
        var wi = (view_mat * vec4<f32>(normalize(light.direction), 0.0)).xyz;
        var coeff = blinn_phong_brdf(wi, wo, n, kd, ks, ns);
        color += coeff * light.color;
    }

    for (var i: u32 = 0u; i < point_lights.len; i = i + 1u) {
        var light = point_lights.data[i];
        // Light position in view space.
        var light_pos = (view_mat * vec4<f32>(light.position, 1.0)).xyz;
        var pos_to_light = light_pos - pos_eye_space;

        var dist = length(pos_to_light) * 0.6;
        var light_color = light.color * (1.0 / pow(dist, 2.0));

        var wi = normalize(pos_to_light);
        var coeff = blinn_phong_brdf(wi, wo, n, kd, ks, ns);
        color += coeff * light_color;
    }

    return color;
}

@fragment
fn fs_main(vout: VSOutput) -> @location(0) vec4<f32> {
    let ia = vec3<f32>(0.02, 0.02, 0.02);

    var materials_count: u32 = arrayLength(&materials);
    var default_material_index: u32 = materials_count - 1u;

    var material = materials[pconsts.material_index];

    var kd = material.kd.rgb;
    if (material.map_kd != INVALID_INDEX) {
        kd = textureSample(textures[material.map_kd], samplers[material.map_kd], vout.uv).rgb;
    }

    // Output kd as color.
    if (material.illum == 11u) {
        return vec4<f32>(kd, 1.0);
    }

    var color = materials[default_material_index].kd.rgb;

    var ks = material.ks.rgb;
    if (material.map_ks != INVALID_INDEX) {
        ks = textureSample(textures[material.map_ks], samplers[material.map_ks], vout.uv).rgb;
    }

    var ns = material.ns;
    if (material.map_ns != INVALID_INDEX) {
        ns = textureSample(textures[material.map_ns], samplers[material.map_ns], vout.uv).r;
    }

    let view_mat = mat4x4<f32>(vout.view_mat_x, vout.view_mat_y, vout.view_mat_z, vout.view_mat_w);

    color = blinn_phong_shading(view_mat, vout.pos_eye_space, vout.normal_eye_space, kd, ks, ns);

    // Ambient on.
    if (material.illum != 0u) {
        var ka = material.ka.rgb;
        if (material.map_ka != INVALID_INDEX) {
            ka = textureSample(textures[material.map_ka], samplers[material.map_ka], vout.uv).rgb;
        }
        color += ka * ia;
    }

    // Output UV as color.
    // color = vec3<f32>(vout.uv, 0.0);

    // Output normal as color.
    // color = vout.normal;

    // Output kd as color.
    // color = kd;
    // color = directional_lights.data[0].direction;
    return vec4<f32>(color, 1.0);
}
