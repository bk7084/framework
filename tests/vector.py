import unittest

from bk7084.geometry.vector import *
import numpy as np


class TestVector3(unittest.TestCase):
    def test_ctor_single_value(self):
        a = Vec3()
        b = Vec3(0.0)
        c = Vec3(0.0, 0.0, 0.0)
        self.assertEqual(a, b)
        self.assertEqual(b, c)

    def test_ctor_multi_values(self):
        a = Vec3([1.0, 2.0, 0.0])
        b = Vec3(1.0, 2.0, 0.0)
        self.assertEqual(a, b)

    def test_ctor_array_like(self):
        arr0 = np.array([.1, .2, .3])
        tuple0 = (.1, .2, .3)
        self.assertEqual(Vec3(arr0), Vec3(tuple0))

    def test_ctor_irregular(self):
        self.assertEqual(Vec3([1.0], [2.0, 3.0], [4.0], 5.0), Vec3(1.0, 2.0, 3.0, 4.0))

    def test_conversions(self):
        v = Vec3(1.0, 2.0, 0.2)
        v0 = Vec2(1.0, 2.0)
        v1 = Vec4(1.0, 2.0, 0.2, 0.8)
        self.assertEqual(Vec3(v0, 0.2), v)
        self.assertEqual(Vec3.from_vec2(v0, 0.2), v)
        self.assertEqual(Vec3.from_vec4(v1), v)
        self.assertEqual(Vec3.from_vec4(v1), Vec3(v1))

    def test_dot_product(self):
        va = Vec3(1.0, 0.0, 0.0)
        vb = Vec3(0.1, 0.2, 0.3).normalised

