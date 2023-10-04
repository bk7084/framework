"""
Shader003:

Shader example using bk7084 framework, one interleaved buffer.
"""

import numpy as np

from bk7084 import app, gl, Window
from bk7084.graphics import VertexBuffer, VertexLayout, VertexArrayObject, VertexAttrib, VertexAttribFormat, \
    VertexAttribDescriptor, ShaderProgram, VertexShader, PixelShader


window = Window('Shader 003', gl_version=(3, 3), profile='core')

vertices = np.array([-0.5, -0.5, 0.0, 0.933, 0.376, 0.333,
                     0.5, -0.5, 0.0, 0.376, 0.827, 0.580,
                     0.0, 0.5, 0.0, 0.000, 0.686, 0.725], dtype=np.float32)

shader = ShaderProgram(
    VertexShader.from_file('./shaders/basic.vert'),
    PixelShader.from_file('./shaders/basic.frag')
)

vao = VertexArrayObject()

vbo = VertexBuffer(3, VertexLayout(VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3),
                                   VertexAttribDescriptor(VertexAttrib.Color0, VertexAttribFormat.Float32, 3)))

vbo.set_data(vertices)

vao.bind_vertex_buffer(vbo)


@window.event
def on_draw(dt):
    with shader:
        with vao:
            gl.glDrawArrays(gl.GL_TRIANGLES, 0, 3)


app.init(window)
app.run()
