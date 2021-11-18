from __future__ import annotations

import abc
import math
import numbers

import numpy as np

__all__ = ['Vec2', 'Vec3', 'Vec4', 'cross', 'dot', 'interpolate', 'Mat2', 'Mat3', 'Mat4']

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


class Vector(np.ndarray):
    """Base class for Vec2, Vec3 and Vec4
    Examples:
        >>> v0 = Vec3()  # all zeros
        >>> v1 = Vec3(1.0)  # filled with 1.0
        >>> v2 = Vec3(1.0, 2.0, 3.0) # x, y, z
        >>> v3 = Vec3([1.0, 2.0, 3.0])
        >>> v4 = Vec3((1.0, 2.0, 3.0))
        >>> v5 = Vec3(np.array([1.0, 2.0, 3.0]))
    """
    _shape = None

    def __new__(cls, *args, **kwargs):
        dtype = kwargs.get('dtype', float)

        if cls._shape is not None:
            arg_count = len(args)
            obj = np.zeros((cls._shape[0],), dtype=dtype)

            if arg_count == 1 and isinstance(args[0], numbers.Number):
                obj.fill(args[0])
            else:
                i = 0
                for arg in args:
                    if i >= cls._shape[0]:
                        break
                    if isinstance(arg, (list, tuple, np.ndarray)):
                        arr = np.asarray(arg).flatten()
                        for e in arr:
                            if i >= cls._shape[0]:
                                break
                            else:
                                obj[i] = e
                                i += 1
                    elif isinstance(arg, numbers.Number):
                        obj[i] = arg
                        i += 1
                    else:
                        raise ValueError('Vector only accepts numeric values.')

            return obj.reshape(cls._shape).view(cls)
        else:
            arg_count = len(args)
            data = np.array([])
            for arg in args:
                if isinstance(arg, (list, tuple, np.ndarray)):
                    arr = np.asarray(arg).flatten()
                    data = np.concatenate([data, arr])
                elif isinstance(arg, numbers.Number):
                    data = np.append(data, arg)
                else:
                    raise ValueError('Vector only accepts numeric values.')
            cls._shape = (len(data), 1)
            return data.reshape(cls._shape).view(cls)

    def __array_finalize__(self, obj):
        if self._shape != obj.shape:
            obj.reshape(self._shape)

    def __add__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(f"Cannot add a '{self.__class__.__name__}' with '{type(other).__name__}'")
        else:
            return np.add(self, other)

    def __iadd__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(f"Cannot add a '{self.__class__.__name__}' with '{type(other).__name__}'")
        else:
            self[:] = np.add(self, other)
            return self

    def __sub__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(f"Cannot subtract a '{self.__class__.__name__}' with '{type(other).__name__}'")
        else:
            return np.subtract(self, other)

    def __isub__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(f"Cannot subtract a '{self.__class__.__name__}' with '{type(other).__name__}'")
        else:
            self[:] = np.subtract(self, other)
            return self

    def __div__(self, other):
        if not isinstance(other, numbers.Number):
            raise TypeError(f"Cannot divide a '{self.__class__.__name__}' by '{type(other).__name__}'")
        else:
            return np.divide(self, other)

    def __idiv__(self, other):
        if not isinstance(other, numbers.Number):
            raise TypeError(f"Cannot divide a '{self.__class__.__name__}' by '{type(other).__name__}'")
        else:
            self[:] = np.divide(self, other)
            return self

    def __mul__(self, other):
        if isinstance(other, numbers.Number):
            return np.multiply(self, other)
        elif isinstance(other, np.ndarray) and self.shape[1] == other.shape[0]:
            return np.matmul(self, other)
        else:
            raise TypeError(f"Cannot multiply a '{self.__class__.__name__}' with {type(other).__name__}")

    def __rmul__(self, other):
        if isinstance(other, numbers.Number):
            return np.multiply(other, self)
        elif isinstance(other, np.ndarray) and other.shape[1] == self.shape[0]:
            return np.matmul(other, self)
        else:
            raise TypeError(f"Cannot multiply a '{self.__class__.__name__}' with '{type(other).__name__}'")

    def __imul__(self, other):
        self[:] = self.__mul__(other)
        return self

    def __getitem__(self, i):
        return super(Vector, self).__getitem__((i, 0))

    def __setitem__(self, i, value):
        if 0 <= i < self._shape[0]:
            super(Vector, self).__setitem__((i, 0), value)
        else:
            raise IndexError('Index out of range.')

    def __eq__(self, other):
        return np.all(np.isclose(np.asarray(self), np.asarray(other)))

    def __str__(self):
        _str = 'Vec{} [' + '{:>11.8f},' * (self._shape[0] - 1) + ' {:>11.8f}]'
        return _str.format(self._shape[0], *self.view(np.ndarray).flatten())

    def __repr__(self):
        return self.__str__()

    @property
    def dims(self):
        return self._shape[0]


