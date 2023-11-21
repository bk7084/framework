__all__ = ['Ray', 'Triangle']

from bk7084.math import Vec3


class Ray:
    def __init__(self, origin: Vec3, direction: Vec3):
        self._origin = origin
        self._direction = direction

    @property
    def origin(self):
        return self._origin

    @origin.setter
    def origin(self, o: Vec3):
        self._origin = o

    @property
    def direction(self):
        return self._direction

    @direction.setter
    def direction(self, d: Vec3):
        self._direction = d


class Triangle:
    def __init__(self, p0: Vec3, p1: Vec3, p2: Vec3):
        self._p0 = p0
        self._p1 = p1
        self._p2 = p2

    @property
    def p0(self):
        return self._p0

    @property
    def p1(self):
        return self._p1

    @property
    def p2(self):
        return self._p2