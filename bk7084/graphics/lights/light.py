import abc
import math

from ...math import Vec3
from ...misc import Color


class Light(metaclass=abc.ABCMeta):
    @classmethod
    def __subclasshook__(cls, subclass):
        return hasattr(subclass, 'matrix')

    def __init__(self, position: Vec3, color: Color, **kwargs):
        """

        Args:
            direction:
            color:
            **kwargs:
                sm_width (int): shadow map width
                sm_height (int): shadow map height
                sm_near (float): shadow map near plane
                sm_far (float): shadow map far plane
                sm_fov (float): shadow map vertical field of view (in degrees or radians).
        """
        self._position = position
        self._color = color
        self._is_dirty = True

        # Shadow map related variables
        self._sm_width = kwargs.get('sm_width', 1024)
        self._sm_height = kwargs.get('sm_width', 1024)
        self._sm_near = kwargs.get('sm_near', 0.1)
        self._sm_far = kwargs.get('sm_far', 100.0)
        fov = kwargs.get('sm_fov', 60.0)
        is_degrees = kwargs.get('degrees', True)
        self._sm_fov = math.radians(fov) if is_degrees else fov
        self._sm_is_perspective = kwargs.get('is_perspective', False)

    @property
    def position(self):
        return self._position

    @position.setter
    def position(self, value: Vec3):
        self._position = value
        self._is_dirty = True

    @property
    def color(self):
        return self._color

    @color.setter
    def color(self, value: Color):
        self._color = value

    @property
    @abc.abstractmethod
    def matrix(self):
        raise NotImplementedError

    @property
    @abc.abstractmethod
    def view_matrix(self):
        raise NotImplementedError

    @property
    @abc.abstractmethod
    def is_perspective(self):
        raise NotImplementedError

    @property
    def is_directional(self):
        return NotImplementedError
