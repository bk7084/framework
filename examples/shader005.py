"""
Shader005:

Shader example using bk7084 framework, uniform demonstration.
"""

import numpy as np

from bk7084 import app, gl, Window
from bk7084.graphics import VertexBuffer, VertexLayout, VertexArrayObject, VertexAttrib, VertexAttribFormat, \
    VertexAttribDescriptor, ShaderProgram, VertexShader, PixelShader

vertex_src = """
# version 330

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec3 a_color;

uniform vec3 displacement;

out vec3 v_color;

void main()
{
    gl_Position = vec4(a_position + displacement, 1.0);
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


window = Window('Shader 005', gl_version=(3, 3), profile='core')

vertices = np.array([-0.5, -0.5, 0.0, 0.933, 0.376, 0.333,
                     0.5, -0.5, 0.0, 0.376, 0.827, 0.580,
                     0.0, 0.5, 0.0, 0.000, 0.686, 0.725], dtype=np.float32)

shader = ShaderProgram(
    VertexShader(vertex_src),
    PixelShader(fragment_src)
)

vao = VertexArrayObject()

vbo = VertexBuffer(3, VertexLayout(VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3),
                                   VertexAttribDescriptor(VertexAttrib.Color0, VertexAttribFormat.Float32, 3)))

vbo.set_data(vertices)

vao.bind_vertex_buffer(vbo)

displacement = np.array([0.0, 0.0, 0.0], dtype=np.float32)


@window.event
def on_draw(dt):
    global displacement
    with shader:
        displacement_loc = gl.glGetUniformLocation(shader.handle, 'displacement')
        displacement += np.array([0.0, dt * 0.1, 0.0])
        gl.glUniform3fv(displacement_loc, 1, displacement)
        with vao:
            gl.glDrawArrays(gl.GL_TRIANGLES, 0, 3)


app.init(window)
app.run()
