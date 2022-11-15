# TODO: supports other kind of shaders
# TODO: parse uniforms and generate its getter and setter
import logging
import platform
from collections.abc import Sequence

import OpenGL.GL as gl
import numpy as np

from .shader import Shader
from .util import DrawingMode, GpuObject, BindSemanticObject
from .variable import gl_get_uniform, glsl_types, gl_set_uniform, gl_type_info


def check_validity(program_id) -> bool:
    """Validates the shader program."""
    gl.glValidateProgram(program_id)
    status = gl.glGetProgramiv(program_id, gl.GL_VALIDATE_STATUS)
    return False if status == gl.GL_FALSE else True


def check_link_status(program_id) -> bool:
    """Check link status of shader program."""
    link_status = gl.glGetProgramiv(program_id, gl.GL_LINK_STATUS)
    return False if link_status == gl.GL_FALSE else True


class ShaderProgram(GpuObject, BindSemanticObject):
    __slots__ = ('_id', '_shaders', '_uniforms', '_attributes', '_is_active', '_current_activated_textured_unit')

    def __init__(self, *shaders: Shader):
        """Creates an OpenGL ShaderProgram from multiple shaders.

        Args:
            *shaders:
        """
        super().__init__(None, -1)
        assert len(shaders) > 1, "At least two Shader objects(vertex shader, fragment shader) are required."
        self._id = self._create_and_link(shaders)
        self._is_active = False

        self._buffers = None
        self._shaders = list(shaders)

        self._uniforms = {k: v for k, v in
                          [(x[0], (glsl_types[x[1]], gl.glGetUniformLocation(self._id, x[0]))) for s in self._shaders
                           for x in s.uniforms]}

        self._attributes = {}
        self._current_activated_textured_unit = -1

        # Allows get/set shader uniform as if it is an attribute of the class.
        for uniform in self._uniforms.keys():
            setattr(self.__class__, uniform, self.__create_uniform_prop(uniform))

    def __create_uniform_prop(self, name):
        @property
        def prop(self):
            return self.get_uniform(name)

        @prop.setter
        def prop(self, value):
            self.set_uniform(name, value)

        return prop

    def __getitem__(self, uniform: str):
        return self.get_uniform(uniform)

    def __setitem__(self, uniform: str, value, transpose=True):
        self.set_uniform(uniform, value, transpose=True)

    def set_uniform(self, name, value, transpose=True):
        if name in self._uniforms:
            typ, loc = self._uniforms[name]
            if typ in (gl.GL_FLOAT_MAT2, gl.GL_FLOAT_MAT3, gl.GL_FLOAT_MAT4):
                gl_set_uniform[typ](loc, 1, transpose, value)
            else:
                gl_set_uniform[typ](loc, 1, value)
        # else:
        #     logging.warning(f'Uniform {name} not existed.')

    def get_uniform(self, name):
        if name in self._uniforms:
            typ, loc = self._uniforms[name]
            type_info = gl_type_info[typ]
            buffer = np.zeros(type_info[0], dtype=type_info[2])
            gl_get_uniform[typ](self._id, loc, buffer.nbytes, buffer.data)
            return buffer

    def active_texture_unit(self, index: int):
        if self.is_valid():
            gl.glActiveTexture(gl.GL_TEXTURE0 + index)

    def active_next_texture_unit(self):
        if self.is_valid():
            self._current_activated_textured_unit += 1
            gl.glActiveTexture(gl.GL_TEXTURE0 + self._current_activated_textured_unit)

    @property
    def uniforms(self):
        return self._uniforms

    def _delete(self):
        gl.glDeleteProgram(self._id)

    def _activate(self):
        self.use()

    def _deactivate(self):
        self.unuse()

    @property
    def is_active(self):
        return self._is_active

    @staticmethod
    def _create_and_link(shaders: Sequence[Shader]):
        program_id = gl.glCreateProgram()
        for shader in shaders:
            gl.glAttachShader(program_id, shader.handle)
        gl.glLinkProgram(program_id)

        if not check_link_status(program_id):
            msg = f'Shader program link failure : {gl.glGetProgramInfoLog(program_id)}'
            raise RuntimeError(msg)

        # MacOs requires that the vao is bound before checking validity.
        if platform.system() != 'Darwin':
            if not check_validity(program_id):
                msg = f'Shader program validation failure : {gl.glGetProgramInfoLog(program_id)}'
                raise RuntimeError(msg)

        for shader in shaders:
            gl.glDetachShader(program_id, shader.handle)
        return program_id

    def use(self):
        gl.glUseProgram(self._id)
        self._is_active = True

    def unuse(self):
        gl.glUseProgram(0)
        self._is_active = False
