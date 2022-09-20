import math

import numpy as np

from .shape import Shape
from ..graphics.util import DrawingMode
from ..math import Vec3, Vec2
from ..misc import PaletteDefault


class Grid(Shape):
    def __init__(self, origin=Vec3(0.0), width=20.0, height=20.0,
                 spacing_width=1.0, spacing_height=1.0):
        points, colors = self._generate_lines(origin, width, height, spacing_width, spacing_height)
        super().__init__(len(points), colors)
        self._origin = origin
        self._width = width
        self._height = height
        self._spacing = Vec2(spacing_width, spacing_height)
        self._points = points
        self._colors = colors

    @staticmethod
    def _generate_lines(o, w, h, dw, dh):
        lines = []
        colors = []
        count_w = math.ceil(w / dw)
        count_h = math.ceil(h / dh)
        for w_i in range(0, count_w + 1):
            lines.extend([Vec3(-w / 2 + w_i * dw, 0.0, -h / 2) + o,
                          Vec3(-w / 2 + w_i * dw, 0.0, h / 2) + o])
            if w_i == w // 2:
                colors.append(PaletteDefault.RedA.as_color())
                colors.append(PaletteDefault.RedA.as_color())
            else:
                colors.append(PaletteDefault.BrownB.as_color())
                colors.append(PaletteDefault.BrownB.as_color())

        for h_i in range(0, count_h + 1):
            lines.extend([Vec3(-w / 2, 0.0, -h / 2 + h_i * dh) + o,
                          Vec3(w / 2, 0.0, -h / 2 + h_i * dh) + o])
            if h_i == w // 2:
                colors.append(PaletteDefault.BlueA.as_color())
                colors.append(PaletteDefault.BlueA.as_color())
            else:
                colors.append(PaletteDefault.BrownB.as_color())
                colors.append(PaletteDefault.BrownB.as_color())

        return np.asarray(lines, dtype=np.float32).reshape((-1, 3)), colors

    @property
    def vertices(self) -> np.ndarray:
        return self._points.ravel()

    @property
    def vertex_count(self) -> int:
        return len(self._points)

    @property
    def indices(self):
        return np.array(list(range(0, len(self._points))), dtype=np.uint32)

    @property
    def index_count(self):
        return len(self._points)

    @property
    def drawing_mode(self) -> DrawingMode:
        return DrawingMode.Lines
