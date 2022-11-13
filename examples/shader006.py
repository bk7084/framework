import imgui
import numpy as np

from bk7084 import Window, app, Camera, gl, ShaderProgram, VertexShader, PixelShader
from bk7084.app import ui
from bk7084.app.input import KeyCode
from bk7084.assets import default_asset_mgr
from bk7084.graphics import draw, VertexArrayObject, VertexBuffer, VertexLayout, VertexAttrib, VertexAttribFormat
from bk7084.graphics.framebuffer import Framebuffer, ColorAttachment
from bk7084.graphics.texture import Texture
from bk7084.math import Vec3, Mat4
from bk7084.misc import PaletteDefault

vertex_src = """
# version 330

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec2 a_texcoord;

out vec2 v_texcoord;

uniform mat4 model_mat;

void main()
{
    gl_Position = model_mat * vec4(a_position, 1.0);
    v_texcoord = a_texcoord;
}
"""

fragment_src = """
# version 330
in vec2 v_texcoord;

uniform sampler2D screen_texture;

out vec4 frag_color;

void main()
{
    frag_color = texture(screen_texture, v_texcoord);
}
"""

# Setup window and add camera
from bk7084.scene import Mesh, Scene

window = Window("BK7084: Framebuffer", width=1024, height=1024)

cube = Mesh("./models/spot_cow.obj")

scene = Scene(window, [cube], draw_light=True)
scene.create_camera(Vec3(2, 1.0, 2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, zoom_enabled=True, safe_rotations=False)

animate = False

framebuffer = Framebuffer(1024, 1024, depth_shader_accessible=True)
screen_shader = ShaderProgram(
    VertexShader(vertex_src),
    PixelShader(fragment_src)
)
vao = VertexArrayObject()
vbo = VertexBuffer(4, VertexLayout((VertexAttrib.Position, VertexAttribFormat.Float32, 3),
                                   (VertexAttrib.TexCoord0, VertexAttribFormat.Float32, 2)))
vbo.set_data(np.array([-0.5, -0.5, 0.0, 0.0, 0.0,
                       0.5, -0.5, 0.0, 1.0, 0.0,
                       0.5, 0.5, 0.0, 1.0, 1.0,
                       -0.5, 0.5, 0.0, 0.0, 1.0], dtype=np.float32))
vao.bind_vertex_buffer(vbo, [0, 1])

model_mat = Mat4.identity()


@window.event
def on_draw(dt):
    # first pass
    with framebuffer:
        framebuffer.clear(PaletteDefault.BlueB.as_color().rgba)
        framebuffer.enable_depth_test()
        scene.draw()

    # second pass
    gl.glClearColor(*PaletteDefault.RedB.as_color().rgba)
    gl.glDisable(gl.GL_DEPTH_TEST)
    gl.glClear(gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT)
    with screen_shader:
        screen_shader['model_mat'] = model_mat
        screen_shader['screen_texture'] = 0
        screen_shader.active_texture_unit(0)
        with framebuffer.color_attachments[0]:
            with vao:
                gl.glDrawArrays(gl.GL_TRIANGLE_FAN, 0, 4)


@window.event
def on_key_press(key, mods):
    global animate
    if key == KeyCode.A:
        animate = not animate

    if key == KeyCode.C:
        framebuffer.save_color_attachment()

    if key == KeyCode.D:
        framebuffer.save_depth_attachment(scene.main_camera.near, scene.main_camera.far)


@window.event
def on_update(dt):
    if animate:
        cube.apply_transform(Mat4.from_axis_angle(Vec3.unit_y(), 45 * dt, True))
        global model_mat
        model_mat = Mat4.from_rotation_z(45 * dt, True) * model_mat


app.init(window)
app.run()
