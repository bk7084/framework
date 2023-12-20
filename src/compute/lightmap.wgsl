struct Locals {
    model: mat4x4<f32>,
}

struct PConsts {
    instance_base_index: u32,
}

@group(0) @binding(0)
var<storage, read_write> lmaps: texture_storage_2d_array<r32uint, read_write>;

@group(0) @binding(1)
var<storage, read> light_matrices: array<mat4x4<f32>>;