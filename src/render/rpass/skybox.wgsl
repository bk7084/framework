struct Globals {
    view: mat4,
    proj: mat4,
    // time: f32,
}

struct VSOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) {
    var output: VSOutput;
    output.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    output.uv = vec2<f32>(0.0, 0.0);
}

@group(0) @binding(1)
var skybox: texture_cube<f32>;
@group(0) @binding(2)
var skybox_sampler: sampler;

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32> {
    return textureSample(skybox, skybox_sampler, in.uv);
}