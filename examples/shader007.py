"""
Shader005:

Shader example using bk7084 framework, shadow mapping.

OpenGL allows us to disable writing to the depth buffer by setting its depth mask to GL_FALSE.
"""

import imgui
import numpy as np

from bk7084 import Window, app, Camera, gl, ShaderProgram, VertexShader, PixelShader
from bk7084.app import ui
from bk7084.app.input import KeyCode
from bk7084.assets import default_asset_mgr
from bk7084.graphics import draw, VertexArrayObject, VertexBuffer, VertexLayout, VertexAttrib, VertexAttribFormat, \
    DirectionalLight
from bk7084.graphics.framebuffer import Framebuffer, ColorAttachment
from bk7084.graphics.texture import Texture
from bk7084.math import Vec3, Mat4
from bk7084.misc import PaletteDefault

vertex_src = """
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

    frag_pos = pos.xyz;  // vertex position in camera space
     
    frag_pos_light_space = light_mat * model_mat * vec4(a_position, 1.0);

    light_pos = vec3(view_mat * vec4(in_light_pos, 1.0));
    light_dir = vec3(view_mat * vec4(in_light_dir, 0.0));

    v_color = a_color;
    v_texcoord = a_texcoord;
    v_normal = mat3(transpose(inverse(view_mat * model_mat))) * a_normal;
    v_tangent = mat3(transpose(inverse(view_mat * model_mat))) * a_tangent;

    gl_Position = proj_mat * view_mat * model_mat * vec4(a_position, 1.0);
}
"""

