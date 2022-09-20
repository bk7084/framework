import ctypes

from bk7084 import app, gl, Window, ShaderProgram, VertexShader, PixelShader
import numpy as np

from bk7084.graphics import VertexBuffer, VertexLayout
from bk7084.graphics.vertex_layout import VertexAttrib

vertex_src = """
# version 330 core

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_color;

out vec3 color;

void main()
{
    gl_Position = vec4(in_position, 1.0);
    color = in_color;
}
"""

fragment_src = """
# version 330 core

in vec3 color;

layout (location = 0) out vec4 out_color;

void main()
{
    out_color = vec4(color, 1.0);
}
"""

window = Window('Shader 001', gl_version=(3, 3), profile='core')

vertices = [-0.5, -0.5, 0.0, 1.0, 0.0, 0.0,
            0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
            0.0, 0.5, 0.0, 0.0, 0.0, 1.0]

vertices = np.array(vertices, dtype=np.float32)

shader = ShaderProgram(VertexShader(vertex_src), PixelShader(fragment_src))

vao = gl.glGenVertexArrays(1)
vbo = gl.glGenBuffers(1)

gl.glBindVertexArray(vao)

gl.glBindBuffer(gl.GL_ARRAY_BUFFER, vbo)

gl.glBufferData(gl.GL_ARRAY_BUFFER, vertices.nbytes, vertices, gl.GL_STATIC_DRAW)

gl.glEnableVertexAttribArray(0)
gl.glVertexAttribPointer(0, 3, gl.GL_FLOAT, gl.GL_FALSE, 24, ctypes.c_void_p(0))

gl.glEnableVertexAttribArray(1)
gl.glVertexAttribPointer(1, 3, gl.GL_FLOAT, gl.GL_FALSE, 24, ctypes.c_void_p(12))

gl.glBindBuffer(gl.GL_ARRAY_BUFFER, vbo)

gl.glBindVertexArray(0)


@window.event
def on_draw():
    with shader:
        gl.glBindVertexArray(vao)
        gl.glDrawArrays(gl.GL_TRIANGLES, 0, 3)


app.init(window)
app.run()
