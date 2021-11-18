import abc
import numpy as np

from ..graphics.util import DrawingMode
from ..misc import Color


class Shape(metaclass=abc.ABCMeta):
    @classmethod
    def __subclasshook__(cls, subclass):
        return hasattr(subclass, 'color') and callable(subclass.color) and \
               hasattr(subclass, 'vertices') and callable(subclass.vertices) and \
               hasattr(subclass, 'vertex_count') and callable(subclass.vertex_count) and \
               hasattr(subclass, 'drawing_mode') and callable(subclass.drawing_mode)

    @property
    @abc.abstractmethod
    def color(self) -> Color:
        raise NotImplementedError

    @color.setter
    @abc.abstractmethod
    def color(self, value) -> Color:
        raise NotImplementedError

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
    def drawing_mode(self) -> DrawingMode:
        raise NotImplementedError

