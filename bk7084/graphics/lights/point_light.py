from ...math import Vec3
from ...misc import PaletteSvg, Color


class PointLight:
    def __init__(self, position: Vec3 = Vec3(400, 400, 400), color: Color = PaletteSvg.White.as_color()):
        self._position = position
        self._color = color

    @property
    def position(self):
        return self._position

    @position.setter
    def position(self, value):
        self._position = value

    @property
    def color(self):
        return self._color

    @color.setter
    def color(self, value):
        self._color = value
