import numpy as np

from .array import VertexArrayObject
from .buffer import VertexBuffer
from .vertex_layout import VertexLayout, VertexAttrib, VertexAttribFormat
from .. import gl
from .. import app
from ..geometry import Shape, Ray, Triangle, Line


def draw(shape: Shape, update=True):
    """Draws a shape object."""
    # record shape and its associated vbo. avoid to create multiple vertex buffer object
    # for the same object each time the function is called.
    if not hasattr(draw, 'created'):
        draw.created = {}

    colors = np.tile(shape.color.rgba, shape.vertex_count)
    buffer_data = np.zeros(7 * shape.vertex_count, dtype=np.float32)

    for i in range(0, shape.vertex_count):
        index = i * 7
        buffer_data.put(list(range(index, index + 3)), shape.vertices[i * 3: i * 3 + 3])
        buffer_data.put(list(range(index + 3, index + 7)), colors[i * 4: i * 4 + 4])

    if shape not in draw.created:
        vbo = VertexBuffer(shape.vertex_count, VertexLayout((VertexAttrib.Position, VertexAttribFormat.Float32, 3),
                                                            (VertexAttrib.Color0, VertexAttribFormat.Float32, 4)))
        vbo.set_data(buffer_data)

        vao = VertexArrayObject()

        vao.bind_vertex_buffer(vbo)

        draw.created[shape] = (vao, vbo)

    vao, vbo = draw.created[shape]

    vbo.set_data(buffer_data)

    with app.current_window().default_shader:
        with vao:
            gl.glDrawArrays(shape.drawing_mode.value, 0, shape.vertex_count)
