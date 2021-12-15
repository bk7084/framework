import enum
import logging

from .resolver import default_resolver
from .image import Image
from .. import gl
from ..graphics.material import Material
from ..graphics.texture import TextureWrapMode, FilterMode, TextureKind, Texture


class AssetKind(enum.Enum):
    Image = 0
    Texture = 1
    Material = 2


class AssetManager:
    def __init__(self, resolver=default_resolver):
        self._resolver = resolver
        self._materials = {}
        self._textures = {}
        self._images = {}

    def get_or_create_image(self, image_path):
        """
        Returns the requested image. Load the image if it doesn't exist.

        Args:
            image_path (str):

        Returns:
            Pillow.Image
        """
        path = self._resolver.resolve(image_path)
        if image_path not in self._images:
            logging.info(f'-- Create image {image_path}')
            image = Image.open(image_path)
            self._images[image_path] = image

        logging.info(f'Load image <{image_path}>')
        return self._images[image_path]

    def get_or_create_texture(self, image_path, kind=TextureKind.DiffuseMap, target=gl.GL_TEXTURE_2D,
                              wrap_mode=TextureWrapMode.Repeat, filter_mode=FilterMode.Linear):
        path = self._resolver.resolve(image_path)
        image = self.get_or_create_image(path)
        texture_name = f'{path}_{kind.name}'
        if texture_name not in self._textures:
            logging.info(f'-- Create texture with image <{path}>')
            texture = Texture(path, image, kind, target, wrap_mode, filter_mode)
            self._textures[texture_name] = texture

        logging.info(f'Load texture <{texture_name}>')
        return self._textures[texture_name]

    def get_or_create_material(self, name, ambient=(0.8, 0.8, 0.8), diffuse=(0.8, 0.8, 0.8), specular=(1.0, 1.0, 1.0),
                               shininess=1.0, ior=1.0, dissolve=1.0, illum=2, **kwargs):
        """

        Args:
            name (str): name of the material
            ambient (array of 3 elements): ambient color
            diffuse (array of 3 elements): diffuse color
            specular (array of 3 elements): specular color
            illum (int): illumination model
            shininess (float): shininess of specular component
            dissolve (float): opacity
            ior (float): refractive index
            **kwargs:
                diffuse_map_path (str):
                    Specifies where to find the texture used as diffuse map.
                bump_map_path (str):
                    Specifies where to find the texture used as bump map.
                normal_map_path (str):
                    Specifies where to find the texture used as normal map.
        Returns:

        """
        if name not in self._materials:
            logging.info(f'-- Create material <{name}>')
            diffuse_map_path = kwargs.get('diffuse_map_path', None)
            if diffuse_map_path is None:
                diffuse_map_path = self._resolver.resolve('textures/checker.png')
            else:
                diffuse_map_path = self._resolver.resolve(diffuse_map_path)

            bump_map_path = kwargs.get('bump_map_path')
            if bump_map_path is None:
                bump_map_path = self._resolver.resolve('textures/checker_bump.png')
            else:
                bump_map_path = self._resolver.resolve(bump_map_path)

            normal_map_path = kwargs.get('normal_map_path')
            if normal_map_path is None:
                normal_map_path = self._resolver.resolve('textures/checker_normal.png')
            else:
                normal_map_path = self._resolver.resolve(normal_map_path)

            diffuse_map = self.get_or_create_texture(diffuse_map_path)
            bump_map = self.get_or_create_texture(bump_map_path, TextureKind.BumpMap)
            normal_map = self.get_or_create_texture(normal_map_path, TextureKind.NormalMap)
            material = Material(name, diffuse_map, bump_map, normal_map, ambient, diffuse, specular, shininess, ior,
                                dissolve, illum)
            self._materials[name] = material

        logging.info(f'Load material <{name}>')
        return self._materials[name]


default_asset_mgr: AssetManager = AssetManager()