class Vec2(Vector):
    _shape = (2, 1)

    def __new__(cls, *args, **kwargs):
        return super(Vec2, cls).__new__(cls, *args, **kwargs)

    @classmethod
    def from_vec3(cls, vec, dtype=None):
        """Create a Vector2 from a Vector3.
        """
        dtype = dtype if dtype else vec.dtype
        return cls(np.asarray(vec[:2]), dtype=dtype)

    @classmethod
    def from_vec4(cls, vec, dtype=None):
        """Create a Vector2 from a Vector4.
        """
        dtype = dtype if dtype else vec.dtype
        return cls(np.asarray(vec[:2]), dtype=dtype)

    @property
    def x(self):
        return super(Vector, self).__getitem__((0, 0))

    @x.setter
    def x(self, value):
        super(Vector, self).__setitem__((0, 0), value)

    @property
    def y(self):
        return super(Vector, self).__getitem__((1, 0))

    @y.setter
    def y(self, value):
        super(Vector, self).__setitem__((1, 0), value)

    @classmethod
    def unit_x(cls, dtype=float):
        return Vec2(1., 0., dtype=dtype)

    @classmethod
    def unit_y(cls, dtype=float):
        return Vec2(0., 1., dtype=dtype)


class Vec3(Vector):
    _shape = (3, 1)

    def __new__(cls, *args, **kwargs):
        return super(Vec3, cls).__new__(cls, *args, **kwargs)

    @classmethod
    def from_vec2(cls, vec, z=0, dtype=None):
        """Create a Vector3 from a Vector2.
        """
        dtype = dtype if dtype else vec.dtype
        return cls(np.append(np.asarray(vec), z), dtype=dtype)

    @classmethod
    def from_vec4(cls, vec, dtype=None):
        """Create a Vector3 from a Vector4.
        """
        dtype = dtype if dtype else vec.dtype
        return cls(np.asarray(vec[:3]), dtype=dtype)

    @property
    def x(self):
        return super(Vector, self).__getitem__((0, 0))

    @x.setter
    def x(self, value):
        super(Vector, self).__setitem__((0, 0), value)

    @property
    def y(self):
        return super(Vector, self).__getitem__((1, 0))

    @y.setter
    def y(self, value):
        super(Vector, self).__setitem__((1, 0), value)

    @property
    def z(self):
        return super(Vector, self).__getitem__((2, 0))

    @z.setter
    def z(self, value):
        super(Vector, self).__setitem__((2, 0), value)

    @classmethod
    def unit_x(cls, dtype=float):
        return Vec3(1., 0., 0., dtype=dtype)

    @classmethod
    def unit_y(cls, dtype=float):
        return Vec3(0., 1., 0., dtype=dtype)

    @classmethod
    def unit_z(cls, dtype=float):
        return Vec3(0., 0., 1., dtype=dtype)

    @property
    def norm_squared(self):
        return np.sum(self ** 2, axis=-1)

    @property
    def norm(self) -> float:
        return math.sqrt(np.sum(self ** 2))

    @property
    def length_squared(self):
        return np.sum(self ** 2, axis=-1)

    @property
    def length(self):
        return np.sqrt(np.sum(self ** 2))

    def dot(self, other):
        return self.x * other.x + self.y * other.y + self.z * other.z

    def cross(self, other):
        return cross(self, other)

    def normalise(self):
        self[:] = self.normalised()
        return self

    @property
    def normalised(self):
        n = self.norm
        if n == 0. or n == 1.:
            return self
        else:
            return self / n

    @property
    def length_recip(self):
        """Computes 1.0 / length(). For valid results, `self` must not be of length zero."""
        return 1. / self.length

    def distance(self, other):
        """Computes the Euclidean distance between two points in space."""
        return (self - other).length

    def project_onto(self, other):
        """Returns the vector projection of `self` onto `other`."""
        other_normalised = other.normalised
        return self.normalised.dot(other_normalised) * other_normalised * self.norm


