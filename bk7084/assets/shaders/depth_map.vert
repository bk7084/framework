#version 330

layout (location = 0) in vec3 a_position;

uniform mat4 light_mat;
uniform mat4 model_mat;

void main() {
    gl_Position = light_mat * model_mat * vec4(a_position, 1.0);
}