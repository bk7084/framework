#version 330

out vec4 frag_color;

uniform float near;
uniform float far;
uniform bool is_perspective;

float linear_depth(float d) {
    float z = d * 2.0 - 1.0;
    return (2.0 * near * far) / (far + near - z * (far - near));
}

void main() {
    // Could be empty, here is an example to display the depth buffer values
    // as the final color output.
    float d;

    if (is_perspective) {
        d = linear_depth(gl_FragCoord.z) / (far - near);
    } else {
        d = gl_FragCoord.z;
    }

    frag_color = vec4(vec3(d), 1.0);
}