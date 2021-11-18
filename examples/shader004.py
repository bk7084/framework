"""
Shader004:

Shader example using bk7084 framework, with multiple buffers.
"""

import numpy as np

from bk7084 import app, gl, Window
from bk7084.graphics import ShaderProgram, VertexShader, PixelShader, VertexArrayObject, VertexBuffer, VertexLayout, \
    VertexAttrib, VertexAttribDescriptor, VertexAttribFormat

window = Window('Shader 004', gl_version=(3, 3), profile='core')

positions = np.array([-0.5, -0.5, 0.0,
                      0.5, -0.5, 0.0,
                      0.0, 0.5, 0.0], dtype=np.float32)

colors = np.array([0.833, 0.276, 0.333,
                   0.376, 0.827, 0.580,
                   0.100, 0.586, 0.925], dtype=np.float32)

shader = ShaderProgram(
    VertexShader('./shaders/basic.vert'),
    PixelShader('./shaders/basic.frag')
)

vao = VertexArrayObject()

vbo_position = VertexBuffer(3, VertexLayout(
    VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3))
                            )

vbo_colors = VertexBuffer(3, VertexLayout(
    VertexAttribDescriptor(VertexAttrib.Color0, VertexAttribFormat.Float32, 3))
                          )

vbo_position.set_data(positions)
vbo_colors.set_data(colors)

vao.bind_vertex_buffer(vbo_position)
vao.bind_vertex_buffer(vbo_colors, 1)


@window.event
def on_draw(dt):
    with shader:
        with vao:
            gl.glDrawArrays(gl.GL_TRIANGLES, 0, 3)


app.init(window)
app.run()
