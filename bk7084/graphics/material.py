import os.path

import numpy as np

from .texture import Texture


class Material:
    """
    Note: case-insensitive when parsing.

    Kd: diffuse color coefficient
    Ka: Ambient color coefficient
    Ks: Specular color coefficient
    d: Dissolve factor
    Ni: Refraction index
    Ns: Specular exponent, also known as glossiness, defines the focus of specular highlights in the material.
        Ns values normally range from 0 to 1000, with a high value resulting in a tight, concentrated highlight.
    illum:
        0 - disable lighting
        1 - ambient & diffuse (set specular color to black),
        2 - full lighting
    map_Kd: Diffuse color texture map
    map_Ks: Specular color texture map
    map_Ka: Ambient color texture map
    map_Ns: Shininess texture map
    map_Bump/map_bump/bump: Bump map
    map_Norm/map_norm/norm: Normal map
    map_Disp/map_disp/disp: Displacement map
    map_d: Opacity texture map
    refl: reflection map
    """
    def __init__(self, name,
                 diffuse_map: Texture = None, bump_map: Texture = None, normal_map: Texture = None,
                 ambient=(0.8, 0.8, 0.8), diffuse=(0.8, 0.8, 0.8), specular=(1.0, 1.0, 1.0), shininess=1.0, ior=1.0,
                 dissolve=1.0, illum=2):
        self.name = name  # material name
        self.ambient = np.asarray(ambient, dtype=np.float32)  # Ka
        self.diffuse = np.asarray(diffuse, dtype=np.float32)  # Kd
        self.specular = np.asarray(specular, dtype=np.float32)  # Ks

        self.diffuse_map = diffuse_map
        self.bump_map = bump_map
        self.normal_map = normal_map

        self.shininess = shininess  # Ns
        self.refractive_index = ior  # Ni
        self.dissolve = dissolve  # d
        self.illumination_model = illum  # illumination model

    def __repr__(self):
        return 'Material <{}>\n' \
               '  - diffuse_map: {}\n' \
               '  - bump_map: {}\n' \
               '  - normal_map: {}\n' \
               '  - ambient_color: {}\n' \
               '  - diffuse_color: {}\n' \
               '  - specular_color: {}\n' \
               '  - glossiness: {}\n' \
               '  - ior: {}\n' \
               '  - dissolve: {}\n'.format(self.name,
                                           self.diffuse_map.name,
                                           self.bump_map.name,
                                           self.normal_map.name,
                                           self.ambient,
                                           self.diffuse,
                                           self.specular,
                                           self.shininess,
                                           self.refractive_index,
                                           self.dissolve)
