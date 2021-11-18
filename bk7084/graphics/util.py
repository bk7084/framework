import enum
import re
from .. import gl


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


# TODO: merge includes


class ShaderCodeParser:
    @classmethod
    def parse(self, code):
        pass

    @classmethod
    def preprocess(cls, code: str) -> str:
        if code:
            cls.remove_comments(code)
            # cls.remove_version(code)
        return code

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
