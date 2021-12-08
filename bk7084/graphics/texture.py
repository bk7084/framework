import ctypes
import enum
import logging

from PIL import Image

from .util import GpuObject, BindSemanticObject
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
    Clamp = gl.GL_CLAMP_TO_EDGE
    # Tiles the texture, creating a repeating pattern by mirroring it at every integer
    # boundary (mirrored repeat wrapping).
    Mirror = gl.GL_MIRRORED_REPEAT


class FilterMode(enum.Enum):
    Linear = gl.GL_LINEAR
    Nearest = gl.GL_NEAREST


class Texture(GpuObject, BindSemanticObject):
    def __init__(self, image_path, target=gl.GL_TEXTURE_2D, wrap_mode=TextureWrapMode.Repeat, filter_mode=FilterMode.Linear):
        super().__init__(gl.GL_TEXTURE_2D, -1)
        self._src = image_path
        self._wrap_mode = wrap_mode
        self._filter_mode = filter_mode
        self._image_path = image_path
        self._target = gl.GL_TEXTURE_2D

        logging.info(f'Create texture with image <{self._image_path}>')
        with Image.open(image_path).convert('RGBA').transpose(Image.FLIP_TOP_BOTTOM) as img:
            num_comps = len(img.getbands())
            self._format = gl.GL_RED if num_comps == 1 else \
                gl.GL_RGB if num_comps == 3 else gl.GL_RGBA

            self._id = gl.glGenTextures(1)
            gl.glBindTexture(self._target, self._id)
            gl.glTexImage2D(self._target, 0, self._format, img.width, img.height,
                            0, self._format, gl.GL_UNSIGNED_BYTE, img.tobytes())
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
