from typing import Sequence

import numpy as np

from .shape import Shape
from ..graphics.util import DrawingMode
from ..math import Vec3
from ..misc import Color, PaletteDefault


class Ray(Shape):
    def __init__(self, origin, direction, colors: Sequence[Color] = (PaletteDefault.GreenB.as_color(),)):
        super().__init__(2, colors)
        self._o = Vec3(origin) if not isinstance(origin, Vec3) else origin
        self._d = Vec3(direction) if not isinstance(direction, Vec3) else direction

    def __str__(self):
        _str = 'Ray ⚬ {} ⟶ {}'
        return _str.format(self._o, self._d)

    @property
    def vertices(self) -> np.ndarray:
        return np.concatenate([self._o, self._o + self._d]).ravel()

    @property
    def vertex_count(self) -> int:
        return 2

    @property
    def drawing_mode(self) -> DrawingMode:
        return DrawingMode.Lines

    @property
    def origin(self):
        return self._o

    @origin.setter
    def origin(self, value: Vec3):
        self._o = value
        self._is_dirty = True

    @property
    def direction(self):
        return self._d

    @direction.setter
    def direction(self, value: Vec3):
        self._d = value
        self._is_dirty = True

    @property
    def indices(self):
        return np.array([0, 1], dtype=int)

    @property
    def index_count(self):
        return 2

