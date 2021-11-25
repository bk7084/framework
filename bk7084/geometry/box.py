from typing import Sequence

import numpy as np

from dataclasses import dataclass

from .shape import Shape
from ..graphics.util import DrawingMode
from ..math import Vec3
from ..misc import Color, PaletteDefault


class Box(Shape):
    def __init__(self, width: float = 1.0, height: float = 1.0, depth: float = 1.0,
                 colors: Sequence[Color] = (PaletteDefault.YellowB.as_color(),)):
        super().__init__(8, colors)
        self._width = width
        self._height = height
        self._depth = depth

        self._vertices, self._indices = Box.gen_vertices_and_indices(width, height, depth)

    @staticmethod
    def gen_vertices_and_indices(w, h, d):
        vertices = []

        start = Vec3(-w * 0.5, -h * 0.5, -d * 0.5)

        for ix in range(0, 2):
            for iz in range(0, 2):
                for iy in range(0, 2):
                    vertices.append(start + Vec3(ix * w, iy * h, iz * d))

        indices = np.array([0, 1, 2, 2, 1, 3,
                            4, 5, 0, 0, 5, 1,
                            6, 7, 4, 4, 7, 5,
                            2, 3, 6, 6, 3, 7,
                            5, 7, 1, 1, 7, 3,
                            6, 4, 2, 2, 4, 0], dtype=np.uint32)

        return np.asarray(vertices, dtype=np.float32).ravel(), indices

    @property
    def vertices(self) -> np.ndarray:
        return self._vertices

    @property
    def vertex_count(self) -> int:
        return len(self._vertices) // 3

    @property
    def drawing_mode(self) -> DrawingMode:
        return DrawingMode.Triangles

    @property
    def indices(self):
        return self._indices

    @property
    def index_count(self):
        return len(self._indices)
