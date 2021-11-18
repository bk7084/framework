#version 330 core

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec3 a_color;

uniform mat4 proj_mat;
uniform mat4 view_mat;
uniform mat4 model_mat;

out vec3 v_color;

void main()
{
    gl_Position = proj_mat * view_mat * model_mat * vec4(a_position, 1.0);

    v_color = a_color;
}
