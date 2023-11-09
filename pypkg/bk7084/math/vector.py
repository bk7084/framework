import math
import numbers
import numpy as np


__all__ = ['Vec2', 'Vec3', 'Vec4', 'cross', 'dot', 'interpolate']


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
        dtype = kwargs.get('dtype', np.float32)

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
    def unit_x(cls, dtype=np.float32):
        return Vec2(1., 0., dtype=dtype)

    @classmethod
    def unit_y(cls, dtype=np.float32):
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
    def unit_x(cls, dtype=np.float32):
        return Vec3(1., 0., 0., dtype=dtype)

    @classmethod
    def unit_y(cls, dtype=np.float32):
        return Vec3(0., 1., 0., dtype=dtype)

    @classmethod
    def unit_z(cls, dtype=np.float32):
        return Vec3(0., 0., 1., dtype=dtype)

    @property
    def norm_squared(self):
        return np.sum(self ** 2, axis=-1)

    @property
    def norm(self) -> np.float32:
        return math.sqrt(self.dot(self))

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
        self = self.normalised
        return self

    @property
    def normalised(self):
        n = self.norm
        if n == 0. or n == 1.:
            return Vec3(self)
        else:
            return Vec3(self.x / n, self.y / n, self.z / n)

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
    def unit_x(cls, dtype=np.float32):
        return Vec4(1., 0., 0., 0., dtype=dtype)

    @classmethod
    def unit_y(cls, dtype=np.float32):
        return Vec4(0., 1., 0., 0., dtype=dtype)

    @classmethod
    def unit_z(cls, dtype=np.float32):
        return Vec4(0., 0., 1., 0., dtype=dtype)

    @classmethod
    def unit_w(cls, dtype=np.float32):
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