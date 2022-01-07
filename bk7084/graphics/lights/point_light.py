from ...math import Vec3, Mat4
from ...misc import PaletteSvg, Color


class PointLight:
    def __init__(self, position: Vec3 = Vec3(5.0, 5.0, 5.0), color: Color = PaletteSvg.White.as_color(), **kwargs):
        """

        Args:
            position:
            color:
            **kwargs:
                sm_width (int): shadow map width
                sm_height (int): shadow map height
                sm_fov (float): shadow map vertical field of view (in degrees or radians).
                sm_near (float): shadow map near plane
                sm_far (float): shadow map far plane
        """
        self._position = position
        self._color = color

        self._view_mat = Mat4.look_at_gl(self._position, Vec3(0.0), Vec3.unit_y())
        # self._proj_mat = Mat4.perspective_gl(60.0, 1.0, 0.1, 100.0, True)
        self._proj_mat = Mat4.orthographic_gl(-20.0, 20.0, -20.0, 20.0, 0.1, 100.0)
        self._is_dirty = False

    @property
    def position(self):
        return self._position

    @position.setter
    def position(self, value):
        self._position = value
        self._is_dirty = True

    @property
    def color(self):
        return self._color

    @color.setter
    def color(self, value):
        self._color = value

    def _update(self):
        self._view_mat = Mat4.look_at_gl(Vec3(self._position), Vec3(0.0), Vec3.unit_y())
        # self._proj_mat = Mat4.perspective_gl(60.0, 1.0, 0.1, 100.0, True)
        self._proj_mat = Mat4.orthographic_gl(-20.0, 20.0, -20.0, 20.0, 0.1, 100.0)

    @property
    def light_space_matrix(self):
        if self._is_dirty:
            self._update()
        return self._proj_mat * self._view_mat
