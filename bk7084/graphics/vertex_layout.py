from .. import gl

import enum
from collections import OrderedDict
from dataclasses import dataclass


@enum.unique
class VertexAttrib(enum.Enum):
    Position = 0,
    Normal = 1,
    Tangent = 2,
    BiTangent = 3,
    Color0 = 4,
    Color1 = 5,
    Color2 = 6,
    Color3 = 7,
    BlendIndices = 8,
    BlendWeight = 9,
    TexCoord0 = 10,
    TexCoord1 = 11,
    TexCoord2 = 12,
    TexCoord3 = 13,
    TexCoord4 = 14,
    TexCoord5 = 15,
    TexCoord6 = 16,
    TexCoord7 = 17,

    @property
    def default_dimension(self):
        if self is VertexAttrib.Position or self is VertexAttrib.Normal:
            return 3
        elif VertexAttrib.TexCoord0 <= self.value <= VertexAttrib.TexCoord7:
            return 2
        elif self is VertexAttrib.Tangent or VertexAttrib.Color0 <= self.value <= VertexAttrib.Color3:
            return 4
        else:
            raise Exception(f"{self.name} doesn't have default dimension.")


@enum.unique
class VertexAttribFormat(enum.Enum):
    Float64 = gl.GL_DOUBLE
    Float32 = gl.GL_FLOAT
    UInt32 = gl.GL_UNSIGNED_INT,
    SInt32 = gl.GL_INT,
    Float16 = gl.GL_HALF_FLOAT
    UInt16 = gl.GL_UNSIGNED_SHORT,
    SInt16 = gl.GL_SHORT,
    UInt8 = gl.GL_UNSIGNED_BYTE
    SInt8 = gl.GL_BYTE

    def size_in_bytes(self):
        if self is VertexAttribFormat.Float64:
            return 8
        elif self is VertexAttribFormat.Float32 or self is VertexAttribFormat.UInt32 or self is VertexAttribFormat.SInt32:
            return 4
        elif self is VertexAttribFormat.Float16 or self is VertexAttribFormat.UInt16 or \
                self is VertexAttribFormat.SInt16 or self is VertexAttribFormat.SNorm16:
            return 2
        else:
            return 1

    @property
    def gl_type(self):
        return self.value


@dataclass
class VertexAttribDescriptor:
    attrib: VertexAttrib
    format: VertexAttribFormat
    dimension: int

    def __init__(self, attrib: VertexAttrib, fmt: VertexAttribFormat, dim=1):
        self.attrib = attrib
        self.format = fmt
        self.dimension = dim


class VertexLayout:
    def __init__(self, *descriptors):
        """Construct the vertex layout from vertex attribute descriptors.

        Args:
            *descriptors:
                Vertex attribute descriptors specify the vertex attribute and its associated data type
                (format) and dimension. It could be either `VertexAttribDescriptor` objects or tuples
                of form:

                (attrib, format, n),

                `attrib` is the `VertexAttrib`, `format` is the `VertexAttribFormat`. `n` is the dimension of this attribute.
        """
        self._description = OrderedDict()
        self._stride = 0
        if descriptors:
            for desc in descriptors:
                if isinstance(desc, VertexAttribDescriptor):
                    self._description[desc.attrib] = (desc.format, desc.dimension)
                    self._stride += desc.format.size_in_bytes() * desc.dimension
                elif isinstance(desc, tuple):
                    self._description[desc[0]] = (desc[1], desc[2])
                    self._stride += desc[1].size_in_bytes() * desc[2]

    @classmethod
    def new(cls):
        return cls()

    @property
    def description(self):
        return self._description

    @classmethod
    def default(cls):
        return cls(VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3),
                   VertexAttribDescriptor(VertexAttrib.Color0, VertexAttribFormat.Float32, 4),
                   VertexAttribDescriptor(VertexAttrib.TexCoord0, VertexAttribFormat.Float32, 2),
                   VertexAttribDescriptor(VertexAttrib.Normal, VertexAttribFormat.Float32, 3),
                   VertexAttribDescriptor(VertexAttrib.Tangent, VertexAttribFormat.Float32, 3))

    def has(self, attrib: VertexAttrib):
        return attrib in self._description

    def offset_of(self, attrib: VertexAttrib):
        if attrib not in self._description:
            raise ValueError(f'{attrib} not exists in current vertex layout.')

        offset = 0
        for attr, (fmt, dim) in self._description.items():
            if attr is attrib:
                return offset
            else:
                offset += fmt.size_in_bytes() * dim

        return offset

    @property
    def stride(self):
        """Returns vertex stride (size of a single vertex in bytes)."""
        return self._stride

    @property
    def attrib_count(self):
        return len(self._description)

    def compute_buffer_size(self, count):
        """Returns the size of vertex buffer in bytes for a certain count of vertices.

        Args:
            count (int): Numbers of vertices.

        Returns:
            int. Size of vertex buffer for a specific number of vertices.
        """
        return count * self._stride
