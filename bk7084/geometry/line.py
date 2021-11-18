import numbers
from typing import Sequence

import numpy as np

from .shape import Shape
from ..graphics.util import DrawingMode
from ..math import Vec3
from ..misc import Color, PaletteDefault


class Line(Shape):
    def __init__(self, points: Sequence[Vec3], color=PaletteDefault.BrownB.as_color()):
        self._points = np.array(points, dtype=np.float32).reshape((3, len(points)))
        self._points_count = len(points)
        self._color = color

    def __getitem__(self, item: int):
        if not isinstance(item, numbers.Integral) or item < 0 or item > 2:
            raise IndexError(f'Invalid index {item}')
        return self._points[item].reshape((3, 1)).view(Vec3)

    def __iter__(self):
        self._i = 0
        return self

    def __next__(self):
        if self._i < 3:
            value = self._points[self._i]
            self._i += 1
            return value
        else:
            raise StopIteration

    @property
    def drawing_mode(self) -> DrawingMode:
        return DrawingMode.LineStrip

    @property
    def vertex_count(self) -> int:
        return self._points_count

    @property
    def color(self) -> Color:
        return self._color

    @property
    def vertices(self) -> np.ndarray:
        return np.array(self._points).ravel()
