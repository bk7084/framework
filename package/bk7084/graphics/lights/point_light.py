from .light import Light
from ...math import Vec3, Mat4
from ...misc import PaletteSvg, Color


class PointLight(Light):
    def __init__(self, position: Vec3 = Vec3(5.0, 5.0, 5.0), color: Color = PaletteSvg.White.as_color(),
                 sm_width=25.0, sm_height=25.0, **kwargs):
        super().__init__(position, color, sm_width=sm_width, sm_height=sm_height, **kwargs)
        self._position = position
        self._color = color
        self._is_dirty = True

    @property
    def position(self):
        return self._position

    @position.setter
    def position(self, value):
        self._position = Vec3(value)
        self._is_dirty = True

    @property
    def color(self):
        return self._color

    @color.setter
    def color(self, value):
        self._color = value

    def _update(self):
        up = Vec3.unit_y()
        dir = (-self._position).normalised
        if dir == -Vec3.unit_y():
            up = Vec3.unit_x()
        elif dir == -Vec3.unit_y():
            up = Vec3.unit_z()
        self._view_mat = Mat4.look_at_gl(self._position, Vec3(0.0), up)
        self._proj_mat = Mat4.orthographic_gl(-self._sm_width / 2.0, self._sm_width / 2.0,
                                              -self._sm_height / 2.0, self._sm_height / 2.0,
                                              self._sm_near, self._sm_far)
        self._is_dirty = False

    @property
    def matrix(self):
        if self._is_dirty:
            self._update()
        return self._proj_mat * self._view_mat

    @property
    def view_matrix(self):
        if self._is_dirty:
            self._update()
        return self._view_mat

    @property
    def is_directional(self):
        return False

    @property
    def is_perspective(self):
        return False
