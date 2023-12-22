struct Locals {
    model: mat4x4<f32>,
}

struct PConsts {
    instance_base_index: u32,
    light_index: u32,
}

struct VSInput {
    @builtin(instance_index) iidx: u32,
    @location(0) position: vec3<f32>,
}

@group(0) @binding(0)
var lmaps: texture_storage_2d_array<r32uint, read_write>;

@group(1) @binding(0)
var<storage, read> light_matrices: array<mat4x4<f32>>;

@group(2) @binding(0)
var<storage, read> instances: array<Locals>;

var<push_constant> pconsts: PConsts;

@vertex
fn vs_main(vin: VSInput) -> @builtin(position) vec4<f32> {
    let locals = instances[vin.iidx + pconsts.instance_base_index];
    let light_mat = light_matrices[pconsts.light_index];
    return light_mat * locals.model * vec4<f32>(vin.position, 1.0);
}

@fragment
fn fs_main(@builtin(position) pixel_pos: vec4<f32>) -> @location(0) vec4<f32> {
    // var texcoord = position.xy / position.w * 0.5 + 0.5;

    // var val = textureLoad(lmaps, texcoord, pconsts.light_index);
    // val += 1u;
    // textureStore(lmaps, texcoord, pconsts.light_index, val);

    // let texture_size = textureDimensions(lmaps);
    // let coord = vec2<u32>(u32(texcoord.x * f32(texture_size.x)),
    //                       u32(texcoord.y * f32(texture_size.y)));
    // textureStore(lmaps, coord, pconsts.light_index, vec4<u32>(255u));

    // texcoord = texcoord * vec2<f32>(1024.0);

    textureStore(lmaps, vec2<u32>(pixel_pos.xy), pconsts.light_index, vec4<u32>(255u));

    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