fragment_src = """
# version 330

struct Material {
    sampler2D diffuse_map;
    sampler2D bump_map;
    sampler2D normal_map;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
    bool enabled;
    bool use_diffuse_map;
    bool use_normal_map;
    bool use_bump_map;
    bool use_parallax_map;
};


in vec4 v_color;
in vec3 v_normal;
in vec2 v_texcoord;
in vec3 v_tangent;
in vec3 frag_pos;
in vec3 world_pos;
in vec3 light_pos;
in vec3 light_dir;
in vec4 frag_pos_light_space;

out vec4 frag_color;

uniform bool shading_enabled;
uniform Material mtl;
uniform vec3 light_color;
uniform bool is_directional;

uniform bool shadow_map_enabled;
uniform sampler2D shadow_map;

float calc_shadow(vec4 pos_in_light_space, vec3 normal, vec3 l) {
    float bias = max(0.00005 * (1.0 - dot(normal, l)), 0.0005); 
    if (shadow_map_enabled) {
        vec3 proj_coords = pos_in_light_space.xyz / pos_in_light_space.w;
        proj_coords = proj_coords * 0.5 + 0.5;
        float depth = proj_coords.z;
        float max_depth = texture(shadow_map, proj_coords.xy).r; 
        return depth - bias > max_depth ? 1.0 : 0.0;
        //return depth > max_depth ? 1.0 : 0.0;
    } else {
        return 0.0;
    }
}

// Blinn Phong BRDF in Camera Space
vec3 blinnPhongBRDF(vec3 light_dir, vec3 view_dir, vec3 normal, vec3 diffuse_color, vec3 specular_color, float shininess) {
    vec3 color = diffuse_color;
    vec3 half_dir = normalize(view_dir + light_dir);
    float spec_dot = max(dot(half_dir, normal), 0.0);
    color += pow(spec_dot, shininess) * specular_color;
    return color;
}

vec4 shading(vec3 ambient_color, vec3 light_dir, vec3 view_dir, vec3 light_color, vec3 n, vec3 diffuse_color, vec3 specular_color, float shininess, float shadow) {
    vec3 luminance = ambient_color.rgb * 0.1 + diffuse_color * 0.3;

    float illuminance = dot(light_dir, n);

    if (illuminance > 0.0) {
        vec3 brdf = blinnPhongBRDF(light_dir, view_dir, n, diffuse_color.rgb, specular_color.rgb, shininess);

        luminance += (1.0 - shadow) * brdf * illuminance * light_color.rgb * 0.6;
    }

    return vec4(luminance, 1.0);
}

mat3 tangentSpaceMatrix () {

    vec3 tangent = normalize(v_tangent);
    vec3 normal = normalize(v_normal);
    vec3 bitangent = normalize (cross(normal, tangent));

    return mat3(tangent, bitangent, normal);
}

vec2 parallaxMap (sampler2D tex) { 
    float parallax_factor = 10.0;

    vec3 view_dir = normalize(-frag_pos);

    float height =  texture(tex, v_texcoord).x;
    vec2 displacement = view_dir.xy / view_dir.z * (height * parallax_factor);

    return v_texcoord - displacement;
} 

vec3 bumpMap (sampler2D tex) {

    //https://developer.download.nvidia.com/CgTutorial/cg_tutorial_chapter08.html
    vec2 tex_size = 1.0/textureSize(tex, 0);


    // texture gradient (only x component is necessary since bump map is gray scale)
    vec2 grad = vec2(texture(tex, v_texcoord).x - texture(tex, v_texcoord+vec2(tex_size.x,0)).x,
                  texture(tex, v_texcoord+vec2(0,tex_size.y)).x - texture(tex, v_texcoord).x);

    // bump map multiplier (to enhance effect)
    grad *= 3.0;

    // cross product of vector (0, 1, grad.x) x (1, 0, grad.y)
    vec3 bump_map = normalize(vec3(-grad.y, grad.x, 1.0));

    // place bump_map in tangent space
    mat3 TBN = tangentSpaceMatrix ();
    bump_map = normalize(TBN * bump_map);

    return bump_map;

}

vec3 normalMap (sampler2D tex) {

    mat3 TBN = tangentSpaceMatrix ();

    vec3 normal_map = texture(tex, v_texcoord).xyz*2.0 - vec3(1.0);
    normal_map = normalize(TBN * normal_map);

    return normal_map;
}

void main() {
    vec3 _light_dir = vec3(0.0);
    if (is_directional)
        _light_dir = -normalize(light_dir);
    else
        _light_dir = normalize(light_pos - frag_pos);
    
    vec3 view_dir = normalize(-frag_pos);
    vec3 n = normalize(v_normal);

    vec4 diffuse_color;
    vec4 specular_color;
    float shininess;
    vec4 ambient_color;

    if (mtl.enabled) {
        if (mtl.use_diffuse_map) {
            diffuse_color = texture(mtl.diffuse_map, v_texcoord);
        } else {
            diffuse_color = vec4(mtl.diffuse, 1.0);
        }
    
        if (mtl.use_normal_map) {
            n = normalMap(mtl.normal_map);
        }

        else if (mtl.use_bump_map) {
            n = bumpMap(mtl.diffuse_map);
            diffuse_color = vec4(mtl.diffuse, 1.0);
        }
        else if (mtl.use_parallax_map) {
            vec2 tex_displacement = parallaxMap(mtl.diffuse_map);
            diffuse_color = texture(mtl.diffuse_map, tex_displacement);
        }

        specular_color = vec4(mtl.specular, 1.0);
        shininess = mtl.shininess;
        ambient_color = vec4(mtl.ambient, 1.0);
    } else {
        diffuse_color = v_color;
        specular_color = vec4(1.0, 1.0, 1.0, 1.0);
        shininess = 1.0;
        ambient_color = vec4(0.5, 0.5, 0.5, 1.0);
    }

    float shadow = calc_shadow(frag_pos_light_space, n, _light_dir);

    if (shading_enabled) {
        frag_color = shading(ambient_color.xyz, _light_dir, view_dir.xyz, light_color.xyz, n, diffuse_color.xyz, specular_color.xyz, shininess, shadow);
    } else {
        frag_color = diffuse_color;
    }
}
"""


# Setup window and add camera
from bk7084.scene import Mesh, Scene

window = Window("BK7084: shadow mapping", width=1024, height=1024)

sphere = Mesh("./models/uv_sphere.obj",
              texture='./textures/checker.png',
              texture_enabled=True
              # vertex_shader=vertex_src, pixel_shader=fragment_src
              )
# cube.cast_shadow = True
# cube.shading_enabled = False
# cube.texture_enabled = False
# print(sphere.alternate_texture_enabled)
cube = default_asset_mgr.get_or_create_mesh_data('cube', 'models/cube.obj')
cube.texture_enabled = True

