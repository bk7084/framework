// illum 0: Color on, ambient off
// illum 1: Color on, ambient on
// illum 2: Highlight on
// illum 3: Reflection on and ray trace on
// illum 4: Transparency: Glass on, Reflection: Ray trace on
// illum 5: Reflection: Fresnel on and Ray trace on
// illum 6: Transparency: Refraction on, Reflection: Fresnel off and Ray trace
// on illum 7: Transparency: Refraction on, Reflection: Fresnel on and Ray trace
// on illum 8: Reflection on and Ray trace off illum 9: Transparency: Glass on,
// Reflection: Ray trace off illum 10: Casts shadows onto invisible surfaces
// Self defined:
// - 11: kd as color, no lighting
// - 12: ks as color, no lighting
// - 13: uv as color, no lighting
// - 14: normal in view space as color, no lighting

const PNT_LIGHT: f32 = 1.0;
const DIR_LIGHT: f32 = 0.0;
const INVALID_INDEX: u32 = 0xffffffffu;

/// Camera data.
struct Globals {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

struct Locals {
    model: mat4x4<f32>,
    model_view_it: mat4x4<f32>,
    material_index: vec4<u32>,
}

struct PConsts {
    instance_base_index: u32,
    material_index: u32,
    enable_shadows: u32,
    enable_lighting: u32,
}

struct Light {
   /// Direction or position of light. The last component is 0 for directional light and 1 for point light.
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
    @location(2) texcoord: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

struct VSOutput {
    // Clip space position when the struct is used as vertex stae output.
    // Screen space position when the struct is used as fragment stage input.
    @builtin(position) position: vec4<f32>,
    @location(0) pos_eye_space: vec3<f32>,
    @location(1) normal_eye_space: vec3<f32>,
    @location(2) tangent_eye_space: vec4<f32>,
    @location(3) texcoord: vec2<f32>,
    @location(4) material_index: u32,
    @location(5) view_mat_x: vec4<f32>, // View matrix.
    @location(6) view_mat_y: vec4<f32>,
    @location(7) view_mat_z: vec4<f32>,
    @location(8) view_mat_w: vec4<f32>,
    @location(9) pos_world: vec3<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var<storage, read> instances: array<Locals>;
@group(2) @binding(0) var<storage, read> materials: array<Material>;
@group(3) @binding(0) var<storage, read> lights: LightArray;

// #if !constant_sized_binding_array
@group(4) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(4) @binding(1) var<storage, read> texture_sampler_ids: array<u32>;
@group(4) @binding(2) var samplers: binding_array<sampler>;
@group(5) @binding(0) var shadow: binding_array<texture_depth_2d_array>;
@group(5) @binding(1) var shadow_sampler: sampler_comparison;
// #else
@group(4) @binding(0) var textures: binding_array<texture_2d<f32>, 64>;
@group(4) @binding(1) var<storage, read> texture_sampler_ids: array<u32>;
@group(4) @binding(2) var samplers: binding_array<sampler, 8>;
@group(5) @binding(0) var shadow: binding_array<texture_depth_2d_array, 1>;
@group(5) @binding(1) var shadow_sampler: sampler_comparison;
// #fi

/* Sampling shadow map as normal texture. */
// @group(5) @binding(0) var shadow: binding_array<texture_2d_array<f32>>;
// @group(5) @binding(1) var shadow_sampler: sampler;

var<push_constant> pconsts: PConsts;

@vertex
fn vs_main(vin: VSInput) -> VSOutput {
    let locals = instances[vin.instance_index + pconsts.instance_base_index];

    var out: VSOutput;
    let model_view = globals.view * locals.model;
    out.pos_world = (locals.model * vec4<f32>(vin.position, 1.0)).xyz;
    let pos_eye_space = model_view * vec4<f32>(vin.position, 1.0);

    if (locals.material_index.x != INVALID_INDEX) {
        out.material_index = locals.material_index.x;
    } else {
        out.material_index = pconsts.material_index;
    }

    let nrm_mat = mat3x3(locals.model_view_it.x.xyz, locals.model_view_it.y.xyz, locals.model_view_it.z.xyz);
    out.position = globals.proj * pos_eye_space;
    out.texcoord = vin.texcoord;
    out.pos_eye_space = pos_eye_space.xyz / pos_eye_space.w;
    out.normal_eye_space = normalize(nrm_mat * vin.normal);
    out.tangent_eye_space = vec4<f32>(normalize(nrm_mat * vin.tangent.xyz), vin.tangent.w);
    out.view_mat_x = globals.view.x;
    out.view_mat_y = globals.view.y;
    out.view_mat_z = globals.view.z;
    out.view_mat_w = globals.view.w;
    return out;
}

/// Blinn-Phong BRDF in camera space.
fn blinn_phong_brdf(wi: vec3<f32>, wo: vec3<f32>, n: vec3<f32>, kd: vec3<f32>, ks: vec3<f32>, ns: f32, illum: u32) -> vec3<f32> {
    if pconsts.enable_lighting == 0u {
        return vec3<f32>(0.0);
    }

    var dot_n_wi = max(0.0, dot(n, wi));
    var diffuse = kd * dot_n_wi;
    if (illum != 2u) {
        return diffuse;
    }
    var h = normalize(wi + wo);
    var dot_n_h = max(0.0, dot(n, h));
    var specular = vec3<f32>(0.0);
    if (dot_n_wi > 0.0) {
        specular = ks * pow(dot_n_h, ns);
    }
    return diffuse + specular;
}

fn blinn_phong_shading_eye_space(view_mat: mat4x4<f32>, pos_world: vec3<f32>, pos_eye_space: vec3<f32>, n: vec3<f32>, kd: vec3<f32>, ks: vec3<f32>, ns: f32, illum: u32) -> vec3<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);

