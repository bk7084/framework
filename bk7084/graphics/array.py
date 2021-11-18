import ctypes

from .buffer import VertexBuffer, IndexBuffer
from .. import gl


# TODO: deal with ibo


class VertexArrayObject:
    """
    Vertex Array Object manages client side states: VBO bindings, IBO bindings and vertex layout.

    For instance you can bind one VAO, then bind a VBO, an IBO and configure the vertex layout,
    bind another VAO, another VBO and IBO and configure a different vertex layout.

    Whenever you bind the first VAO, the VBO and IBO associated with it are bound and the associated
    vertex format is used. Binding the second VAO will then switch to the other pair of VBO/IBO and
    respective vertex layout.
    """
    def __init__(self):
        self._id = gl.glGenVertexArrays(1)

    def __del__(self):
        gl.glDeleteVertexArrays(1, (self._id, ))

    def __enter__(self):
        self.bind()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.unbind()

    @property
    def raw_id(self):
        return self._id

    def bind_vertex_buffer(self, buffer: VertexBuffer, index: int = 0):
        self.bind()

        buffer.bind()

        for i, (attrib, (fmt, dim)) in enumerate(buffer.layout.description.items()):
            gl.glEnableVertexAttribArray(i + index)
            gl.glVertexAttribPointer(i + index, dim, fmt.gl_type, gl.GL_FALSE,
                                     buffer.layout.stride, ctypes.c_void_p(buffer.layout.offset_of(attrib)))

        buffer.unbind()
        self.unbind()

    def bind_index_buffer(self, buffer: IndexBuffer):
        # TODO
        pass

    def bind(self):
        gl.glBindVertexArray(self._id)

    def unbind(self):
        gl.glBindVertexArray(0)
