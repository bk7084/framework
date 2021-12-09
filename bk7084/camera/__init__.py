from __future__ import annotations

import math
import OpenGL.GL as gl

from .. import app
from ..app.window.input import MouseButton
from ..math import Vec3, Vec4, Mat4


# TODO: decouple camera and control


class Camera:
    def __init__(self, pos, look_at, up, aspect_ratio, fov_v=45.0, near=0.1, far=1000., degrees=True, zoom_enabled=False, safe_rotations=True):
        self._pos = Vec3(pos)
        self._look_at = Vec3(look_at)
        self._up = Vec3(up)
        self._aspect_ratio = aspect_ratio
        self._near = near
        self._far = far
        self._fov = fov_v if not degrees else math.radians(fov_v)
        self._proj_mat = Mat4.perspective_gl(self._fov, self._aspect_ratio, self._near, self._far)
        self._view_mat = Mat4.look_at_gl(self._pos, self._look_at, self._up)
        self._d_angle_x = 0.
        self._d_angle_y = 0.
        self._zoom_enabled = zoom_enabled
        self._safe_rotations = safe_rotations

    @property
    def position(self):
        return self._pos

    @property
    def field_of_view(self):
        return self._fov

    @field_of_view.setter
    def field_of_view(self, value):
        raise NotImplementedError

    @property
    def aspect_ratio(self):
        return self._aspect_ratio

    @aspect_ratio.setter
    def aspect_ratio(self, value):
        self._aspect_ratio = value
        self._update_matrices()

    @property
    def near(self):
        return self._near

    @property
    def far(self):
        return self._far

    @property
    def forward(self):
        return Vec3(-self._view_mat.row(2))

    @property
    def right(self):
        return Vec3(self._view_mat.row(0))

    @property
    def up(self) -> Vec3:
        return self._up

    @property
    def projection_matrix(self) -> Mat4:
        return self._proj_mat

    @property
    def view_matrix(self) -> Mat4:
        return self._view_mat

    @property
    def look_at(self) -> Vec3:
        return self._look_at

    @property
    def zoom_enabled(self):
        return self._zoom_enabled

    @zoom_enabled.setter
    def zoom_enabled(self, value):
        self._zoom_enabled = value

    @property
    def safe_rotations(self):
        return self._safe_rotations

    @safe_rotations.setter
    def safe_rotations(self, value):
        self._safe_rotations = value

    def __enter__(self):
        self._update_matrices()
        return self

    def __exit__(self, *args):
        pass

    def update_view(self, eye, look_at, up):
        self._pos = Vec3(eye)
        self._look_at = Vec3(look_at)
        self._up = Vec3(up)
        self._update_matrices()

    def _update_matrices(self):
        self._proj_mat = Mat4.perspective_gl(self._fov, self._aspect_ratio, self._near, self._far)
        self._view_mat = Mat4.look_at_gl(self._pos, self._look_at, self._up)

    def on_update(self, dt):
        shader = app.current_window().default_shader
        if shader is not None:
            with shader:
                shader.view_mat = self._view_mat
                shader.proj_mat = self._proj_mat
        else:
            gl.glMatrixMode(gl.GL_PROJECTION)
            gl.glLoadTransposeMatrixf(self._proj_mat)

            gl.glMatrixMode(gl.GL_MODELVIEW)
            gl.glLoadTransposeMatrixf(self._view_mat)

    def on_resize(self, width, height):
        self._d_angle_x = 2.0 * math.pi / width
        self._d_angle_y = math.pi / height
        self._aspect_ratio = width / height
        self._update_matrices()

    def on_mouse_drag(self, x, y, dx, dy, btn):
        if btn == MouseButton.Left:
            pos = Vec4(self._pos, 1.0)
            pivot = Vec4(self._look_at, 1.0)
            angle_x = dx * self._d_angle_x
            angle_y = dy * self._d_angle_y
            cos_angle = self.forward.dot(self.up)
            # When the camera direction is the same as the up vector
            if cos_angle > 0.99:
                self._up = Vec3(Mat4.from_axis_angle(self.right, 45, True) * Vec4(self._up, 1.0))
            elif cos_angle < -0.99:
                self._up = Vec3(Mat4.from_axis_angle(-self.right, 45, True) * Vec4(self._up, 1.0))

            # Rotate the camera around the pivot point.
            rot_x = Mat4.from_axis_angle(self.up, angle_x)
            pos = rot_x * (pos - pivot) + pivot
            rot_y = Mat4.from_axis_angle(self.right, angle_y)
            pos = rot_y * (pos - pivot) + pivot

            new_cos_angle = Vec3(pos).normalise().dot(self.up)
            if not(self.safe_rotations and (abs(new_cos_angle) > 0.99 or new_cos_angle < 0.01)):
                self.update_view(pos, self.look_at, self.up)

    def on_mouse_scroll(self, x, y, x_offset, y_offset, min_dist=1e-5):
        if self._zoom_enabled:
            look_dir = (self.look_at - self.position).normalise()
            pos = self.position + look_dir * y_offset * 0.25
            if pos.norm > min_dist:
                self.update_view(pos, self.look_at, self.up)
