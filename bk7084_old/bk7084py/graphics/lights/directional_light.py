from .light import Light
from ...math import Vec3, Mat4
from ...misc import Color, PaletteSvg


class DirectionalLight(Light):
    def __init__(self, position: Vec3 = Vec3(8.0, 8.0, 8.0), direction: Vec3 = Vec3(-1.0, -1.0, -1.0),
                 color: Color = PaletteSvg.White.as_color(), sm_width=20.0, sm_height=20.0, **kwargs):
        """

        Args:
            direction:
            color:
            sm_width (int): shadow map width
            sm_height (int): shadow map height
            **kwargs:
                sm_near (float): shadow map near plane
                sm_far (float): shadow map far plane
        """
        super(DirectionalLight, self).__init__(position, color, sm_width=sm_width, sm_height=sm_height, **kwargs)
        self._direction = direction.normalised
        self._is_dirty = True

        self._update()

    def _update(self):
        up = Vec3.unit_y()
        if self._direction == -Vec3.unit_y():
            up = Vec3.unit_x()
        elif self._direction == -Vec3.unit_y():
            up = Vec3.unit_z()
        self._view_mat = Mat4.look_at_gl(Vec3(self._position),
                                         (self._position + self._direction).normalised,
                                         up)
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
    def direction(self):
        return self._direction

    @direction.setter
    def direction(self, value):
        self._direction = Vec3(value).normalised
        self._position = self._direction / 0.577 * -8.0
        self._is_dirty = True

    @property
    def is_directional(self):
        return True

    @property
    def is_perspective(self):
        return False
