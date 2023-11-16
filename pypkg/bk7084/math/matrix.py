import math
import numbers

import numpy as np

from .quaternion import Quat
from .vector import Vec2, Vec3, Vec4, cross


class Matrix(np.ndarray):
    """Base class for Mat3 and Mat4. Data is stored in column major.

    Numpy stores the matrix by default in row-major order.
    """
    _shape = None

    def __new__(cls, *args, **kwargs):
        dtype = kwargs.get('dtype', np.float32)

        if len(args) == 0:
            return np.zeros(cls._shape, dtype=dtype).view(dtype)

        elif len(args) == 1:
            if isinstance(args[0], (list, tuple, np.ndarray)):
                obj = np.array(args[0], dtype=dtype)
                obj = obj.reshape(cls._shape)
            elif isinstance(args[0], numbers.Number):
                obj = np.zeros(cls._shape, dtype=dtype)
                np.fill_diagonal(obj, args[0])
            else:
                raise ValueError("Non number objects can't be matrix's elements")

            if obj.ndim > 2:
                raise ValueError("Matrix must be 2-dimensional")

            return obj.view(cls)

        else:
            obj = np.array([*args], dtype=dtype)

            return obj.view(cls)

    def __array_finalize__(self, obj):
        if self._shape != obj.shape:
            raise ValueError(f"{type(self).__name__} only accepts 1-D array of size {self._shape[0] * self._shape[1]}"
                             f" or 2-D array of size {self._shape[1]}x{self._shape[1]}")

    def __add__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(f"Cannot add a '{self.__class__.__name__}' with '{type(other).__name__}'")
        else:
            return np.add(self, other)

    def __sub__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(f"Cannot subtract a '{self.__class__.__name__}' with '{type(other).__name__}'")
        else:
            return np.subtract(self, other)

    def __iadd__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(f"Cannot add a '{self.__class__.__name__}' with '{type(other).__name__}'")
        else:
            self[:] = np.add(self, other)
            return self

    def __isub__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(f"Cannot add a '{self.__class__.__name__}' with '{type(other).__name__}'")
        else:
            self[:] = np.subtract(self, other)
            return self

    def __mul__(self, other):
        if isinstance(other, numbers.Number):
            return np.multiply(self, other).view(type(self))
        elif isinstance(other, np.ndarray) and self.shape[1] == other.shape[0]:
            result = np.matmul(self, other)
            if isinstance(other, Vec2):
                return result.view(Vec2)
            elif isinstance(other, Vec3):
                return result.view(Vec3)
            elif isinstance(other, Vec4):
                return result.view(Vec4)
            elif isinstance(other, Matrix):
                return result.view(type(self))
            else:
                return result.view(np.ndarray)
        else:
            raise TypeError(f"Cannot multiply a '{self.__class__.__name__}' '{type(other).__name__}'")

    def __rmul__(self, other):
        if isinstance(other, numbers.Number):
            print('yes')
            return np.multiply(other, self).view(type(self))
        elif isinstance(other, np.ndarray) and other.shape[1] == self.shape[0]:
            return np.matmul(other, self)
        else:
            raise TypeError(f"Cannot multiply a '{type(other).__name__}' by '{self.__class__.__name__}'")

    def __imul__(self, other):
        self[:] = self.__mul__(other)
        return self

    def __idiv__(self, other):
        if not isinstance(other, numbers.Number):
            raise TypeError(f"Cannot divide a '{self.__class__.__name__}' by '{type(other).__name__}'")
        else:
            self[:] = np.divide(self, other)
            return self

    def __eq__(self, other):
        return np.asarray(self) == np.asarray(other)

    def col(self, i):
        """Returns the matrix column for the given index `i`."""
        return self[:, i]

    def row(self, i):
        """Returns the matrix row for the given index `i`."""
        return self[i, :]

    @property
    def determinant(self):
        """Returns the determinate of the matrix."""
        return 0

    @property
    def inverse(self):
        """Returns the (multiplicative) inverse of invertible `self`."""
        return np.linalg.inv(self)

    @property
    def transpose(self):
        """Returns the transpose of the matrix.
        """
        return super(Matrix, self).transpose(self)

    @property
    def conjugate_transpose(self):
        return self.transpose.conjugate()

    @classmethod
    def identity(cls, dtype=np.float32):
        return np.identity(cls._shape[0], dtype=dtype).view(cls)

    @classmethod
    def from_mat2(cls, mat: np.ndarray, dtype=None):
        """Creates an affine transformation matrix from the given 2x2 matrix."""
        if mat.shape != (2, 2):
            raise ValueError("Input matrix doesn't have shape of 2 x 2.")
        intype = dtype
        if dtype is None:
            intype = np.float32 if mat.dtype is None else mat.dtype
        m = cls.identity(intype)
        m[:2, :2] = mat[:, :]
        return m.view(cls)

    @classmethod
    def from_mat3(cls, mat: np.ndarray, dtype=None):
        """Creates a 2x2 matrix from a 3x3 matrix, discarding the last row and column."""
        if mat.shape != (3, 3):
            raise ValueError("Input matrix doesn't have shape of 3 x 3.")
        intype = dtype
        if dtype is None:
            intype = np.float32 if mat.dtype is None else mat.dtype
        m = cls.identity(intype)
        if cls._shape[0] <= 3:
            m[:, :] = mat[:cls._shape[0], :cls._shape[1]]
        else:
            m[:3, :3] = mat[:, :]
        return m.view(cls)

    @classmethod
    def from_mat4(cls, mat: np.ndarray, dtype=None):
        """Creates an affine transformation matrix from the given 4x4 matrix."""
        if mat.shape != (4, 4):
            raise ValueError("Input matrix doesn't have shape of 4 x 4.")
        intype = dtype
        if dtype is None:
            intype = np.float32 if mat.dtype is None else mat.dtype
        m = cls.identity(intype)
        m[:, :] = m[:cls._shape[0], :cls._shape[1]]
        return m.view(cls)

    @classmethod
    def from_diagonal(cls, diagonal: np.ndarray, dtype=np.float32):
        """Creates a matrix from a vector."""
        m = np.zeros(cls._shape, dtype=dtype)
        np.fill_diagonal(m, diagonal)
        return m.view(cls)

    def apply(self, vec):
        """Apply the rotation onto a vector."""
        pass


