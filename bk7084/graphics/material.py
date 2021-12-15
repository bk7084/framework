import os.path

import numpy as np

import bk7084
from .texture import Texture, TextureKind
from ..assets import default_resolver
from ..assets.manager import AssetManager, default_asset_mgr


class Material:
    """
    Kd: diffuse color coefficient
    Ka: Ambient color coefficient
    Ks: Specular color coefficient
    d: Dissolve factor
    Ni: Refraction index
    illum:
        0 - disable lighting
        1 - ambient & diffuse (set specular color to black),
        2 - full lighting
    map_Kd: Diffuse color texture map
    map_Ks: Specular color texture map
    map_ka: Ambient color texture map
    map_Ns: Shininess texture map
    map_Bump: Bump texture map
    map_d: Opacity texture map
    map_Disp: Displacement map
    refl: reflection type
    """
    def __init__(self, name, image_path=None, ambient=None, diffuse=None, specular=None, shininess=None, ior=None,
                 dissolve=None, illum=None, resolver=default_resolver):
        self.name = name  # material name

        self._is_default = True
        resolved = resolver.resolve('textures/checker.png')

        if image_path is not None:
            resolved = resolver.resolve(image_path)
            self._is_default = False

        self.ambient = np.asarray(ambient, dtype=np.float32)  # Ka
        self.diffuse = np.asarray(diffuse, dtype=np.float32)  # Kd
        self.specular = np.asarray(specular, dtype=np.float32)  # Ks

        self.texture_diffuse = Texture(resolved)  # map_Kd

        self.shininess = shininess  # Ns
        self.refractive_index = ior  # Ni
        self.dissolve = dissolve  # d
        self.illumination_model = illum  # illumination model

        self.texture_ambient = None
        self.texture_specular_color = None
        self.texture_specular_highlight = None
        self.texture_alpha = None
        self.texture_normal_map = None

    def __repr__(self):
        return 'Material <{}>\n' \
               '  - texture_map: {}\n' \
               '  - ambient_color: {}\n' \
               '  - diffuse_color: {}\n' \
               '  - specular_color: {}\n' \
               '  - glossiness: {}\n' \
               '  - ior: {}\n' \
               '  - dissolve: {}\n'.format(self.name,
                                           self.texture_diffuse,
                                           self.ambient,
                                           self.diffuse,
                                           self.specular,
                                           self.shininess,
                                           self.refractive_index,
                                           self.dissolve)

    @classmethod
    def default(cls, texture=None):
        return cls(
            'default_material',
            texture,
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            1.0,
            1.0,
            1.0,
            0.0
        )

    @property
    def is_default(self):
        return self._is_default
