// This shader computes the scores for each light position of a single light
// source (directional light).
//
// The input is an array of light maps, each of which is a 2D array of
// floating-point values. The output is an array of scores, one for each
// light map.
//
// A light map is the rasterizition result of objects in the scene from the
// perspective of a light source. The light map is computed by rendering the
// scene from the light source's point of view, and each pixel in the light
// map represents the number of objects that are projected onto that pixel.
//
// The score is computed by firstly summing the light map values, then divided
// by the number of pixels that not equal to zero.
//

@group(0) @binding(0) var<storage, read_write> scores: array<f32>;
@group(1) @binding(0) var lmaps: texture_storage_2d_array<r32uint, read>;

fn compute_score(index: u32) -> f32 {
  var sum: f32 = 0.0;
  var count: f32 = 0.0;
  let size = textureDimensions(lmaps);
  for (var y = 0u; y < size.y; y += 1u) {
    for (var x = 0u; x < size.x; x += 1u) {
      let v = textureLoad(lmaps, vec2<u32>(x, y), index).x;
      sum = sum + f32(v);
      if (v != 0u) {
        count += 1.0;
      }
    }
  }
  return sum / count;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  scores[global_id.x] = compute_score(global_id.x);
}