class Mat2(Matrix):
    _shape = (2, 2)

    def __new__(cls, *args, **kwargs):
        return super(Mat2, cls).__new__(cls, *args, **kwargs)

    def row(self, i):
        return Vec2(self[i, :])

    def col(self, i):
        return Vec2(self[:, i])

    @classmethod
    def from_rows(cls, x_axis, y_axis):
        """Creates a 2x2 matrix from two row vectors."""
        return cls([x_axis, y_axis])

    @classmethod
    def from_cols(cls, x_axis, y_axis):
        """Creates a 2x2 matrix from two column vectors."""
        return cls([[x_axis[0], y_axis[0]], [x_axis[1], y_axis[1]]])

    @classmethod
    def from_rotation(cls, angle: np.float32, dtype=np.float32):
        """Creates a 2x2 matrix containing a rotation of `angle` (in radians)."""
        cos = math.cos(angle)
        sin = math.sin(angle)
        return cls([[cos, -sin],
                    [sin, cos]], dtype=dtype)

    def apply(self, vec):
        if not isinstance(vec, Vec2):
            raise ValueError('Mat2 can only be applied to 2-D vector.')
        return (self * vec).view(Vec2)


class Mat3(Matrix):
    _shape = (3, 3)

    def __new__(cls, *args, **kwargs):
        return super(Mat3, cls).__new__(cls, *args, **kwargs)

    def row(self, i):
        return Vec3(self[i, :])

    def col(self, i):
        return Vec3(self[:, i])

    @classmethod
    def from_cols(cls, x_axis: Vec3, y_axis: Vec3, z_axis: Vec3, dtype=np.float32):
        """Creates a 3x3 matrix from three column vectors."""
        return cls([[x_axis[0], y_axis[0], z_axis[0]],
                    [x_axis[1], y_axis[1], z_axis[1]],
                    [x_axis[2], y_axis[2], z_axis[2]]], dtype=dtype)

    @classmethod
    def from_rows(cls, x_axis: Vec3, y_axis: Vec3, z_axis: Vec3, dtype=np.float32):
        """Creates a 3x3 matrix from three row vectors."""
        return cls([x_axis, y_axis, z_axis], dtype=dtype)

    @classmethod
    def from_euler_angles(cls, seq, angles, degrees=False, dtype=np.float32):
        """Creates a Matrix from the specified Euler angles (more precisely, Tait-Bryan angles, in radians).

        Rotations in 3-D can be represented by a sequence of 3 rotations around sequence of axes.

        Args:
            seq (str):
                Specifies sequence of axes for rotations. Up to 3 characters belonging to the set {'x', 'y', 'z'}.

            angles (np.float32 or array_like, shape (N,) or list):
                Euler angles specified in radians (`degrees` is False) or in degrees (`degrees` is True)
                For a single character in `seq`, `angles` can be:
                  - a single value
                For 2- and 3-character wide `seq`, `angles` can be:
                  - array_like with shape (N,) where `N` is the width of `seq`
                  - list of values with minimal size of `N`, where `N` is the width of `seq`

            degrees (bool):
                If True, then the given angles are assumed to be in degrees. Default is False.

            dtype (dtype of numpy):
                Specifies that element type inside of the created matrix.

        Returns:
            Mat3

        Examples:
            Initialize a single rotation matrix along a single axis:

            >>> m = Mat3.from_euler_angles('x', 90, degrees=True)

            Initialize a single rotation matrix with a give axis sequence:

            >>> m = Mat3.from_euler_angles('yzx', [30, 60, 90], degrees=True)
        """
        if seq is None or len(seq) == 0:
            raise ValueError('Not a valid sequence.')
        angles = list([angles]) if not isinstance(angles, (tuple, list, np.ndarray)) else angles
        if len(angles) != len(seq):
            raise ValueError('Provided values don\'t match the length of rotation sequence.')
        if seq.count('x') > 1 or seq.count('y') > 1 or seq.count('z') > 1:
            raise ValueError('Rotation axis can only appear one time in sequence.')
        m = cls.identity(dtype)
        for axis, angle in zip(seq, angles):
            if axis == 'x':
                m = cls.from_rotation_x(angle, degrees, dtype) * m
            if axis == 'y':
                m = cls.from_rotation_y(angle, degrees, dtype) * m
            if axis == 'z':
                m = cls.from_rotation_z(angle, degrees, dtype) * m
        return m

    @classmethod
    def from_axis_angle(cls, axis: Vec3, angle, degrees=False, dtype=np.float32):
        """Creates a 3D rotation matrix from a normalized rotation `axis` and `angle` (in radians)."""
        if axis == Vec3.unit_x(dtype):
            return cls.from_rotation_x(angle, degrees, dtype)
        elif axis == Vec3.unit_y(dtype):
            return cls.from_rotation_y(angle, degrees, dtype)
        elif axis == Vec3.unit_z(dtype):
            return cls.from_rotation_z(angle, degrees, dtype)
        else:
            cos = math.cos(angle)
            sin = math.sin(angle)
            a = 1. - cos
            ux, uy, uz = axis.x, axis.y, axis.z
            uxx, uxy, uxz = ux * ux, ux * uy, ux * uz
            uyz, uyy, uzz = uy * uz, uy * uy, uz * uz
            return cls([[cos + uxx * a, uxy * a - uz * sin, uxz * a + uy * sin],
                        [uxy * a + uz * sin, cos + uyy * a, uyz * a - ux * sin],
                        [uxz * a - uy * sin, uyz * a + ux * sin, cos + uzz * a]], dtype=dtype)

    @classmethod
    def from_rotation_x(cls, angle, degrees=False, dtype=np.float32):
        """Creates a 3D rotation matrix from `angle` (in radians) around the x axis."""
        angle = angle if not degrees else math.radians(angle)
        cos = math.cos(angle)
        sin = math.sin(angle)
        return cls([[1., 0., 0.],
                    [0., cos, -sin],
                    [0., sin, cos]], dtype=dtype)

    @classmethod
    def from_rotation_y(cls, angle, degrees=False, dtype=np.float32):
        """Creates a 3D rotation matrix from `angle` (in radians) around the y axis."""
        angle = angle if not degrees else math.radians(angle)
        cos = math.cos(angle)
        sin = math.sin(angle)
        return cls([[cos, 0., sin],
                    [0., 1., 0.],
                    [-sin, 0., cos]], dtype=dtype)

    @classmethod
    def from_rotation_z(cls, angle, degrees=False, dtype=np.float32):
        """Creates a 3D rotation matrix from `angle` (in radians) around the z axis."""
        angle = angle if not degrees else math.radians(angle)
        cos = math.cos(angle)
        sin = math.sin(angle)
        return cls([[cos, -sin, 0.],
                    [sin, cos, 0.],
                    [0., 0., 1.]], dtype=dtype)

    @classmethod
    def from_scale(cls, scale: Vec3, dtype=np.float32):
        """Creates an affine transformation matrix from the given non-uniform 3D `scale`.
        The resulting matrix can be used to transform 3D points and vectors."""
        return cls.from_diagonal(scale, dtype)

    @classmethod
    def from_translation(cls, translation: Vec2, dtype=np.float32):
        """Creates an affine transformation matrix from the given 2D `translation`.
        The resulting matrix can be used to transform 2D points and vectors."""
        m = cls.identity(dtype=dtype)
        m[:2, 2] = translation
        return m

    @classmethod
    def from_mat4(cls, mat: np.ndarray, dtype=None):
        """Creates an affine transformation matrix from the given 4x4 matrix.
        The resulting matrix can be used to transform 2D points and vectors."""
        assert (mat.shape == (4, 4))
        intype = dtype
        if dtype is None:
            intype = np.float32 if mat.dtype is None else mat.dtype
        m = np.zeros(cls._shape, dtype=intype)
        m[:3, :3] = mat[:3, :3]
        return m

    @classmethod
    def from_quat(cls, quat: Quat, dtype=np.float32):
        """Creates an affine transformation matrix from the given quaternion (unit, normalised)."""
        xx, yy, zz = quat.x * quat.x, quat.y * quat.y, quat.z * quat.z
        xy, xz, xw = quat.x * quat.y, quat.x * quat.z, quat.x * quat.w
        yz, yw, zw = quat.y * quat.z, quat.y * quat.w, quat.z * quat.w
        return cls([[1. - 2. * yy - 2. * zz, 2. * xy - 2. * zw, 2. * xz + 2. * yw],
                    [2. * xy + 2. * zw, 1. - 2. * xx - 2. * zz, 2. * yz - 2. * xw],
                    [2. * xz - 2. * yw, 2. * yz + 2. * xw, 1. - 2. * xx - 2. * yy]], dtype=dtype)

    def apply(self, vec):
        if not isinstance(vec, Vec3):
            raise ValueError('Mat3 can only be applied to 3-D vector.')
        return (self * vec).view(Vec3)


