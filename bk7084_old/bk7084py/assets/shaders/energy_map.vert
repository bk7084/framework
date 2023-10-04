#version 330

layout (location = 0) in vec3 a_position;
layout (location = 3) in vec3 a_normal;

out vec3 v_normal;
out vec3 light_dir;

uniform vec3 light_pos;
uniform mat4 light_mat;
uniform mat4 light_view_mat;
uniform mat4 model_mat;

void main() {
    gl_Position = light_mat * model_mat * vec4(a_position, 1.0);
    v_normal = mat3(transpose(inverse(light_view_mat * model_mat))) * a_normal;
    light_dir = (light_view_mat * vec4(light_pos, 0.0)).xyz;
}