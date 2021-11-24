"""
Shader000:

Simple shader example using raw OpenGL api.
"""

import ctypes

import numpy as np

from bk7084 import app, gl, Window
from bk7084.graphics import VertexShader, ShaderProgram, PixelShader

vertex_src = """
#version 330 core

layout (location = 0) in vec3 in_position;

void main()
{
    gl_Position = vec4(in_position, 1.0);
}
"""

fragment_src = """
# version 330 core

out vec4 frag_color; 

void main()
{
    frag_color = vec4(0.282, 0.749, 0.890, 1.0);
}
"""

window = Window('Shader 000', gl_version=(3, 3), profile='core')

vertices = np.array([-0.5, -0.5, 0.0,
                     0.5, -0.5, 0.0,
                     0.0, 0.5, 0.0], dtype=np.float32)

shader = ShaderProgram(VertexShader(vertex_src), PixelShader(fragment_src))

vbo = gl.glGenBuffers(1)
gl.glBindBuffer(gl.GL_ARRAY_BUFFER, vbo)
gl.glBufferData(gl.GL_ARRAY_BUFFER, vertices.nbytes, vertices, gl.GL_STATIC_DRAW)
gl.glBindBuffer(gl.GL_ARRAY_BUFFER, 0)

vao = gl.glGenVertexArrays(1)
gl.glBindVertexArray(vao)

gl.glEnableVertexAttribArray(0)
gl.glBindBuffer(gl.GL_ARRAY_BUFFER, vbo)
gl.glVertexAttribPointer(0, 3, gl.GL_FLOAT, gl.GL_FALSE, 3 * 4, ctypes.c_void_p(0))

gl.glBindVertexArray(0)


@window.event
def on_draw(dt):
    gl.glUseProgram(shader.handle)
    gl.glBindVertexArray(vao)
    gl.glDrawArrays(gl.GL_TRIANGLES, 0, 3)
    gl.glBindVertexArray(0)
    gl.glUseProgram(0)


app.init(window)
app.run()
