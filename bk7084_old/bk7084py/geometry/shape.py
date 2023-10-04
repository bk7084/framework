import abc
from typing import Sequence

import numpy as np

from ..graphics.util import DrawingMode
from ..misc import Color, PaletteDefault


class Shape(metaclass=abc.ABCMeta):
    @classmethod
    def __subclasshook__(cls, subclass):
        return hasattr(subclass, 'vertices') and \
               hasattr(subclass, 'vertex_count') and hasattr(subclass, 'indices') and \
               hasattr(subclass, 'index_count') and hasattr(subclass, 'drawing_mode')

    def __init__(self, vertex_count, colors: Sequence[Color]):
        self._vertex_count = vertex_count

        colors_count = len(colors)
        if colors_count == 0:
            self._colors = list(PaletteDefault.GreenA.as_color())
        elif colors_count == 1:
            self._colors = list(colors)
        elif colors_count != vertex_count:
            raise ValueError(f'{self.__class__.__name__} has {vertex_count} vertices, '
                             f'but only {colors_count} points are specified with a color.')
        else:
            self._colors = list(colors[:vertex_count])

    @property
    def color(self):
        if len(self._colors) == 1:
            return self._colors[0]
        else:
            return self._colors

    @color.setter
    def color(self, new_color):
        if isinstance(new_color, (list, tuple)):
            if len(new_color) < 3:
                raise ValueError(
                    f'Triangle has three vertices, but only {len(new_color)} points are specified with new color.')
            self._colors = new_color[:3]
        elif isinstance(new_color, Color):
            self._colors = [new_color]

    @property
    def colors(self):
        if len(self._colors) == 1:
            return np.tile(self._colors[0].rgba, self._vertex_count).ravel()
        else:
            colors = np.array([], dtype=np.float32)
            for color in self._colors:
                colors = np.concatenate([colors, color.rgba.astype(np.float32)])
            return colors.ravel()

    @property
    @abc.abstractmethod
    def vertices(self) -> np.ndarray:
        raise NotImplementedError

    @property
    @abc.abstractmethod
    def vertex_count(self) -> int:
        raise NotImplementedError

    @property
    @abc.abstractmethod
    def indices(self):
        raise NotImplementedError

    @property
    @abc.abstractmethod
    def index_count(self):
        raise NotImplementedError

    @property
    @abc.abstractmethod
    def drawing_mode(self) -> DrawingMode:
        raise NotImplementedError

