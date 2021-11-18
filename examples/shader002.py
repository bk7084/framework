"""
Shader002:

Shader example using raw OpenGL api, interleaved data in one buffer, raw OpenGL API.
"""

import ctypes

from bk7084 import app, gl, Window
from bk7084.graphics import ShaderProgram, VertexShader, PixelShader
import numpy as np

vertex_src = """
# version 330

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec3 a_color;

out vec3 v_color;

void main()
{
    gl_Position = vec4(a_position, 1.0);
    v_color = a_color;
}
"""

fragment_src = """
# version 330

in vec3 v_color;
out vec4 frag_color;

void main()
{
    frag_color = vec4(v_color, 1.0);
}
"""

window = Window('Shader 002', gl_version=(3, 3), profile='core')


vertices = np.array([-0.5, -0.5, 0.0, 0.933, 0.376, 0.333,
                     0.5, -0.5, 0.0, 0.376, 0.827, 0.580,
                     0.0, 0.5, 0.0, 0.000, 0.686, 0.725], dtype=np.float32)

shader = ShaderProgram(VertexShader(vertex_src), PixelShader(fragment_src))

vao = gl.glGenVertexArrays(1)

vbo = gl.glGenBuffers(1)

gl.glBindBuffer(gl.GL_ARRAY_BUFFER, vbo)
gl.glBufferData(gl.GL_ARRAY_BUFFER, vertices.nbytes, vertices, gl.GL_STATIC_DRAW)
gl.glBindBuffer(gl.GL_ARRAY_BUFFER, 0)

gl.glBindVertexArray(vao)
gl.glEnableVertexAttribArray(0)
gl.glBindBuffer(gl.GL_ARRAY_BUFFER, vbo)
gl.glVertexAttribPointer(0, 3, gl.GL_FLOAT, gl.GL_FALSE, 24, ctypes.c_void_p(0))

gl.glEnableVertexAttribArray(1)
gl.glVertexAttribPointer(1, 3, gl.GL_FLOAT, gl.GL_FALSE, 24, ctypes.c_void_p(12))

gl.glBindBuffer(gl.GL_ARRAY_BUFFER, 0)

gl.glBindVertexArray(0)


@window.event
def on_draw(dt):
    gl.glUseProgram(shader.raw_id)
    gl.glBindVertexArray(vao)
    gl.glDrawArrays(gl.GL_TRIANGLES, 0, 3)
    gl.glBindVertexArray(0)
    gl.glUseProgram(0)


app.init(window)
app.run()
