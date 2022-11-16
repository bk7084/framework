import numbers
from typing import Sequence

import numpy as np

from .shape import Shape
from .ray import Ray
from ..graphics.util import DrawingMode
from ..math import Vec3
from ..misc import Color, PaletteDefault


class Triangle(Shape):
    def __init__(self, p0, p1, p2, colors: Sequence[Color] = (PaletteDefault.YellowB.as_color(),)):
        """Creates a triangle from three positions and colors.

        Args:
            p0 (array_like, shape (3,) or list):
                First point of the triangle.

            p1 (array_like, shape (3,) or list):
                Second point of the triangle.

            p2 (array_like, shape (3,) or list):
                Third point of the triangle.

            colors (Sequence[Color]):
                Specifies the color of each point of triangle.
        """
        super(Triangle, self).__init__(3, colors)
        self._points = np.array([p0, p1, p2]).reshape((3, 3))

    def __str__(self):
        _str = 'Triangle [{}, {}, {}]'
        return _str.format(*self._points)

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
    def vertex_count(self) -> int:
        return 3

    @property
    def drawing_mode(self) -> DrawingMode:
        return DrawingMode.Triangles

    @property
    def p0(self) -> Vec3:
        return self._points[0].reshape(3, 1).view(Vec3)

    @property
    def p1(self) -> Vec3:
        return self._points[1].reshape(3, 1).view(Vec3)

    @property
    def p2(self) -> Vec3:
        return self._points[2].reshape(3, 1).view(Vec3)

    @property
    def vertices(self) -> np.ndarray:
        return np.array(self._points).ravel()

    def intersect_with_ray(self, ray: Ray) -> bool:
        a = np.array([[self[0].x - self[1].x, self[0].x - self[2].x, ray.direction.x],
                      [self[0].y - self[1].y, self[0].y - self[2].y, ray.direction.y],
                      [self[0].z - self[1].z, self[0].z - self[2].z, ray.direction.z]])

        b = np.array([self[0].x - ray.origin.x, self[0].y - ray.origin.y, self[0].z - ray.origin.z])

        x = np.linalg.solve(a, b)

        if x[0] < 0 or x[1] < 0 or x[0] + x[1] > 1 or x[2] < 0:
            return False

        return True

    @property
    def indices(self):
        return np.array([0, 1, 2], dtype=int)

    @property
    def index_count(self):
        return 3
