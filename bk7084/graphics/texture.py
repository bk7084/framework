import enum
from .. import gl


class TextureKind(enum.Enum):
    UVMap = 0
    CubeReflectionMap = 1
    CubeRefractionMap = 2
    EquirectangularReflectionMap = 3,
    EquirectangularRefractionMap = 4,
    CubeUVReflectionMap = 5,
    CubeUVRefractionMap = 6


class TextureWrapMode(enum.Enum):
    # Tiles the texture, creating a repeating pattern (repeat wrapping).
    Repeat = gl.GL_REPEAT
    # Clamps the texture to the last pixel at the edge (clamp to edge wrapping).
    Clamp = 1
    # Tiles the texture, creating a repeating pattern by mirroring it at every integer
    # boundary (mirrored repeat wrapping).
    Mirror = 2


class Texture:
    def __init__(self):
        self._wrap_mode = TextureWrapMode.Repeat

    @property
    def wrap_mode(self):
        return self._wrap_mode
