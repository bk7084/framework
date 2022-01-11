#version 330

out vec4 energy;

in vec3 v_normal;
in vec3 light_dir;

uniform sampler2D depth_map;
uniform vec2 resolution;  // viewport resolution

void main() {
    vec2 uv = gl_FragCoord.xy / resolution.xy;
    float depth = texture(depth_map, uv).x;

    vec3 l = normalize(light_dir);
    vec3 n = normalize(v_normal);
    float ratio = dot(n, l);
    float bias = max(0.0005 * (1.0 - ratio), 0.0005);

    if (gl_FragCoord.z - bias > depth) {
        // red if it's occluded
        energy = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        // green if it's not occluded
        energy = vec4(0.0, ratio, 0.0, 1.0);
    }
}