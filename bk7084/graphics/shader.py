import enum
import os.path
import re
import sys

from .util import ShaderCodeParser
from .. import gl

OPENGL_ERROR_REGEX = [
    # Nvidia
    # 0(7): error C1008: undefined variable "MV"
    # 0(2) : error C0118: macros prefixed with '__' are reserved
    re.compile(r'^\s*(\d+)\((?P<line_no>\d+)\)\s*:\s(?P<error_msg>.*)', re.MULTILINE),
    # ATI / Intel
    # ERROR: 0:131: '{' : syntax error parse error
    re.compile(r'^\s*ERROR:\s(\d+):(?P<line_no>\d+):\s(?P<error_msg>.*)', re.MULTILINE),
    # Nouveau
    # 0:28(16): error: syntax error, unexpected ')', expecting '('
    re.compile(r'^\s*(\d+):(?P<line_no>\d+)\((\d+)\):\s(?P<error_msg>.*)', re.MULTILINE)
]


class ShaderType(enum.Enum):
    Vertex = gl.GL_VERTEX_SHADER
    Fragment = gl.GL_FRAGMENT_SHADER


class Shader:
    def __init__(self, shader_type: ShaderType, code: str) -> None:
        """Creates an OpenGL shader.

        Args:
            shader_type (ShaderType):
                The shader type.

            code (str):
                It could be either the source code string or the filepath to the source code.
        """
        self._id = -1
        self._type = shader_type

        if os.path.isfile(code):
            with open(code, 'rt') as file:
                self._code = ShaderCodeParser.preprocess(file.read())
                self._origin = os.path.basename(code)
        else:
            self._code = ShaderCodeParser.preprocess(code)
            self._origin = '<string>'

        self._create()
        self._compile()

    def __del__(self):
        gl.glDeleteShader(self._id)

    @property
    def code(self) -> str:
        """
        Shader source code.

        Returns:
            GLSL code string.
        """
        return self._code

    @property
    def type(self):
        return self._type

    @property
    def uniforms(self):
        return NotImplementedError

    @property
    def attributes(self):
        return NotImplementedError

    @property
    def id(self):
        return self._id

    def _create(self):
        if not self._code:
            raise RuntimeError("No code has been given during shader creation.")

        if self._id != -1:
            print("Trying to recreate a existing shader: SKIP.", file=sys.stderr)

        self._id = gl.glCreateShader(self._type.value)

        if self._id <= 0:
            raise RuntimeError("Cannot create shader object.")

    def _compile(self):
        gl.glShaderSource(self._id, self._code)
        gl.glCompileShader(self._id)
        status = gl.glGetShaderiv(self._id, gl.GL_COMPILE_STATUS)
        if not status:
            error = gl.glGetShaderInfoLog(self._id).decode()
            for lineno, msg in self._parse_error(error):
                self._print_error(msg, lineno - 1)
            raise RuntimeError("Shader compilation error.")

    def _delete(self):
        gl.glDeleteShader(self._id)

    def _parse_error(self, error):
        """
        Parses a single GLSL error and extracts the line number and error description.

        Args:
            error (str): An error string returned from shader compilation.

        Returns:

        """
        print(error, file=sys.stderr)
        for regex in OPENGL_ERROR_REGEX:
            matches = list(regex.finditer(error))
            if matches:
                errors = [(int(m.group('line_no')), m.group('error_msg')) for m in matches]
                return sorted(errors, key=lambda elem: elem[0])
            else:
                raise ValueError(f"Unknown GLSL error format: \n{error}\n")

    def _print_error(self, error, lineno):
        lines = self._code.split('\n')
        start = max(0, lineno - 3)
        end = min(len(lines), lineno + 3)

        print(f'Error in {repr(self)} -> {error}\n')
        if start > 0:
            print(' ...')
        for i, line in enumerate(lines[start:end]):
            if (i + start) == lineno:
                print(' {:03i+start} {line}')
            else:
                if len(line):
                    print(' {:03i+start} {line}')
        if end < len(lines):
            print(' ...')
        print()


class VertexShader(Shader):
    def __init__(self, code=None):
        super(VertexShader, self).__init__(ShaderType.Vertex, code)

    def __repr__(self):
        return f"VertexShader ({self._origin})"


class FragmentShader(Shader):
    def __init__(self, code=None):
        super(FragmentShader, self).__init__(ShaderType.Fragment, code)

    def __repr__(self):
        return f"FragmentShader ({self._origin})"


class PixelShader(Shader):
    def __init__(self, code=None):
        super(PixelShader, self).__init__(ShaderType.Fragment, code)

    def __repr__(self):
        return f"PixelShader ({self._origin})"
