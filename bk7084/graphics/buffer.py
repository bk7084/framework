import ctypes
import sys

import glfw
import numpy as np

from .util import DataUsage, BufferBindingTarget, GpuObject, BindSemanticObject
from .vertex_layout import VertexLayout
from .. import gl


class Buffer(GpuObject, BindSemanticObject):
    """Representation of an OpenGL buffer object.

    Doesn't hold a memory from cpu side.
    """
    def __init__(self, size: int, target: BufferBindingTarget, usage: DataUsage, mutable=True):
        super().__init__(target.value, -1)
        self._size = size

        if target not in BufferBindingTarget:
            raise ValueError(f'{target} is not a valid OpenGL buffer binding target.')

        if usage not in DataUsage:
            raise ValueError(f'{usage} is not a valid usage.')

        self._target = target.value
        self._usage = usage.value
        self._mutable = mutable

        # Creates a OpenGL buffer object.
        self._id = gl.glGenBuffers(1)

        # Setup its internal state.
        gl.glBindBuffer(self._target, self._id)

        if (not self._mutable) and (gl.current_context_version()[0] >= 4 and gl.current_context_version()[1] >= 4):
            gl.glBufferStorage(self._target, self._size, None, flags=None)
        else:
            gl.glBufferData(self._target, self._size, None, self._usage)

    def _delete(self):
        if self.is_valid():
            gl.glDeleteBuffers(1, [self._id])

    def _activate(self):
        self.bind()

    def _deactivate(self):
        self.unbind()

    @property
    def target(self) -> gl.Constant:
        return self._target

    @property
    def size(self):
        """Returns the size in bytes of the buffer."""
        return self._size

    # TODO: layout related

    def bind(self):
        """Bind this buffer to its OpenGL target."""
        gl.glBindBuffer(self._target, self._id)

    def unbind(self):
        """Reset the buffer's OpenGL target."""
        gl.glBindBuffer(self._target, 0)

    def set_data(self, data: np.ndarray, offset: int = 0, size: int = -1):
        """Update the contents of the buffer.

        Note:
            No boundary check for input data.

        Args:
            data (array like):
                Specifies a pointer to the new data that will be copied into the buffer..

            offset (int):
                Specifies the offset of the buffer where data replacement will begin, measured in bytes.

            size (int):
                Specifies the size in bytes of the data store region being replaced.
                Defaults to -1, the actual size will be the size of buffer.

        Returns:
            None
        """
        if not self._mutable:
            print('Warning: trying to update immutable buffer.', file=sys.stderr)

        gl.glBindBuffer(self._target, self._id)

        if offset == 0 and size == -1:
            gl.glBufferData(self._target, self._size, data, self._usage)
        else:
            gl.glBufferSubData(self._target, offset, size, data)

        gl.glBindBuffer(self._target, 0)

    def map(self, offset=0, size=0, ptr_type=ctypes.POINTER(ctypes.c_byte)):
        """Maps the entire buffer into system memory.

        TODO: test

        Args:
            offset:
                Byte offset from the beginning of the buffer memory.

            size:
                The size of the memory range (whole range by default) to map.

            ptr_type:
                Desired
        Returns:
            Pointer to the mapped range.
        """
        gl.glBindBuffer(self._target, self._id)

        # map whole buffer
        if offset == 0 and size == 0:
            ptr = ctypes.cast(gl.glMapBuffer(self._target, gl.GL_WRITE_ONLY),
                              ctypes.POINTER(ptr_type * self._size)).contents
            return ptr
        else:
            if offset > self._size or offset + size > self._size:
                print('Warning: trying to map the buffer memory with a range greater than its size.', file=sys.stderr)
            ptr = ctypes.cast(gl.glMapBufferRange(self._target, offset, size, gl.GL_MAP_WRITE_BIT), ptr_type).contents
            gl.glUnmapBuffer(self._target)
            return ptr

    def unmap(self):
        """Unmaps mapped memory."""
        gl.glUnmapBuffer(self._target)

    def resize(self, new_size):
        # map -> copy -> init
        dst = (ctypes.c_byte * new_size)()
        gl.glBindBuffer(self._target, self._id)
        data_ptr = gl.glMapBuffer(self._target, gl.GL_READ_ONLY)
        ctypes.memmove(dst, data_ptr, min(new_size, self._size))
        gl.glUnmapBuffer(self._target)

        self._size = new_size
        gl.glBufferData(self._target, self._size, dst, self._usage)


class VertexBuffer(Buffer):
    """Buffer for vertex attribute data."""
    def __init__(self, count, layout: VertexLayout, usage=DataUsage.StaticDraw):
        """Creates a Vertex Buffer Object.

        Args:
            count (int): Number of vertices.
            layout (VertexLayout): Attributes and its format attached to vertices.
            usage (DataUsage): Data usage of vertex buffer.
        """
        super(VertexBuffer, self).__init__(layout.compute_buffer_size(count), BufferBindingTarget.ArrayBuffer, usage)
        self._layout = layout

    @property
    def layout(self):
        return self._layout

    @layout.setter
    def layout(self, new_layout: VertexLayout):
        self._layout = new_layout

    @staticmethod
    def empty():
        return IndexBuffer(0, DataUsage.DynamicDraw)


class IndexBuffer(Buffer):
    """Buffer for index data."""
    def __init__(self, count, usage=DataUsage.StaticDraw):
        """Creates a Buffer object holding indices for drawing. By default, use `np.uint32` as index data type.

        Args:
            count (int): Number of indices.
            usage (DataUsage): Data usage of index buffer.
        """
        self._index_count = count
        size = count * np.dtype(np.uint32).itemsize
        super(IndexBuffer, self).__init__(size, BufferBindingTarget.ElementArrayBuffer, usage)

    @property
    def index_count(self):
        return self._index_count
    
    def set_data(self, data: np.ndarray, offset: int = 0, size: int = -1):
        super(IndexBuffer, self).set_data(data, offset, size)
        self._index_count = len(data.ravel())

    @staticmethod
    def empty():
        return IndexBuffer(0, DataUsage.DynamicDraw)
