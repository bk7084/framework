import os.path

import numpy as np

import bk7084
from .texture import Texture
from ..assets import assets_dir


class Material:
    def __init__(self, name, image_path=None, ambient=None, diffuse=None, specular=None, glossiness=None, ior=None, dissolve=None, illum=None):
        self.name = name  # material name

        if image_path is None:
            # load default checker board
            image_path = os.path.join(assets_dir(), 'textures/checker_small.png')

        print(image_path)

        self.texture = Texture(image_path)  # map_Kd
        self.ambient_color = np.asarray(ambient, dtype=np.float32)  # Ka
        self.diffuse_color = np.asarray(diffuse, dtype=np.float32)  # Kd
        self.specular_color = np.asarray(specular, dtype=np.float32)  # Ks
        self.glossiness = glossiness  # Ns
        self.refractive_index = ior  # Ni
        self.dissolve = dissolve  # d
        self.illum = illum  # illumination model; illum

    def __repr__(self):
        return 'Material <{}>\n' \
               '  - texture_map: {}\n' \
               '  - ambient_color: {}\n' \
               '  - diffuse_color: {}\n' \
               '  - specular_color: {}\n' \
               '  - glossiness: {}\n' \
               '  - ior: {}\n' \
               '  - dissolve: {}\n'.format(self.name,
                                           self.texture,
                                           self.ambient_color,
                                           self.diffuse_color,
                                           self.specular_color,
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