class Vec4(Vector):
    _shape = (4, 1)

    def __new__(cls, *args, **kwargs):
        return super(Vec4, cls).__new__(cls, *args, **kwargs)

    @classmethod
    def from_vec2(cls, vec, z=0, w=0, dtype=None):
        """Create a Vector4 from a Vector2.

        Examples:
            >>> va = Vec2(0.2, 0.1)
            >>> vb = Vec4.from_vec2(va)
        """
        dtype = dtype if dtype else vec.dtype
        return cls(np.append(np.asarray(vec), [z, w]), dtype=dtype)

    @classmethod
    def from_vec3(cls, vec, w=0, dtype=None):
        """Create a Vector4 from a Vector2.
        """
        dtype = dtype if dtype else vec.dtype
        return cls(np.append(np.asarray(vec), w), dtype=dtype)

    @property
    def x(self):
        return super(Vector, self).__getitem__((0, 0))

    @x.setter
    def x(self, value):
        super(Vector, self).__setitem__((0, 0), value)

    @property
    def y(self):
        return super(Vector, self).__getitem__((1, 0))

    @y.setter
    def y(self, value):
        super(Vector, self).__setitem__((1, 0), value)

    @property
    def z(self):
        return super(Vector, self).__getitem__((2, 0))

    @z.setter
    def z(self, value):
        super(Vector, self).__setitem__((2, 0), value)

    @property
    def w(self):
        return super(Vector, self).__getitem__((3, 0))

    @w.setter
    def w(self, value):
        super(Vector, self).__setitem__((3, 0), value)

    @classmethod
    def unit_x(cls, dtype=float):
        return Vec4(1., 0., 0., 0., dtype=dtype)

    @classmethod
    def unit_y(cls, dtype=float):
        return Vec4(0., 1., 0., 0., dtype=dtype)

    @classmethod
    def unit_z(cls, dtype=float):
        return Vec4(0., 0., 1., 0., dtype=dtype)

    @classmethod
    def unit_w(cls, dtype=float):
        return Vec4(0., 0., 0., 1., dtype=dtype)


def dot(a: Vec3, b: Vec3):
    """Computes the dot product of two vectors.
    Args:
        a (Vec3): an Nd array with dimension 3.
        b (Vec3):  an Nd array with dimension 3.

    Returns:
        The dot product of two vectors.
    """
    return np.sum(a * b, axis=-1)


def cross(a: Vec3, b: Vec3):
    """Computes the cross product of two vectors.
    Args:
        a (Vec3): an Nd array with dimension 3.
        b (Vec3): an Nd array with dimension 3.

    Returns:
        The cross product of two vectors.
    """
    return Vec3(np.cross(a, b, axis=0).reshape(3))  # cross along the first axis


def interpolate(a: Vec3, b: Vec3, t: float):
    return a + (b - a) * t


