import enum
import logging
from dataclasses import dataclass

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
    ClampEdge = gl.GL_CLAMP_TO_EDGE
    # Tiles the texture, creating a repeating pattern by mirroring it at every integer
    # boundary (mirrored repeat wrapping).
    Mirror = gl.GL_MIRRORED_REPEAT
    ClampBorder = gl.GL_CLAMP_TO_BORDER


class FilterMode(enum.Enum):
    Linear = gl.GL_LINEAR
    Nearest = gl.GL_NEAREST


@dataclass(frozen=True)
class ObjTextureParams:
    PARAM_NAMES = ['o', 's', 't', 'bm', 'blendu', 'blendv', 'cc', 'clamp', 'texres', 'mm', 'imfchan', 'type']
    """
    Parameters for a texture loaded from an OBJ file.
    """
    # Offsets added to the u, v and w coordinates of the texture map.
    # Default is (0, 0, 0).
    #
    # -o u v w
    offset: tuple = (0.0, 0.0, 0.0)
    # Scaling factors applied to the u, v and w coordinates of the texture map.
    # Default is (1, 1, 1).
    #
    # -s u v w
    scale: tuple = (1.0, 1.0, 1.0)
    # Specifies the turbulence values. Adding turbulence to a texture along a
    # specified direction adds variance to the original image and allows a simple
    # image to be repeated over a larger area without noticeable tiling effects.
    # The default is (0, 0, 0).
    #
    # -t u v w
    turbulence: tuple = (0.0, 0.0, 0.0)
    # Bias and gain values used to modify the color values of the texture map.
    # Modifies the range over which scalar or color texture values may vary. This
    # has an effect only during rendering and does not change the file.
    #
    # "base" adds a base value to the texture values. Default is 0.0. A positive
    # value makes everything brighter, a negative value makes everything dimmer.
    #
    # "gain" expands the range of the texture values. Default is 1.0; the range is
    # unlimited. Increasing the number increases the contrast.
    #
    # -mm base gain
    mm = (0.0, 1.0)
    # Turns on or off the clamping of texture coordinates. When clamping is on,
    # texture coordinates are limited to the range [0, 1]. Default is off.
    # When clamping is turned on, one copy of the texture is mapped onto the
    # surface, rather than repeating copies of the original texture across the
    # surface of a polygon, which is the default.  Outside the origin
    # texture, the underlying material is unchanged.
    #
    # -clamp on | off
    clamp = False
    # Turns on or off the texture blending in the vertical direction. Default is on.
    #
    # -blendu on | off
    blend_u = True
    # Turns on or off the texture blending in the horizontal direction. Default is on.
    #
    # -blendv on | off
    blend_v = True
    # Specifies a multiplier for the bump map values. Values stored with the texture are
    # multiplied by this value before they are applied to the surface. The default value is 1.0.
    # It can be positive or negative. For best results, the value should be between -1.0 and 1.0.
    #
    # -bm mult
    bump_mult = 1.0
    # Specifies the channel of the texture used to create a scalar or bump texture. Scalar
    # textures are applied to transparency, specular exponent, decal, and displacement maps.
    # Channel values can be r, g, b, m, l, or z. The default for bump and scalar textures is
    # 'l'(luminance), unless you are building a decal. In that case, the default is 'm'(matte).
    #
    # -imfchan r | g | b | m | l | z
    imfchan = 'l'
    # Turns on or off the color correction for the texture mao. It can only be used with
    # map_Ka, map_Kd, and map_Ks. Default is off.
    #
    # -cc on | off
    color_correction = False
    # Specifies the resolution of texture created when and image is used. The default texture
    # size is the largest power of two that does not exceed the original image size.
    #
    # If the source image is an exact power of 2, the texture cannot be built
    # any larger.  If the source image size is not an exact power of 2, you
    # can specify that the texture be built at the next power of 2 greater
    # than the source image size.
    #
    # The original image should be square, otherwise, it will be scaled to
    # fit the closest square size that is not larger than the original.
    # Scaling reduces sharpness.
    #
    # -texres resolution
    tex_res = None
    # Used in combination with reflection map, to specify how the image should be interpreted.
    # It can be 'cube_top', 'cube_bottom', 'cube_left', 'cube_right', 'cube_front', 'cube_back', 'sphere'.
    #
    # refl -type sphere -options -args filename
    refl_type = None


class Texture(GpuObject, BindSemanticObject):
    def __init__(self, image_path, image, kind=TextureKind.DiffuseMap, target=gl.GL_TEXTURE_2D,
                 wrap_mode=TextureWrapMode.Repeat, filter_mode=FilterMode.Linear, gen_mipmap=True, obj_tex_params=None):
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

        if gen_mipmap:
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
