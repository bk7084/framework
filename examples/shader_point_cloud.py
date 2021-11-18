"""
Shader point cloud.
"""

import numpy as np

from bk7084 import app, gl, Window
from bk7084.graphics import VertexBuffer, VertexLayout, ShaderProgram, VertexShader, PixelShader, VertexArrayObject, \
    VertexAttrib, VertexAttribFormat


window = Window('BK7084: Shader Point Cloud', gl_version=(3, 3), profile='core')

vertex_count = 50000

dummy_vertices = np.zeros(vertex_count * 3, dtype=np.float32)

shader = ShaderProgram(VertexShader('./shaders/point_cloud.vert'), PixelShader('./shaders/basic.frag'))

vbo = VertexBuffer(vertex_count, VertexLayout((VertexAttrib.Position, VertexAttribFormat.Float32, 3)))

vbo.set_data(dummy_vertices)

vao = VertexArrayObject()

vao.bind_vertex_buffer(vbo)


@window.event
def on_draw(dt):
    with shader:
        time_loc = gl.glGetUniformLocation(shader.raw_id, 'time')
        resolution_loc = gl.glGetUniformLocation(shader.raw_id, 'resolution')
        with vao:
            gl.glUniform1f(time_loc, window.elapsed_time)
            gl.glUniform2f(resolution_loc, window.width, window.height)

            gl.glDrawArrays(gl.GL_POINTS, 0, vertex_count - 1)


app.init(window)
app.run()
