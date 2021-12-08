from ...math import Vec3
from ...misc import PaletteSvg, Color


class PointLight:
    def __init__(self, position: Vec3 = Vec3(400, 400, 400), color: Color = PaletteSvg.White.as_color()):
        self._position = position
        self._color = color
