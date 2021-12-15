import enum
import logging

from PIL import Image

from .util import GpuObject, BindSemanticObject
from .. import gl


class TextureKind(enum.Enum):
    # The texture is combined with the result of the diffuse lighting equation.
    DiffuseMap = 0
    # The texture is combined with the result of the specular lighting equation.
    SpecularMap = 1
    # The texture is combined with the result of the ambient lighting equation.
    AmbientMap = 2
    # The texture is added to the result of the lighting calculation.It isn't influenced by incoming light.
    EmissiveMap = 3
    # The texture is a height map.
    HeightMap = 4
    # The texture is a bump map.
    BumpMap = 5
    # The texture is a(tangent space) normal - map.
    NormalMap = 6
    # The texture defines the glossiness of the material.
    ShininessMap = 7
    # The texture defines per - pixel opacity.
    OpacityMap = 8
    # Displacement texture
    DisplacementMap = 9
    # Lightmap texture(aka Ambient Occlusion)
    LightMap = 10
    # Reflection texture.
    ReflectionMap = 11


class TextureWrapMode(enum.Enum):
    # Tiles the texture, creating a repeating pattern (repeat wrapping).
    Repeat = gl.GL_REPEAT
    # Clamps the texture to the last pixel at the edge (clamp to edge wrapping).
    Clamp = gl.GL_CLAMP_TO_EDGE
    # Tiles the texture, creating a repeating pattern by mirroring it at every integer
    # boundary (mirrored repeat wrapping).
    Mirror = gl.GL_MIRRORED_REPEAT


class FilterMode(enum.Enum):
    Linear = gl.GL_LINEAR
    Nearest = gl.GL_NEAREST


class Texture(GpuObject, BindSemanticObject):
    def __init__(self, image_path, image, kind=TextureKind.DiffuseMap, target=gl.GL_TEXTURE_2D,
                 wrap_mode=TextureWrapMode.Repeat, filter_mode=FilterMode.Linear):
        super().__init__(gl.GL_TEXTURE_2D, -1)
        self._wrap_mode = wrap_mode
        self._filter_mode = filter_mode
        self._name_path = image_path
        self._target = target
        self._image = image
        self._kind = kind
        num_comps = self._image.num_channels
        self._format = gl.GL_RED if num_comps == 1 else gl.GL_RGB if num_comps == 3 else gl.GL_RGBA

        self._id = gl.glGenTextures(1)
        gl.glBindTexture(self._target, self._id)
        gl.glTexImage2D(self._target, 0, self._format, self._image.width, self._image.height,
                            0, self._format, gl.GL_UNSIGNED_BYTE, self._image.raw_data)
        gl.glGenerateMipmap(self._target)

        gl.glTexParameteri(self._target, gl.GL_TEXTURE_WRAP_S, self._wrap_mode.value)
        gl.glTexParameteri(self._target, gl.GL_TEXTURE_WRAP_T, self._wrap_mode.value)
        gl.glTexParameteri(self._target, gl.GL_TEXTURE_MIN_FILTER, self._filter_mode.value)
        gl.glTexParameteri(self._target, gl.GL_TEXTURE_MAG_FILTER, self._filter_mode.value)

    def _delete(self):
        if self.is_valid():
            gl.glDeleteTextures(1, [self._id])

    @staticmethod
    def _load_image(image_path):
        try:
            img = Image.open(image_path).transpose(Image.FLIP_TOP_BOTTOM)
            num_comps = len(img.getbands())
            tex_format = gl.GL_RED if num_comps == 1 else \
                gl.GL_RGB if num_comps == 3 else gl.GL_RGBA

            return img, tex_format

        except FileNotFoundError:
            import sys
            print(f'Texture failed to load at path: {image_path}', file=sys.stderr)

    def _activate(self):
        self.bind()

    def _deactivate(self):
        self.unbind()

    def bind(self):
        gl.glBindTexture(self._target, self._id)

    def unbind(self):
        gl.glBindTexture(self._target, 0)

    @property
    def wrap_mode(self):
        return self._wrap_mode

    @property
    def filter_mode(self):
        return self._filter_mode

    @property
    def name(self):
        return self._name_path
