#version 330

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec4 a_color;
layout (location = 2) in vec2 a_texcoord;
layout (location = 3) in vec3 a_normal;
layout (location = 4) in vec3 a_tangent;

out vec4 v_color;
out vec3 v_normal;
out vec2 v_texcoord;
out vec3 v_tangent;
out vec3 frag_pos;
out vec3 light_pos;
out vec3 light_dir;
out vec4 frag_pos_light_space;

uniform mat4 model_mat;
uniform mat4 view_mat;
uniform mat4 proj_mat;
uniform mat4 light_mat;
uniform vec3 in_light_pos;
uniform vec3 in_light_dir;

void main() {
    vec4 pos = view_mat * model_mat * vec4(a_position, 1.0);

    frag_pos = pos.xyz;

    frag_pos_light_space = light_mat * model_mat * vec4(a_position, 1.0);
    light_pos = vec3(view_mat * vec4(in_light_pos, 1.0));
    light_dir = vec3(view_mat * vec4(in_light_dir, 0.0));

    v_color = a_color;
    v_texcoord = a_texcoord;
    v_normal = mat3(transpose(inverse(view_mat * model_mat))) * a_normal;
    v_tangent = mat3(transpose(inverse(view_mat * model_mat))) * a_tangent;

    gl_Position = proj_mat * view_mat * model_mat * vec4(a_position, 1.0);
}