class Mat4(Matrix):
    _shape = (4, 4)

    def __new__(cls, *args, **kwargs):
        return super(Mat4, cls).__new__(cls, *args, **kwargs)

    def row(self, i):
        return Vec4(self[i, :])

    def col(self, i):
        return Vec4(self[:, i])

    @classmethod
    def from_rows(cls, x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4):
        """Creates a 4x4 matrix from four column vectors."""
        return cls([x_axis, y_axis, z_axis, w_axis])

    @classmethod
    def from_cols(cls, x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4):
        """Creates a 3x3 matrix from three column vectors."""
        return cls([[x_axis[0], y_axis[0], z_axis[0], w_axis[0]],
                    [x_axis[1], y_axis[1], z_axis[1], w_axis[1]],
                    [x_axis[2], y_axis[2], z_axis[2], w_axis[2]],
                    [x_axis[3], y_axis[3], z_axis[3], w_axis[3]]])

    @classmethod
    def from_euler_angles(cls, seq, angles, degrees=False, dtype=np.float32):
        """Creates a Matrix from the specified Euler angles (more precisely, Tait-Bryan angles, in radians).

        Rotations in 3-D can be represented by a sequence of 3 rotations around sequence of axes.

        Args:
            seq (str):
                Specifies sequence of axes for rotations. Up to 3 characters belonging to the set {'x', 'y', 'z'}.

            angles (np.float32 or array_like, shape (N,) or list):
                Euler angles specified in radians (`degrees` is False) or in degrees (`degrees` is True)
                For a single character in `seq`, `angles` can be:
                    - a single value
                For 2- and 3-character wide `seq`, `angles` can be:
                    - array_like with shape (N,) where `N` is the width of `seq`
                    - list of values with minimal size of `N`, where `N` is the width of `seq`

            degrees (bool):
                If True, then the given angles are assumed to be in degrees. Default is False.

            dtype (dtype of numpy):
                Specifies that element type inside of the created matrix.

            Returns:
                Mat3

            Examples:
                Initialize a single rotation matrix along a single axis:

                >>> m = Mat3.from_euler_angles('x', 90, degrees=True)

                Initialize a single rotation matrix with a give axis sequence:

                >>> m = Mat3.from_euler_angles('yzx', [30, 60, 90], degrees=True)
        """
        return cls.from_mat3(Mat3.from_euler_angles(seq, angles, degrees, dtype), dtype).view(cls)

    @classmethod
    def from_axis_angle(cls, axis: Vec3, angle, degrees=False, dtype=np.float32):
        """Creates an affine transformation matrix containing a rotation around a normalized
        rotation `axis` and `angle` (in radians)."""
        return cls.from_mat3(Mat3.from_axis_angle(axis, angle, degrees, dtype), dtype)

    @classmethod
    def from_rotation_x(cls, angle, degrees=False, dtype=np.float32):
        """Creates a 3D rotation matrix from `angle` (in radians) around the x axis."""
        return cls.from_mat3(Mat3.from_rotation_x(angle, degrees, dtype), dtype)

    @classmethod
    def from_rotation_y(cls, angle, degrees=False, dtype=np.float32):
        """Creates a 3D rotation matrix from `angle` (in radians) around the y axis."""
        return cls.from_mat3(Mat3.from_rotation_y(angle, degrees, dtype), dtype)

    @classmethod
    def from_rotation_z(cls, angle, degrees=False, dtype=np.float32):
        """Creates a 3D rotation matrix from `angle` (in radians) around the z axis."""
        return cls.from_mat3(Mat3.from_rotation_z(angle, degrees, dtype), dtype)

    @classmethod
    def from_scale(cls, scale: Vec3, dtype=np.float32):
        """Creates an affine transformation matrix from the given non-uniform 2D `scale`.
        The resulting matrix can be used to transform 2D points and vectors."""
        return cls.from_diagonal(Vec4.from_vec3(scale, 1.), dtype=dtype)

    @classmethod
    def from_translation(cls, translation: Vec3, dtype=np.float32):
        """Creates an affine transformation matrix from the given 3D `translation`."""
        m = cls.identity(dtype=dtype)
        m[:3, 3] = np.asarray(translation).reshape(3, )
        return m

    @classmethod
    def from_quat(cls, quat, dtype=np.float32):
        """Creates an affine transformation matrix from the given quaternion (normalised)."""
        return cls.from_mat3(Mat3.from_quat(quat, dtype=dtype))

    @classmethod
    def look_at_gl(cls, eye: Vec3, center: Vec3, up: Vec3, dtype=np.float32):
        """Creates a right-handed view matrix using a camera position, and up direction, and a focal point.
        This is the same as the OpenGL `gluLookAt` function. See
        https://www.khronos.org/registry/OpenGL-Refpages/gl2.1/xhtml/gluLookAt.xml
        """
        return cls.look_at_rh_y_up(eye, center, up, dtype)

    @classmethod
    def look_at_rh_y_up(cls, eye: Vec3, center: Vec3, up: Vec3, dtype=np.float32):
        forward = (center - eye).normalised
        up = up.normalised
        u = cross(forward, up).normalised
        v = cross(u, forward).normalised
        w = -forward
        return Mat4.from_rows(Vec4(u, -u.dot(eye)),
                              Vec4(v, -v.dot(eye)),
                              Vec4(w, -w.dot(eye)),
                              Vec4(0., 0., 0., 1.))

    @classmethod
    def perspective_gl(cls, fov_y, aspect_ratio, z_near, z_far, degrees=False, dtype=np.float32):
        """Creates a right-handed perspective projection matrix with [-1, 1] depth range for a symmetric
        perspective-view frustum.

        The source coordinate space is right-handed and y-up, the destination space is left-handed
        and y-up, with Z(depth) clip extending from -1.0 (close) to 1.0 (far).

        This is the same as the OpenGL `gluPerspective` function. See
        https://www.khronos.org/registry/OpenGL-Refpages/gl2.1/xhtml/gluPerspective.xml

        Args:
            fov_y (np.float32):
                Specifies the field of view angle in the y-direction, in radians if `degrees` not set to True.

            aspect_ratio (np.float32):
                Specifies the aspect ratio that determines the field of view in the x-direction. The aspect ratio is the
                ratio of x (width) to y (height).

            z_near (np.float32):
                Specifies the near distance from the viewer to the near clipping plane (not in the world frame,
                always positive).

            z_far (np.float32):
                Specifies the far distance from the viewer to the far clipping plane (not in the world frame,
                always positive).

            degrees (bool):
                If True, then the given field of view angle is assumed to be in degrees. Default is False.

            dtype (dtype of numpy):
                Specifies that element type inside of the created matrix.

        Returns:
            Mat4
        """
        fov = math.radians(fov_y) if degrees else fov_y
        n, f = z_near, z_far
        tan = math.tan(fov / 2.)
        return cls([[1. / (tan * aspect_ratio), 0., 0., 0.],
                    [0., 1. / tan, 0., 0.],
                    [0., 0., -(f + n) / (f - n), -2. * f * n / (f - n)],
                    [0., 0., -1., 0.]], dtype=dtype)

    @classmethod
    def orthographic_gl(cls, left, right, bottom, top, near, far, dtype=np.float32):
        """Creates a right-handed orthographic projection matrix with [-1, 1] depth range.

        The source coordinate space is right-handed and y-up, the destination space is left-handed
        and y-up, with Z(depth) clip extending from -1.0 (close) to 1.0 (far).

        This is the same as the OpenGL `glOrtho` function. See
        https://www.khronos.org/registry/OpenGL-Refpages/gl2.1/xhtml/glOrtho.xml"""
        _a = 1. / (right - left)
        _b = 1. / (top - bottom)
        _c = 1. / (far - near)
        return cls([[2. * _a, 0., 0., -(right + left) * _a],
                    [0., 2. * _b, 0., -(top + bottom) * _b],
                    [0., 0., -2. * _c, -(far + near) * _c],
                    [0., 0., 0., 1.]], dtype=dtype)

    def apply(self, vec):
        if not isinstance(vec, Vec4):
            raise ValueError('Mat4 can only be applied to 4-D vector.')
        return (self * vec).view(Vec4)
