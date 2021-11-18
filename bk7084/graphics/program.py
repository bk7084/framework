# TODO: supports other kind of shaders
# TODO: parse uniforms and generate its getter and setter
import platform
from collections import Sequence

import OpenGL.GL as gl

from .shader import Shader
from .util import DrawingMode


def check_validity(program_id) -> bool:
    """Validates the shader program."""
    gl.glValidateProgram(program_id)
    status = gl.glGetProgramiv(program_id, gl.GL_VALIDATE_STATUS)
    return False if status == gl.GL_FALSE else True


def check_link_status(program_id) -> bool:
    """Check link status of shader program."""
    link_status = gl.glGetProgramiv(program_id, gl.GL_LINK_STATUS)
    return False if link_status == gl.GL_FALSE else True


class ShaderProgram:
    def __init__(self, *shaders):
        """Creates an OpenGL ShaderProgram from multiple shaders.

        TODO

        Args:
            *shaders:
        """
        assert len(shaders) > 1, "At least two Shader objects(vertex shader, fragment shader) are required."
        self._id = self._create_and_link(shaders)
        self._is_active = False

        self._buffers = None
        self._shaders = [*shaders]

        self._uniforms = {}
        self._attributes = {}

    def __enter__(self):
        """Start use of the program."""
        self.use()

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Stop use of the program"""
        self.unuse()

    def __del__(self):
        gl.glDeleteProgram(self._id)

    def __getitem__(self, uniform: str):
        pass

    def __setitem__(self, uniform: str, value):
        pass

    @property
    def raw_id(self):
        return self._id

    @property
    def is_active(self):
        return self._is_active

    def update_uniform(self, name, value):
        pass

    def _create_and_link(self, shaders: Sequence[Shader]):
        program_id = gl.glCreateProgram()
        for shader in shaders:
            gl.glAttachShader(program_id, shader.id)
        gl.glLinkProgram(program_id)

        # MacOs requires that the vao is bound before checking validity.
        if platform.system() != 'Darwin':
            if not check_validity(program_id):
                raise RuntimeError(f'Shader program validation failure : {gl.glGetProgramInfoLog(program_id)}')

        if not check_link_status(program_id):
            raise RuntimeError(
                f'Shader program link failure : {gl.glGetProgramInfoLog(program_id)}')
        for shader in shaders:
            gl.glDetachShader(program_id, shader.id)
        return program_id

    def use(self):
        gl.glUseProgram(self._id)
        self._is_active = True

    def unuse(self):
        gl.glUseProgram(0)
        self._is_active = False

    def draw(self, primitive_type: DrawingMode):
        pass