    // View direction in camera space.
    let wo = normalize(-pos_eye_space);

    var dir_light_index = 0u;
    for (var i: u32 = 0u; i < lights.len; i++) {
        let light = lights.data[i];
        if (light.dir_or_pos.w == DIR_LIGHT) {
            // Light direction in view space.
            let wi = (view_mat * normalize(light.dir_or_pos)).xyz;
            let coeff = blinn_phong_brdf(wi, wo, n, kd, ks, ns, illum);
            let pos_light_space = light.world_to_light * vec4<f32>(pos_world, 1.0);
            var shadow = 1.0;
            if (pconsts.enable_shadows != 0u) {
                shadow = fetch_shadow(dir_light_index, pos_light_space);
            }
            color += shadow * coeff * light.color;
            dir_light_index += 1u;
        } else if (light.dir_or_pos.w == PNT_LIGHT) {
            // Light position in view space.
            var light_pos = view_mat * light.dir_or_pos;
            var pos_to_light = (light_pos / light_pos.w).xyz - pos_eye_space;
            var dist = length(pos_to_light);
            var light_color = light.color * (1.0 / (1.0 + 0.02 * dist * dist));
            var wi = normalize(pos_to_light);
            var coeff = blinn_phong_brdf(wi, wo, n, kd, ks, ns, illum);
            color += coeff * light_color;
        }
    }

    return color;
}

/// Unpack normal from normal map.
///
/// The normal map is assumed to be in tangent space. The normal is unpacked to [-1, 1].
fn unpack_normal_map(map: u32, texcoord: vec2<f32>) -> vec3<f32> {
    var m = textureSample(textures[map], samplers[texture_sampler_ids[map]], texcoord).xyz;
    m = m * 2.0 - vec3<f32>(1.0);
    return normalize(m);
}

/// Constructs a TBN matrix transforming from tangent space to other spaces.
fn tbn_matrix(tangent: vec4<f32>, normal: vec3<f32>) -> mat3x3<f32> {
    let n = normalize(normal);
    var t = normalize(tangent.xyz);
    t = normalize(t - n * dot(t, n));
    let b = normalize(cross(n, t) * tangent.w);
    return mat3x3<f32>(t, b, n);
}

fn fetch_shadow(light_idx: u32, pos_light_space: vec4<f32>) -> f32 {
    if (pos_light_space.w <= 0.0) {
        // The point is outside the light frustum.
        return 1.0;
    }
    // Compensate for the Y-flip difference between the NDC space and the texture space.
    let flip_correction = vec2<f32>(0.5, -0.5);
    let proj_correction = 1.0 / pos_light_space.w;
    // Compute texture coordinates for shadow map lookup. Transform from [-1, 1] to [0, 1]. * 0.5 + 0.5
    let light_local = pos_light_space.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    // Fetch shadow map, use HW PCF and comparison                                    current depth
    return textureSampleCompareLevel(shadow[0], shadow_sampler, light_local, light_idx, pos_light_space.z * proj_correction);
}

@fragment
fn fs_main(vout : VSOutput) -> @location(0) vec4<f32> {
    var materials_count : u32 = arrayLength(&materials);
    var default_material_index : u32 = materials_count - 1u;
    var material = materials[vout.material_index];
    let texcoord = vec2<f32>(vout.texcoord.x, 1.0 - vout.texcoord.y);

    var kd = material.kd.rgb;
    if (material.map_kd != INVALID_INDEX) {
        kd = textureSample(textures[material.map_kd], samplers[texture_sampler_ids[material.map_kd]], texcoord).rgb;
    }

    var color = materials[default_material_index].kd.rgb;

    var ks = material.ks.rgb;
    if (material.map_ks != INVALID_INDEX) {
        ks = textureSample(textures[material.map_ks], samplers[texture_sampler_ids[material.map_ks]], texcoord).rgb;
    }

    var ns = material.ns;
    if (material.map_ns != INVALID_INDEX) {
        ns = textureSample(textures[material.map_ns], samplers[texture_sampler_ids[material.map_ns]], texcoord).r;
    }

    // Output kd as color.
    if (material.illum == 11u) {
        return vec4<f32>(kd, 1.0);
    }

    if (material.illum == 12u) {
        return vec4<f32>(ks, 1.0);
    }

    if (material.illum == 13u) {
        return vec4<f32>(texcoord, 0.0, 1.0);
    }

    var n = normalize(vout.normal_eye_space);
    if (material.map_norm != INVALID_INDEX) {
        n = unpack_normal_map(material.map_norm, texcoord);
        let tbn = tbn_matrix(vout.tangent_eye_space, vout.normal_eye_space);
        n = normalize(tbn * n);
    }
    let view_mat = mat4x4<f32>(vout.view_mat_x, vout.view_mat_y, vout.view_mat_z, vout.view_mat_w);
    color = blinn_phong_shading_eye_space(view_mat, vout.pos_world, vout.pos_eye_space, n, kd, ks, ns, material.illum);

    // Ambient on.
    if (material.illum != 0u) {
        var ka = material.ka.rgb;
        if (material.map_ka != INVALID_INDEX) {
            ka = textureSample(textures[material.map_ka], samplers[texture_sampler_ids[material.map_ka]], texcoord).rgb;
        }

        var ia = vec3<f32>(0.0, 0.0, 0.0);
        for (var i : u32 = 0u; i < lights.len; i++) {
            ia += lights.data[i].color;
        }
        ia = 0.08 * ia / f32(lights.len);
        color += ka * ia * kd;
    }

    return vec4<f32>(color, 1.0);
}
