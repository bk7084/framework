import unittest

from bk7084 import Triangle, Vec3


class TestTriangle(unittest.TestCase):
    def test_points_access(self):
        a = Vec3([1, 2, 3])
        b = Vec3([4, 5, 6])
        c = Vec3([7, 8, 9])
        tri = Triangle(a, b, c)
        self.assertEqual(tri[0], Vec3(1, 2, 3))
        self.assertEqual(tri[1], Vec3(4, 5, 6))
        self.assertEqual(tri[2], Vec3(7, 8, 9))
        self.assertEqual(tri.p0, a)
        self.assertEqual(tri.p1, b)
        self.assertEqual(tri.p2, c)
