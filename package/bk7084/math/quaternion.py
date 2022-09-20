from __future__ import annotations

import math
import numbers

import numpy as np

from .vector import Vec3

"""
TODO:
  - power of a quaternion
  - dot product
  - cross product
  - create from rotation matrix
  - into rotation matrix
  - lerp
  - slerp
  - power
  - log
  - test
"""


class Quat(np.ndarray):
    _shape = (4,)

    def __new__(cls, x, y, z, w=1., dtype=float):
        return np.array([w, x, y, z]).astype(dtype).view(cls)

    def __array_finalize__(self, obj):
        pass

    def __str__(self):
        _str = f'Quat [' + '{:>11.8f},' * 3 + ' {:>11.8f}]'
        return _str.format(*self.flatten())

    def __repr__(self):
        return super(Quat, self).__repr__()

    def __len__(self):
        return 4

    def __neg__(self):
        return np.multiply(self, -1)

    def __add__(self, other):
        if not isinstance(other, Quat):
            raise ValueError('Quaternion can only add with another Quaternion')
        else:
            return np.add(self, other)

    def __sub__(self, other):
        if not isinstance(other, Quat):
            raise ValueError('Quaternion can only be subtracted by another Quaternion')
        else:
            return np.subtract(self, other)

    def __mul__(self, other):
        """Hamilton product of two quaternions.
        (w0, v0) * (w1, v1) = (w0w1 - v0*v1, w0v1 + w1v0 + v0 x v1)
        """
        if isinstance(other, numbers.Number):
            return np.multiply(self, other)
        elif isinstance(other, Quat):
            if self.is_real and other.is_real:
                return Quat(self[0] * other[0], 0., 0., 0.)
            elif self.is_pure and other.is_pure:
                v0 = self[1:]
                v1 = other[1:]
                _dot = np.dot(v0, v1)
                _cross = np.cross(v0, v1)
                return Quat(_cross[0], _cross[1], _cross[2], -_dot)
            else:
                w0, v0 = self[0], self[1:]
                w1, v1 = other[0], other[1:]
                val = w0 * v1 + w1 * v0 + np.cross(v0, v1)
                return Quat(val[0], val[1], val[2], self.w * other.w - np.dot(v0, v1))

    def __rmul__(self, other):
        return self.__mul__(other)

    def __div__(self, other):
        if isinstance(other, numbers.Number) and other != 0:
            return np.divide(self, other)
        else:
            raise ValueError('Quaternion can only be divided by a non-zero number')

    def __iadd__(self, other):
        self[:] = self.__add__(other)
        return self

    def __isub__(self, other):
        self[:] = self.__sub__(other)
        return self

    def __imul__(self, other):
        self[:] = self.__mul__(other)
        return self

    def __idiv__(self, other):
        self[:] = self.__div__(other)
        return self

    @classmethod
    def identity(cls) -> Quat:
        return Quat(0., 0., 0., 1., float)

    @classmethod
    def from_rotation_x(cls, angle, degrees=False, dtype=float):
        angle = math.radians(angle) if degrees else angle
        half = angle * 0.5
        return Quat(math.sin(half), 0., 0., math.cos(half), dtype=dtype)

    @classmethod
    def from_rotation_y(cls, angle, degrees=False, dtype=float):
        angle = math.radians(angle) if degrees else angle
        half = angle * 0.5
        return Quat(0., math.sin(half), 0., math.cos(half), dtype=dtype)

    @classmethod
    def from_rotation_z(cls, angle, degrees=False, dtype=float):
        angle = math.radians(angle) if degrees else angle
        half = angle * 0.5
        return Quat(0., 0., math.sin(half), math.cos(half), dtype=dtype)

    @classmethod
    def from_euler_angles(cls, angle, degrees=False, dtype=float):
        pass

    @classmethod
    def from_axis_angle(cls, axis, angle, degrees=False, dtype=float):
        dtype = axis.dtype or dtype
        angle = math.radians(angle) if degrees else angle
        if isinstance(axis, Vec3):
            axis.normalise()
        elif isinstance(axis, np.ndarray) and axis.shape == (3,):
            if not np.isclose(np.linalg.norm(axis), 1.):
                axis.view(Vec3).normalise()
        elif isinstance(axis, (list, tuple)) and len(axis) == 3:
            axis = np.asarray(axis)
            if not np.isclose(np.linalg.norm(axis), 1.):
                axis.view(Vec3).normalise()
        else:
            raise ValueError('Axis is not in array like form')

        half = angle * 0.5
        v = axis * math.sin(half)
        return Quat(v.x, v.y, v.z, math.cos(half))

    @classmethod
    def from_matrix(cls, mat, dtype=float):
        pass

    @property
    def conjugate(self):
        return Quat(-self.x, -self.y, -self.z, self.w)

    @property
    def norm(self):
        """The norm of the quaternion equals to the square root of the product of a quaternion with its
         conjugate is called its norm."""
        return math.sqrt(np.sum(self ** 2))

    @property
    def inverse(self):
        if self.is_nonzero:
            return self.conjugate / np.sum(self ** 2)
        else:
            raise ZeroDivisionError("Zero quaternion doesn't have inverse")

    @property
    def normalised(self):
        """Returns this quaternion with a magnitude of 1"""
        if not self.is_nonzero:
            return self
        else:
            return self[:] / self.norm

    def normalise(self):
        self[:] = self.normalised()
        return self

    @property
    def is_nonzero(self) -> bool:
        return self.any()

    @property
    def is_real(self) -> bool:
        """A Real quaternion is a quaternion with a vector term of 0."""
        return not self[1:].any()

    @property
    def is_pure(self) -> bool:
        """A Pure quaternion has a zero scalar term."""
        return self.w == 0

    @property
    def is_unit(self) -> bool:
        """A unit quaternion has a zero scalar and a unit vector."""
        return self.is_pure and np.sum(self[1:] ** 2) == 1

    @property
    def w(self):
        return self[0]

    @w.setter
    def w(self, value):
        self[0] = value

    @property
    def x(self):
        return self[1]

    @x.setter
    def x(self, value):
        self[1] = value

    @property
    def y(self):
        return self[2]

    @y.setter
    def y(self, value):
        self[2] = value

    @property
    def z(self):
        return self[3]

    @z.setter
    def z(self, value):
        self[3] = value

    @property
    def rotation_angle(self):
        """Calculates the rotation angle around the quaternion's axis.
        Returns:
            float, rotation angle in radians
        """
        return math.acos(self.w) * 2.0

    @property
    def rotation_axis(self):
        sin = math.sin(math.acos(self.w))
        return Vec3(self[1:]) / sin

    def lerp(self, other, t):
        """Interpolates between two quaternions by `t`. `t` is clamped to the range [0, 1]"""
        return NotImplementedError

    def slerp(self, other, t):
        """Spherically interpolates between two quaternions by `t`. `t` is clamped to the range [0, 1]"""
        return NotImplementedError

    def squad(self):
        """Spherical and Quadrangle, it smoothly interpolates over a path of rotations."""
        pass

    def dot(self, other):
        return NotImplementedError

    def cross(self, other):
        """Returns the cross-product of the two quaternions.
        Quaternions are NOT communicative. Therefore, order is important.

        Note:
            This is NOT the same as a vector cross-product. Quaternion cross-product is the equivalent
            of matrix multiplication.
        """
        return NotImplementedError

    def distance(self, other: Quat):
        """Computes the angular difference between two quaternions."""
        return self.inverse * other

    def exp(self):
        """Computes the exponential of the quaternion"""
        e = math.exp(self.w)
        vec_norm = np.linalg.norm(self[1:])
        if np.isclose(vec_norm, 0):
            return Quat(0, 0, 0, e, dtype=self.dtype)

        sin = np.sin(vec_norm) / vec_norm
        return Quat(self.x * sin, self.y * sin, self.z * sin, np.cos(vec_norm), dtype=self.dtype) * e

    def log(self):
        pass

    def to_mat3(self):
        return NotImplementedError

    def to_mat4(self):
        return NotImplementedError


class DualQuat(np.ndarray):
    _shape = (8,)

    def __new__(cls, real, dual):
        pass
