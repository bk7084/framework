import enum
import os.path
import re
import sys

from .util import ShaderCodeParser, GpuObject
from .. import gl
from ..assets.resolver import PathResolver


class ShaderType(enum.Enum):
    Vertex = gl.GL_VERTEX_SHADER
    Fragment = gl.GL_FRAGMENT_SHADER
    Pixel = gl.GL_FRAGMENT_SHADER


class Shader(GpuObject):
    def __init__(self, shader_type: ShaderType, code: str, origin='<string>', is_file=False) -> None:
        """Creates an OpenGL shbader.

        Args:
            shader_type (ShaderType):
                The shader type.

            code (str):
                The source code string or the filepath to shader source code.
        """
        super().__init__(shader_type.value, -1)

        if is_file:
            if not (os.path.isfile(code) and os.path.exists(code)):
                msg = f'Shader source file {code} does not exist or is not a valid file path.'
                raise ValueError(msg)
            with open(code, 'rt') as file:
                self._code, self._uniforms, self._attribs = ShaderCodeParser.preprocess(file.read())
                self._origin = origin
        else:
            self._code, self._uniforms, self._attribs = ShaderCodeParser.preprocess(code)
            self._origin = origin

        self._create()
        self._compile()

    @staticmethod
    def _from_file(shader_type, filepath, resolver=PathResolver()):
        filepath = resolver.resolve(filepath) if not os.path.isabs(filepath) else filepath
        if not (os.path.isfile(filepath) and os.path.exists(filepath)):
            msg = f'Shader source file {os.path.abspath(filepath)} does not exist or is not a valid file path.'
            raise ValueError(msg)

        with open(filepath, 'rt') as file:
            return Shader(shader_type,
                          code=file.read(),
                          origin=os.path.basename(filepath),
                          is_file=False)

    @property
    def code(self) -> str:
        """
        Shader source code.

        Returns:
            GLSL code string.
        """
        return self._code

    @property
    def uniforms(self):
        """
        Shader uniforms obtained from source code.

        Returns:
            A list of names of uniforms.
        """
        return self._uniforms

    @property
    def attributes(self):
        """Shader input vertex attributes.

        Returns:
            A list of names of vertex attributes.
        """
        return self._attribs

    @property
    def handle(self):
        return self._id

    def _create(self):
        if not self._code:
            raise RuntimeError("No code has been given during shader creation.")

        if self._id != -1:
            print("Trying to recreate a existing shader: SKIP.", file=sys.stderr)

        self._id = gl.glCreateShader(self._type)

        if self._id <= 0:
            raise RuntimeError("Cannot create shader object.")

    def _compile(self):
        gl.glShaderSource(self._id, self._code)
        gl.glCompileShader(self._id)
        status = gl.glGetShaderiv(self._id, gl.GL_COMPILE_STATUS)
        if not status:
            error = str(gl.glGetShaderInfoLog(self._id))
            print('\033[93m' + error + '\033[0m', file=sys.stderr)
            raise RuntimeError("Shader compilation error.")

    def _delete(self):
        gl.glDeleteShader(self._id)


class VertexShader(Shader):
    def __init__(self, code: str):
        super(VertexShader, self).__init__(ShaderType.Vertex, code, '<string>', False)

    @classmethod
    def from_file(cls, filepath):
        return super(VertexShader, cls)._from_file(ShaderType.Vertex, filepath)

    def __repr__(self):
        return f"VertexShader ({self._origin})"


class FragmentShader(Shader):
    def __init__(self, code: str):
        super(FragmentShader, self).__init__(ShaderType.Fragment, code, '<string>', False)

    @classmethod
    def from_file(cls, filepath):
        return super(FragmentShader, cls)._from_file(ShaderType.Fragment, filepath)

    def __repr__(self):
        return f"FragmentShader ({self._origin})"


class PixelShader(Shader):
    def __init__(self, code: str):
        super(PixelShader, self).__init__(ShaderType.Fragment, code, '<string>', False)

    @classmethod
    def from_file(cls, filepath):
        return super(PixelShader, cls)._from_file(ShaderType.Fragment, filepath)

    def __repr__(self):
        return f"PixelShader ({self._origin})"
