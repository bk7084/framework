import numpy as np

from ..graphics.vertex_layout import VertexAttrib, VertexAttribFormat, VertexLayout


class Component:
    """
    A Component is a container of vertex attributes:
        - positions
        - colors
        - texture coordinates
        - normals
    """
    def __init__(self):
        self._vertex_layout = VertexLayout(
            (VertexAttrib.Position, VertexAttribFormat.Float32, 3),
            (VertexAttrib.Color0, VertexAttribFormat.Float32, 4),
            (VertexAttrib.TexCoord0, VertexAttribFormat.Float32, 2),
            (VertexAttrib.Normal, VertexAttribFormat.Float32, 3),
        )
        self._positions = np.array([], dtype=np.float32)
        self._colors = np.array([], dtype=np.float32)
        self._texcoords = np.array([], dtype=np.float32)
        self._normals = np.array([], dtype=np.float32)

    @property
    def positions(self):
        return self._positions

    @positions.setter
    def positions(self, positions):
        self._positions = positions

    @property
    def colors(self):
        return self._colors

    @colors.setter
    def colors(self, colors):
        self._colors = colors

    @property
    def texcoords(self):
        return self._texcoords

    @texcoords.setter
    def texcoords(self, texcoords):
        self._texcoords = texcoords

    @property
    def normals(self):
        return self._normals

    @normals.setter
    def normals(self, normals):
        self._normals = normals
