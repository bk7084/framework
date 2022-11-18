import enum
import math

import numpy as np

from .shape import Shape
from ..graphics.util import DrawingMode
from ..math import Vec3, Vec2
from ..misc import PaletteDefault


class AxisAlignment(enum.Enum):
    XY = 0
    XZ = 1
    YX = 2
    YZ = 3
    ZX = 4
    ZY = 5


class Grid(Shape):
    def __init__(self,
                 width=20.0,
                 height=20.0,
                 spacing_width=1.0,
                 spacing_height=1.0,
                 origin=Vec3(0.0),
                 axis_alignment=AxisAlignment.XZ,
                 axis_marker=False):
        points, colors = self._generate_lines(origin, width, height, spacing_width, spacing_height, axis_alignment,
                                              axis_marker)
        super().__init__(len(points), colors)
        self._origin = origin
        self._width = width
        self._height = height
        self._spacing = Vec2(spacing_width, spacing_height)
        self._points = points
        self._colors = colors

    @staticmethod
    def _generate_lines(o, w, h, dw, dh, alignment, axis_marker):
        lines = []
        colors = []
        count_w = math.ceil(w / dw)
        count_h = math.ceil(h / dh)
        color_axis_vertical = PaletteDefault.Background
        for w_i in range(0, count_w + 1):
            a = -w / 2 + w_i * dw
            b = h / 2
            if alignment is AxisAlignment.XZ:
                lines.extend([Vec3(a, 0.0, -b) + o,
                              Vec3(a, 0.0, b) + o])
                color_axis_vertical = PaletteDefault.BlueA.as_color()
            elif alignment is AxisAlignment.XY:
                lines.extend([Vec3(a, -b, 0.0) + o,
                              Vec3(a, b, 0.0) + o])
                color_axis_vertical = PaletteDefault.GreenA.as_color()
            elif alignment is AxisAlignment.YX:
                lines.extend([Vec3(-b, a, 0.0) + o,
                              Vec3(b, a, 0.0) + o])
                color_axis_vertical = PaletteDefault.RedA.as_color()
            elif alignment is AxisAlignment.YZ:
                lines.extend([Vec3(0.0, a, -b) + o,
                              Vec3(0.0, a, b) + o])
                color_axis_vertical = PaletteDefault.BlueA.as_color()
            elif alignment is AxisAlignment.ZX:
                lines.extend([Vec3(-b, 0.0, a) + o,
                              Vec3(b, 0.0, a) + o])
                color_axis_vertical = PaletteDefault.RedA.as_color()
            elif alignment is AxisAlignment.ZY:
                lines.extend([Vec3(0.0, -b, a) + o,
                              Vec3(0.0, b, a) + o])
                color_axis_vertical = PaletteDefault.GreenA.as_color()
            else:
                pass
            if axis_marker and w_i == w // 2:
                colors.append(color_axis_vertical)
                colors.append(color_axis_vertical)
            else:
                colors.append(PaletteDefault.BrownB.as_color())
                colors.append(PaletteDefault.BrownB.as_color())

        color_axis_horizontal = PaletteDefault.Background
        for h_i in range(0, count_h + 1):
            a = w / 2
            b = -h / 2 + h_i * dh
            if alignment == AxisAlignment.XZ:
                lines.extend([Vec3(-a, 0.0, b) + o,
                              Vec3(a, 0.0, b) + o])
                color_axis_horizontal = PaletteDefault.RedA.as_color()
            elif alignment is AxisAlignment.XY:
                lines.extend([Vec3(-a, b, 0.0) + o,
                              Vec3(a, b, 0.0) + o])
                color_axis_horizontal = PaletteDefault.RedA.as_color()
            elif alignment is AxisAlignment.YX:
                lines.extend([Vec3(b, -a, 0.0) + o,
                              Vec3(b, a, 0.0) + o])
                color_axis_horizontal = PaletteDefault.GreenA.as_color()

            elif alignment is AxisAlignment.YZ:
                lines.extend([Vec3(0.0, -a, b) + o,
                              Vec3(0.0, a, b) + o])
                color_axis_horizontal = PaletteDefault.GreenA.as_color()

            elif alignment is AxisAlignment.ZX:
                lines.extend([Vec3(b, 0.0, -a) + o,
                              Vec3(b, 0.0, a) + o])
                color_axis_horizontal = PaletteDefault.BlueA.as_color()

            elif alignment is AxisAlignment.ZY:
                lines.extend([Vec3(0.0, b, -a) + o,
                              Vec3(0.0, b, a) + o])
            else:
                pass

            color_axis_horizontal = PaletteDefault.BlueA.as_color()
            if axis_marker and h_i == w // 2:
                colors.append(color_axis_horizontal)
                colors.append(color_axis_horizontal)
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