model = Mesh("./models/spot_cow.obj",
             texture='./textures/checker.png',
             # vertex_shader=vertex_src, pixel_shader=fragment_src
             )
model.cast_shadow = True
model.apply_transform(Mat4.from_translation(Vec3(2.0, 0.0, 2.0)))
model.texture_enabled = True

cube.transformation = model.transformation

ground = Mesh(
    vertices=[[-10.0, 0.0, -10.0],
              [10.0,  0.0, -10.0],
              [10.0,  0.0,  10.0],
              [-10.0, 0.0,  10.0]],
    colors=[PaletteDefault.GreenB.as_color()],
    normals=[[0, 1, 0]],
    uvs=[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
    faces=[[(0, 1, 2, 3), (0, 1, 2, 3), (0, 0, 0, 0)]],
    texture='./textures/checker.png',
    # vertex_shader=vertex_src,
    # pixel_shader=fragment_src,
)
ground.texture_enabled = True

# scene = Scene(window, [cube, ground], light=DirectionalLight(), draw_light=False)
scene = Scene(window, [sphere, ground, model, cube], draw_light=True)
scene.create_camera(Vec3(6, 6.0, 6.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, zoom_enabled=True, safe_rotations=False, near=0.1, far=100.0)

animate = False

# framebuffer = Framebuffer(1024, 1024, depth_shader_accessible=True)
# depth_map_shader = ShaderProgram(
#     VertexShader.from_file('./shaders/depth_map.vert'),
#     PixelShader.from_file('./shaders/depth_map.frag')
# )


@window.event
def on_draw(dt):
    # # first pass: render to depth map
    # with framebuffer:
    #     framebuffer.enable_depth_test()
    #     framebuffer.clear(PaletteDefault.BlueB.as_color().rgba)
    #     scene.draw(shader=depth_map_shader)
    #
    # # second pass: render scene as normal with shadow mapping (using depth map from 1st pass)
    # gl.glClearColor(*PaletteDefault.Background.as_color().rgba)
    # gl.glEnable(gl.GL_DEPTH_TEST)
    # gl.glClear(gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT)
    # scene.draw(shadow_map=framebuffer.depth_attachments[0], shadow_map_enabled=True)

    # scene.draw(auto_shadow=False)
    scene.draw_v2(auto_shadow=True)

    # # first pass: render to depth map
    # with scene._framebuffer:
    #     scene._framebuffer.enable_depth_test()
    #     scene._framebuffer.clear(PaletteDefault.BlueB.as_color().rgba)
    #     scene.draw(shader=scene._depth_map_pipeline)
    #
    # # second pass: render scene as normal with shadow mapping (using depth map from 1st pass)
    # gl.glClearColor(*PaletteDefault.Background.as_color().rgba)
    # gl.glEnable(gl.GL_DEPTH_TEST)
    # gl.glClear(gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT)
    # scene.draw(shadow_map=scene._framebuffer.depth_attachments[0], shadow_map_enabled=True)


@window.event
def on_key_press(key, mods):
    global animate
    if key == KeyCode.A:
        animate = not animate

    if key == KeyCode.C:
        scene._depth_map_framebuffer.save_color_attachment()

    if key == KeyCode.D:
        scene._depth_map_framebuffer.save_depth_attachment(scene.main_camera.near, scene.main_camera.far, False)

    if key == KeyCode.Up:
        ground.apply_transform(Mat4.from_translation(Vec3(0.0, 0.1, 0.0)))

    if key == KeyCode.Down:
        ground.apply_transform(Mat4.from_translation(Vec3(0.0, -0.1, 0.0)))


@window.event
def on_update(dt):
    if animate:
        sphere.apply_transform(Mat4.from_axis_angle(Vec3.unit_y(), 45 * dt, True))
        model.apply_transform(Mat4.from_axis_angle(Vec3.unit_y(), -45 * dt, True))
        cube.apply_transform(Mat4.from_axis_angle(Vec3.unit_y(), -15 * dt, True))


app.init(window)
app.run()
