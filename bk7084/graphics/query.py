import ctypes
import enum
import sys

import numpy as np

from .util import DataUsage, BufferBindingTarget, GpuObject, BindSemanticObject
from .vertex_layout import VertexLayout
from .. import gl


class QueryTarget(enum.Enum):
    SamplesPassed = gl.GL_SAMPLES_PASSED
    AnySamplesPassed = gl.GL_ANY_SAMPLES_PASSED
    AnySamplesPassedConservative = gl.GL_ANY_SAMPLES_PASSED_CONSERVATIVE
    PrimitivesGenerated = gl.GL_PRIMITIVES_GENERATED
    TransformFeedbackPrimitivesWritten = gl.GL_TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN
    TimeElapsed = gl.GL_TIME_ELAPSED


class Query(GpuObject):
    def __init__(self, target: QueryTarget):
        super().__init__(target.value, -1)

        if target not in QueryTarget:
            raise ValueError(f'{target} is not a valid OpenGL query target.')

        self._target = target.value
        self._id = gl.glGenQueries(1)[0]
        self._is_in_use = False

    def _delete(self):
        if self.is_valid():
            gl.glDeleteQueries(1, [self._id])

    @property
    def target(self) -> gl.Constant:
        return self._target

    def begin(self):
        gl.glColorMask(gl.GL_FALSE, gl.GL_FALSE, gl.GL_FALSE, gl.GL_FALSE)
        gl.glDepthMask(gl.GL_FALSE)
        gl.glEnable(gl.GL_DEPTH_TEST)
        gl.glBeginQuery(self._target, self._id)
        self._is_in_use = True

    def end(self):
        gl.glEndQuery(self._target)
        gl.glColorMask(gl.GL_TRUE, gl.GL_TRUE, gl.GL_TRUE, gl.GL_TRUE)
        gl.glDepthMask(gl.GL_TRUE)
        self._is_in_use = False

    def result(self):
        # check availability of the query result
        if gl.glGetQueryObjectiv(self._id, gl.GL_QUERY_RESULT_AVAILABLE):
            return gl.glGetQueryObjectiv(self._id, gl.GL_QUERY_RESULT)

    def is_in_use(self):
        return self._is_in_use
