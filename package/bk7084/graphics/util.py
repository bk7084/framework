import abc
import atexit
import enum
import re
import OpenGL.error

from .variable import glsl_types
from .. import gl
import uuid


class BindSemanticObject(metaclass=abc.ABCMeta):
    def __enter__(self):
        self._activate()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self._deactivate()

    @abc.abstractmethod
    def _activate(self):
        raise NotImplementedError

    @abc.abstractmethod
    def _deactivate(self):
        raise NotImplementedError


class GpuObject:
    def __init__(self, gl_type, gl_id):
        self._id = gl_id
        self._uuid = uuid.uuid1()
        self._type = gl_type
        atexit.register(self.__delete)

    def __delete(self):
        try:
            # if the context is alive, delete the resource from GPU
            self._delete()
        except OpenGL.error.NullFunctionError as error:
            # do nothing; context is not existing anymore
            pass
        except gl.GLError as error:
            # do nothing, context doesn't exists anymore
            if error.err != 1282:
                raise error

    @abc.abstractmethod
    def _delete(self):
        raise NotImplementedError

    @property
    def handle(self):
        return self._id

    @property
    def uuid(self):
        return self._uuid

    @property
    def gl_type(self):
        return self._type

    def is_valid(self):
        return self._id > 0

    def delete(self):
        self._delete()


@enum.unique
class DrawingMode(enum.Enum):
    Points = gl.GL_POINTS
    Lines = gl.GL_LINES
    LineStrip = gl.GL_LINE_STRIP
    LineLoop = gl.GL_LINE_LOOP
    Triangles = gl.GL_TRIANGLES
    TrianglesAdjacency = gl.GL_TRIANGLES_ADJACENCY
    TriangleStrip = gl.GL_TRIANGLE_STRIP
    TriangleStripAdjacency = gl.GL_TRIANGLE_STRIP_ADJACENCY
    TriangleFan = gl.GL_TRIANGLE_FAN


# Valid OpenGL buffer binding targets.
@enum.unique
class BufferBindingTarget(enum.Enum):
    ArrayBuffer = gl.GL_ARRAY_BUFFER  # Vertex attributes
    AtomicCounterBuffer = gl.GL_ATOMIC_COUNTER_BUFFER  # Atomic counter storage, requires GL version >= 4.2
    CopyReadBuffer = gl.GL_COPY_READ_BUFFER  # Buffer copy source, requires GL version >= 3.1
    CopyWriteBuffer = gl.GL_COPY_WRITE_BUFFER  # Buffer copy destination
    DispatchIndirectBuffer = gl.GL_DISPATCH_INDIRECT_BUFFER  # Indirect compute dispatch commands, requires GL version >= 4.3
    DrawIndirectBuffer = gl.GL_DRAW_INDIRECT_BUFFER  # Indirect command arguments
    ElementArrayBuffer = gl.GL_ELEMENT_ARRAY_BUFFER  # Vertex array indices
    PixelPackBuffer = gl.GL_PIXEL_PACK_BUFFER  # Pixel read target
    PixelUnpackBuffer = gl.GL_PIXEL_UNPACK_BUFFER  # Texture data source
    QueryBuffer = gl.GL_QUERY_BUFFER  # Query result buffer, requires GL version >= 4.4
    ShaderStorageBuffer = gl.GL_SHADER_STORAGE_BUFFER  # Read-write storage for shaders, requires GL version >= 4.3
    TextureBuffer = gl.GL_TEXTURE_BUFFER  # Texture data buffer, requires GL version >= 3.1
    TransformFeedbackBuffer = gl.GL_TRANSFORM_FEEDBACK_BUFFER  # Transform feedback buffer
    UniformBuffer = gl.GL_UNIFORM_BUFFER  # Uniform block storage, requires GL version >= 3.1


# Valid OpenGL buffer data storage.
# https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glBufferStorage.xhtml
@enum.unique
class BufferStorageFlag(enum.Flag):
    Dynamic = gl.GL_DYNAMIC_STORAGE_BIT
    MapRead = gl.GL_MAP_READ_BIT
    MapWrite = gl.GL_MAP_WRITE_BIT
    MapPersistent = gl.GL_MAP_PERSISTENT_BIT
    MapCoherent = gl.GL_MAP_COHERENT_BIT
    ClientStorage = gl.GL_CLIENT_STORAGE_BIT


@enum.unique
class AccessPolicy(enum.Enum):
    """
    Access policy for OpenGL buffer object.
    """
    ReadOnly = gl.GL_READ_ONLY
    WriteOnly = gl.GL_WRITE_ONLY
    ReadWrite = gl.GL_READ_WRITE


