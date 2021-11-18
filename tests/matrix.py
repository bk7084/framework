import unittest
from bk7084.geometry.matrix import *


class TestMatrix2(unittest.TestCase):
    pass


class TestMatrix3(unittest.TestCase):
    pass


class TestMatrix4(unittest.TestCase):
    def test_ctor_zero_params(self):
        pass

    def test_look_at_matrix(self):
        m = Mat4.look_at_gl()