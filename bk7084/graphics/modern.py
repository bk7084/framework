import ctypes
import logging
from typing import Union

import numpy as np

from .array import VertexArrayObject
from .buffer import VertexBuffer, IndexBuffer
from .vertex_layout import VertexLayout, VertexAttrib, VertexAttribFormat
from .. import gl
from .. import app
from ..math import Mat4, Vec3
from ..scene import Mesh


def draw(*objs, **kwargs):
    """Draws a shape object."""
    # record shape and its associated vbo. avoid to create multiple vertex buffer object
    # for the same object each time the function is called.
    shader = kwargs.get('shader', app.current_window().default_shader)
    transform = kwargs.get('transform', Mat4.identity())

    if not hasattr(draw, 'shapes_created_gpu_objects'):
        draw.shapes_created_gpu_objects = {}

    from ..geometry.shape import Shape
    from ..scene.mesh import Mesh

    for obj in objs:
        if isinstance(obj, Shape):
            if obj not in draw.shapes_created_gpu_objects:
                vbo = VertexBuffer(obj.vertex_count,
                                   VertexLayout((VertexAttrib.Position, VertexAttribFormat.Float32, 3),
                                                (VertexAttrib.Color0, VertexAttribFormat.Float32, 4)))
                vbo.set_data(obj.interleaved_vertices)

                ibo = IndexBuffer(obj.index_count)
                ibo.set_data(obj.indices.astype(np.uint32))

                vao = VertexArrayObject()

                vao.bind_vertex_buffer(vbo, [0, 1])

                draw.shapes_created_gpu_objects[obj] = (vao, vbo, ibo)

            vao, vbo, ibo = draw.shapes_created_gpu_objects[obj]

            if obj.is_dirty:
                vbo.set_data(obj.interleaved_vertices)
                ibo.set_data(obj.indices.astype(np.uint32))
                obj.is_dirty = False

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