@enum.unique
class DataUsage(enum.Enum):
    """
    Usage pattern of stored data.
    https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glBufferData.xhtml

    Frequency of access:
        - Stream: the data store contents will be modified once and used at most a few times.
        - Static: the data store contents will be modified once and used many times.
        - Dynamic: the data store contents will be modified repeatedly and used many times.

    Nature of access:
        - Draw:
            the data store contents are modified by the application, and used as the source for GL drawing
            and image specification commands.

        - Read:
            the data store contents are modified by reading data from the GL, and used to return that data
            when queried by the application.

        - Copy:
            the data store contents are modified by reading data from the GL, and used as the source for GL
             drawing and image specification commands.
    """
    StreamDraw = gl.GL_STREAM_DRAW
    StreamRead = gl.GL_STREAM_READ
    StreamCopy = gl.GL_STREAM_COPY
    StaticDraw = gl.GL_STATIC_DRAW
    StaticRead = gl.GL_STATIC_READ
    StaticCopy = gl.GL_STATIC_COPY
    DynamicDraw = gl.GL_DYNAMIC_DRAW
    DynamicRead = gl.GL_DYNAMIC_READ
    DynamicCopy = gl.GL_DYNAMIC_COPY


class ShaderCodeParser:
    @classmethod
    def parse(self, code):
        pass

    @classmethod
    def preprocess(cls, code: str):
        uniforms = []
        attribs = []
        if code:
            cls.remove_comments(code)
            structs = cls.parse_defined_struct(code)
            uniforms = cls.parse_declarations(code, 'uniform', user_defined_types=structs)
            attribs = cls.parse_declarations(code, 'in')
        return code, uniforms, attribs

    @classmethod
    def remove_comments(cls, code: str) -> str:
        """
        Replace C-style comment from GLSL code string.

        Args:
            code (str): GLSL code string.

        Returns:
            GLSL code string without comments.
        """
        regex = re.compile(r"(\".*?\"|\'.*?\')|(/\*.*?\*/|//[^\r\n]*\n)", re.MULTILINE | re.DOTALL)

        return regex.sub(
            lambda matched: "" if matched.group(2) is not None else matched.group(1),
            code
        )

    @classmethod
    def parse_defined_struct(cls, code: str):
        struct_regex = re.compile(r"(?s)struct\s*(?P<sname>\w+)\s*\{(?P<sdef>.*?)\};", re.MULTILINE)
        struct_field_regex = re.compile(r"\s+(?P<type>\w+)\s+(?P<name>[\w,\[\]\n = \.$]+);")
        structs = {}
        for matched in re.finditer(struct_regex, code):
            sname = matched.group('sname')
            structs[sname] = []
            for field in re.finditer(struct_field_regex, matched.group('sdef')):
                var_type = field.group('type')
                var_name = field.group('name')
                structs[sname].append((var_type, var_name))

        return structs

    @classmethod
    def remove_version(cls, code: str) -> str:
        """
        Remove OpenGL version directive.

        Args:
            code (str): GLSL code string.

        Returns:
            GLSL code string with version directive removed.
        """
        regex = re.compile('\#\s*version[^\r\n]*\n', re.MULTILINE | re.DOTALL)
        return regex.sub('\n', code)

    @classmethod
    def parse_declarations(cls, code, qualifiers='', user_defined_types={}):
        """Extract declaraions of different types (type qualifiers) inside of shader.

        Note:
              Do NOT pass multiple qualifiers (not working for the moment).

        Args:
            code (str):
                Shader source code string.

            qualifier (str):
                GLSL type qualifiers. See https://www.khronos.org/opengl/wiki/Type_Qualifier_(GLSL).
                A string of qualifier(s) separated by comma.

        Returns:
            list of parsed declarations.
        """
        # todo: deal with multiple qualifiers.
        if qualifiers != '':
            variables = []
            # qualifiers = f"({'|'.join(list(map(str.strip, qualifiers.split(','))))})"
            regex = re.compile(f'{qualifiers}\s+(?P<type>\w+)\s+(?P<names>[\w,\[\]\n = \.$]+);')
            for matched in re.finditer(regex, code):
                var_type = matched.group('type')
                var_names = list(map(str.strip, matched.group('names').split(',')))

                if var_type not in glsl_types and var_type in user_defined_types:
                    user_defined = user_defined_types[var_type]
                    for var_name in var_names:
                        for field_type, field_name in user_defined:
                            variables.append((f'{var_name}.{field_name}', field_type))
                else:
                    for var_name in var_names:
                        variables.append((var_name, var_type))
            return variables
        else:
            return ''