class Matrix(np.ndarray):
    """Base class for Mat3 and Mat4. Data is stored in column major.

    Numpy stores the matrix by default in row-major order.
    """
    _shape = None

    def __new__(cls, *args, **kwargs):
        dtype = kwargs.get('dtype', float)

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
            return np.multiply(self, other)
        elif isinstance(other, np.ndarray) and self.shape[1] == other.shape[0]:
            result = self @ other
            if isinstance(other, Vec2):
                return result.view(Vec2)
            elif isinstance(other, Vec3):
                return result.view(Vec3)
            elif isinstance(other, Vec4):
                return result.view(Vec4)
            else:
                return result.view(np.ndarray)
        else:
            raise TypeError(f"Cannot multiply a '{self.__class__.__name__}' '{type(other).__name__}'")

    def __rmul__(self, other):
        if isinstance(other, numbers.Number):
            return np.multiply(other, self)
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
    def identity(cls, dtype=float):
        return np.identity(cls._shape[0], dtype=dtype)

    @classmethod
    def from_mat2(cls, mat: np.ndarray, dtype=None):
        """Creates an affine transformation matrix from the given 2x2 matrix."""
        if mat.shape != (2, 2):
            raise ValueError("Input matrix doesn't have shape of 2 x 2.")
        intype = dtype
        if dtype is None:
            intype = float if mat.dtype is None else mat.dtype
        m = cls.identity(intype)
        m[:2, :2] = mat[:, :]
        return m

    @classmethod
    def from_mat3(cls, mat: np.ndarray, dtype=None):
        """Creates a 2x2 matrix from a 3x3 matrix, discarding the last row and column."""
        if mat.shape != (3, 3):
            raise ValueError("Input matrix doesn't have shape of 3 x 3.")
        intype = dtype
        if dtype is None:
            intype = float if mat.dtype is None else mat.dtype
        m = cls.identity(intype)
        if cls._shape[0] <= 3:
            m[:, :] = mat[:cls._shape[0], :cls._shape[1]]
        else:
            m[:3, :3] = mat[:, :]
        return m

    @classmethod
    def from_mat4(cls, mat: np.ndarray, dtype=None):
        """Creates an affine transformation matrix from the given 4x4 matrix."""
        if mat.shape != (4, 4):
            raise ValueError("Input matrix doesn't have shape of 4 x 4.")
        intype = dtype
        if dtype is None:
            intype = float if mat.dtype is None else mat.dtype
        m = cls.identity(intype)
        m[:, :] = m[:cls._shape[0], :cls._shape[1]]

    @classmethod
    def from_diagonal(cls, diagonal: np.ndarray, dtype=float):
        """Creates a matrix from a vector."""
        m = np.zeros(cls._shape, dtype=dtype)
        np.fill_diagonal(m, diagonal)
        return m

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
    def from_rotation(cls, angle: float, dtype=float):
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
    def from_cols(cls, x_axis: Vec3, y_axis: Vec3, z_axis: Vec3, dtype=float):
        """Creates a 3x3 matrix from three column vectors."""
        return cls([[x_axis[0], y_axis[0], z_axis[0]],
                    [x_axis[1], y_axis[1], z_axis[1]],
                    [x_axis[2], y_axis[2], z_axis[2]]], dtype=dtype)

    @classmethod
    def from_rows(cls, x_axis: Vec3, y_axis: Vec3, z_axis: Vec3, dtype=float):
        """Creates a 3x3 matrix from three row vectors."""
        return cls([x_axis, y_axis, z_axis], dtype=dtype)

    @classmethod
    def from_euler_angles(cls, seq, angles, degrees=False, dtype=float):
        """Creates a Matrix from the specified Euler angles (more precisely, Tait-Bryan angles, in radians).

        Rotations in 3-D can be represented by a sequence of 3 rotations around sequence of axes.

        Args:
            seq (str):
                Specifies sequence of axes for rotations. Up to 3 characters belonging to the set {'x', 'y', 'z'}.

            angles (float or array_like, shape (N,) or list):
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
    def from_axis_angle(cls, axis: Vec3, angle, degrees=False, dtype=float):
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
    def from_rotation_x(cls, angle, degrees=False, dtype=float):
        """Creates a 3D rotation matrix from `angle` (in radians) around the x axis."""
        angle = angle if not degrees else math.radians(angle)
        cos = math.cos(angle)
        sin = math.sin(angle)
        return cls([[1., 0., 0.],
                    [0., cos, -sin],
                    [0., sin, cos]], dtype=dtype)

    @classmethod
    def from_rotation_y(cls, angle, degrees=False, dtype=float):
        """Creates a 3D rotation matrix from `angle` (in radians) around the y axis."""
        angle = angle if not degrees else math.radians(angle)
        cos = math.cos(angle)
        sin = math.sin(angle)
        return cls([[cos, 0., sin],
                    [0., 1., 0.],
                    [-sin, 0., cos]], dtype=dtype)

    @classmethod
    def from_rotation_z(cls, angle, degrees=False, dtype=float):
        """Creates a 3D rotation matrix from `angle` (in radians) around the z axis."""
        angle = angle if not degrees else math.radians(angle)
        cos = math.cos(angle)
        sin = math.sin(angle)
        return cls([[cos, -sin, 0.],
                    [sin, cos, 0.],
                    [0., 0., 1.]], dtype=dtype)

    @classmethod
    def from_scale(cls, scale: Vec3, dtype=float):
        """Creates an affine transformation matrix from the given non-uniform 3D `scale`.
        The resulting matrix can be used to transform 3D points and vectors."""
        return cls.from_diagonal(scale, dtype)

    @classmethod
    def from_translation(cls, translation: Vec2, dtype=float):
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
            intype = float if mat.dtype is None else mat.dtype
        m = np.zeros(cls._shape, dtype=intype)
        m[:3, :3] = mat[:3, :3]
        return m

    @classmethod
    def from_quat(cls, quat: Quat, dtype=float):
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
    def from_euler_angles(cls, seq, angles, degrees=False, dtype=float):
        """Creates a Matrix from the specified Euler angles (more precisely, Tait-Bryan angles, in radians).

        Rotations in 3-D can be represented by a sequence of 3 rotations around sequence of axes.

        Args:
            seq (str):
                Specifies sequence of axes for rotations. Up to 3 characters belonging to the set {'x', 'y', 'z'}.

            angles (float or array_like, shape (N,) or list):
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
        return cls.from_mat3(Mat3.from_euler_angles(seq, angles, degrees, dtype), dtype)

    @classmethod
    def from_axis_angle(cls, axis: Vec3, angle, degrees=False, dtype=float):
        """Creates an affine transformation matrix containing a rotation around a normalized
        rotation `axis` and `angle` (in radians)."""
        return cls.from_mat3(Mat3.from_axis_angle(axis, angle, degrees, dtype), dtype)

    @classmethod
    def from_rotation_x(cls, angle, degrees=False, dtype=float):
        """Creates a 3D rotation matrix from `angle` (in radians) around the x axis."""
        return cls.from_mat3(Mat3.from_rotation_x(angle, degrees, dtype), dtype)

    @classmethod
    def from_rotation_y(cls, angle, degrees=False, dtype=float):
        """Creates a 3D rotation matrix from `angle` (in radians) around the y axis."""
        return cls.from_mat3(Mat3.from_rotation_y(angle, degrees, dtype), dtype)

    @classmethod
    def from_rotation_z(cls, angle, degrees=False, dtype=float):
        """Creates a 3D rotation matrix from `angle` (in radians) around the z axis."""
        return cls.from_mat3(Mat3.from_rotation_z(angle, degrees, dtype), dtype)

    @classmethod
    def from_scale(cls, scale: Vec3, dtype=float):
        """Creates an affine transformation matrix from the given non-uniform 2D `scale`.
        The resulting matrix can be used to transform 2D points and vectors."""
        return cls.from_diagonal(Vec4.from_vec3(scale, 1.), dtype=dtype)

    @classmethod
    def from_translation(cls, translation: Vec3, dtype=float):
        """Creates an affine transformation matrix from the given 3D `translation`."""
        m = cls.identity(dtype=dtype)
        m[:3, 3] = np.asarray(translation).reshape(3, )
        return m

    @classmethod
    def from_quat(cls, quat, dtype=float):
        """Creates an affine transformation matrix from the given quaternion (normalised)."""
        return cls.from_mat3(Mat3.from_quat(quat, dtype=dtype))

    @classmethod
    def look_at_gl(cls, eye: Vec3, center: Vec3, up: Vec3, dtype=float):
        """Creates a right-handed view matrix using a camera position, and up direction, and a focal point.
        This is the same as the OpenGL `gluLookAt` function. See
        https://www.khronos.org/registry/OpenGL-Refpages/gl2.1/xhtml/gluLookAt.xml
        """
        return cls.look_at_rh_y_up(eye, center, up, dtype)

    @classmethod
    def look_at_rh_y_up(cls, eye: Vec3, center: Vec3, up: Vec3, dtype=float):
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
    def perspective_gl(cls, fov_y, aspect_ratio, z_near, z_far, degrees=False, dtype=float):
        """Creates a right-handed perspective projection matrix with [-1, 1] depth range for a symmetric
        perspective-view frustum.

        The source coordinate space is right-handed and y-up, the destination space is left-handed
        and y-up, with Z(depth) clip extending from -1.0 (close) to 1.0 (far).

        This is the same as the OpenGL `gluPerspective` function. See
        https://www.khronos.org/registry/OpenGL-Refpages/gl2.1/xhtml/gluPerspective.xml

        Args:
            fov_y (float):
                Specifies the field of view angle in the y-direction, in radians if `degrees` not set to True.

            aspect_ratio (float):
                Specifies the aspect ratio that determines the field of view in the x-direction. The aspect ratio is the
                ratio of x (width) to y (height).

            z_near (float):
                Specifies the near distance from the viewer to the near clipping plane (not in the world frame,
                always positive).

            z_far (float):
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
    def orthographic_gl(cls, left, right, bottom, top, near, far, dtype=float):
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
            raise ValueError('Mat2 can only be applied to 2-D vector.')
        return (self * vec).view(Vec4)


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
