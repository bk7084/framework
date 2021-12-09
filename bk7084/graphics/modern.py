import ctypes
import logging
from typing import Union

import numpy as np

from .array import VertexArrayObject
from .buffer import VertexBuffer, IndexBuffer
from .vertex_layout import VertexLayout, VertexAttrib, VertexAttribFormat
from .. import gl
from .. import app
from ..geometry import Shape
from ..math import Mat4
from ..scene import Mesh


def draw(*objs: Union[Shape, Mesh], **kwargs):
    """Draws a shape object."""
    # record shape and its associated vbo. avoid to create multiple vertex buffer object
    # for the same object each time the function is called.
    update = kwargs.get('update', False)
    shader = kwargs.get('shader', app.current_window().default_shader)

    transform = kwargs.get('transform', Mat4.identity())

    if not hasattr(draw, 'shapes_created_gpu_objects'):
        draw.shapes_created_gpu_objects = {}

    for obj in objs:
        if isinstance(obj, Shape):
            colors = obj.colors
            vertices_data = np.zeros(7 * obj.vertex_count, dtype=np.float32)

            for i in range(0, obj.vertex_count):
                index = i * 7
                vertices_data.put(list(range(index, index + 3)), obj.vertices[i * 3: i * 3 + 3])
                vertices_data.put(list(range(index + 3, index + 7)), colors[i * 4: i * 4 + 4])

            if obj not in draw.shapes_created_gpu_objects:
                vbo = VertexBuffer(obj.vertex_count,
                                   VertexLayout((VertexAttrib.Position, VertexAttribFormat.Float32, 3),
                                                (VertexAttrib.Color0, VertexAttribFormat.Float32, 4)))
                vbo.set_data(vertices_data)

                ibo = IndexBuffer(obj.index_count)
                ibo.set_data(obj.indices.astype(np.uint32))

                vao = VertexArrayObject()

                vao.bind_vertex_buffer(vbo, [0, 1])

                draw.shapes_created_gpu_objects[obj] = (vao, vbo, ibo)

            vao, vbo, ibo = draw.shapes_created_gpu_objects[obj]

            if update:
                vbo.set_data(vertices_data)
                ibo.set_data(obj.indices)

            with shader:
                shader.model_mat = transform
                shader.shading_enabled = False
                shader['mtl.enabled'] = False
                with vao:
                    with ibo:
                        gl.glDrawElements(obj.drawing_mode.value, ibo.index_count, gl.GL_UNSIGNED_INT,
                                          ctypes.c_void_p(0))

        elif isinstance(obj, Mesh):
            obj.draw()

        else:
            logging.info('Nothing to draw.')

