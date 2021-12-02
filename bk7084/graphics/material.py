import os.path

import numpy as np

import bk7084
from .texture import Texture
from ..assets import assets_dir


class Material:
    def __init__(self, name, image_path=None, ambient=None, diffuse=None, specular=None, glossiness=None, ior=None, dissolve=None, illum=None):
        self.name = name  # material name
        self._is_default = False

        if image_path is None:
            # load default checker board
            image_path = os.path.join(assets_dir(), 'textures/checker_color.png')
            self._is_default = True

        self.ambient = np.asarray(ambient, dtype=np.float32)  # Ka
        self.diffuse = np.asarray(diffuse, dtype=np.float32)  # Kd
        self.specular = np.asarray(specular, dtype=np.float32)  # Ks

        self.texture_diffuse = Texture(image_path)  # map_Kd

        self.glossiness = glossiness  # Ns
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
                                           self.glossiness,
                                           self.refractive_index,
                                           self.dissolve)

    @classmethod
    def default(cls):
        return cls(
            'default_material',
            os.path.join(assets_dir(), 'textures/checker_medium.png'),
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            0,
            1.0,
            1.0,
            0.0
        )

    @property
    def is_default(self):
        return self._is_default
