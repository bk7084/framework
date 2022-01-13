import abc
from re import S
import uuid
from collections import namedtuple

from .mesh import Mesh, SubMesh
from .entity import Entity
from ..math import Mat4
from ..misc import PaletteDefault as Palette

import numpy as np


class Component(metaclass=abc.ABCMeta):
    def __init__(self, cast_shadow=True):
        self._id = uuid.uuid1()
        self._transform = Mat4.identity()
        self._is_drawable = True
        self._cast_shadow = cast_shadow

    @property
    @abc.abstractmethod
    def mesh(self) -> Mesh:
        raise NotImplementedError

    @property
    def transform(self):
        return self._transform

    @transform.setter
    def transform(self, value: Mat4):
        self._transform = value

    @property
    def id(self):
        return self._id

    @property
    def cast_shadow(self):
        return self._cast_shadow

    @cast_shadow.setter
    def cast_shadow(self, value):
        self._cast_shadow = value

    @property
    def drawable(self):
        return self._is_drawable

    @drawable.setter
    def drawable(self, value):
        self._is_drawable = value

    def draw(self, matrices=None, **kwargs):
        if self._is_drawable:
            matrix = Mat4.identity()
            if matrices is not None:
                for m in matrices:
                    matrix = m * matrix
            self.mesh.draw(matrix, **kwargs)

    def compute_energy(self, shader, light, viewport_size, depth_map, matrices=None):
        if self._is_drawable:
            transform = Mat4.identity()
            if matrices is not None:
                for m in matrices:
                    transform = m * transform
            self.mesh.compute_energy(shader, transform, light, viewport_size, depth_map)


class Building(Entity):
    def __init__(self, name=None):
        super().__init__(name)
        self._components = []  # stores the Component objects
        self._root_components = []  # store the Component objects index
        self._hierarchy = {}  # store the parent index of components: { comp_index: parent_index }
        self._is_drawable = True
        self._transform = Mat4.identity()

    def append(self, comp: Component, parent: Component = None):
        exists = False
        for c in self._components:
            if c.id == comp.id:
                exists = True
                break

        if exists:
            raise ValueError('Component already exists.')

        self._components.append(comp)
        index = len(self._components) - 1

        parent_index = -1

        if parent is not None:
            try:
                parent_index = self._components.index(parent)
            except ValueError:
                raise ValueError('Parent does not exist.')

        self._hierarchy[index] = parent_index

        if parent_index == -1:
            self._root_components.append(index)

    @property
    def components(self):
        return self._components

    @property
    def transform(self):
        return self._transform

    @transform.setter
    def transform(self, value: Mat4):
        self._transform = value

    def _parent_list(self, comp, parent_list):
        if comp in self._root_components:
            return parent_list
        else:
            if comp in self._hierarchy:
                parent_list.append(self._hierarchy[comp])
                return self._parent_list(self._hierarchy[comp], parent_list)
            else:
                return []

    def transform_of(self, comp):
        matrix = Mat4.identity()
        if comp in self._components:
            idx = self._components.index(comp)
            parents = self._parent_list(idx, [idx])
            matrices = [self._components[p].transform for p in parents] + [self.transform]
            for m in matrices:
                matrix = m * matrix
        return matrix

    def draw(self, shader=None, **kwargs):
        for idx, comp in enumerate(self._components):
            if comp.drawable:
                parents = self._parent_list(idx, [idx])
                matrices = [self._components[p].transform for p in parents] + [self.transform]
                comp.draw(matrices, shader=shader, **kwargs)

    def compute_energy(self, shader, light, viewport_size, depth_map):
        for idx, comp in enumerate(self._components):
            if comp.drawable:
                parents = self._parent_list(idx, [idx])
                matrices = [self._components[p].transform for p in parents] + [self.transform]
                comp.compute_energy(shader, light, viewport_size, depth_map, matrices)

    def convert_to_mesh(self):
        """ Converts a building to a single mesh that can be rendered more quickly.
        """
        vertices, v_offset = [], 0
        uvs, uv_offset = [], 0
        normals, n_offset = [], 0
        triangles, tri_offset = [], 0
        sub_mesh_dict = {}
        for idx, comp in enumerate(self._components):
            if comp.drawable:
                parents = self._parent_list(idx, [idx])
                matrices = [self._components[p].transform for p in parents] + [self.transform]
                transform = Mat4.identity()
                if matrices is not None:
                    for m in matrices:
                        transform = m * transform
                transform = np.array(transform)

                mesh = comp.mesh
                
                # Transform vertices with model matrix
                comp_vertices = (transform @ np.concatenate((mesh.vertices, np.ones_like(mesh.vertices[:, 0:1])), axis=1).T).T
                comp_vertices = comp_vertices[:, :3] / comp_vertices[:, 3:]
                vertices.append(comp_vertices)

                # Correctly transform the normals with the inverse transpose
                comp_normals = (np.linalg.inv(transform).T @ np.concatenate((mesh.vertex_normals, np.ones_like(mesh.vertex_normals[:, 0:1])), axis=1).T).T
                comp_normals = comp_normals[:, :3] / np.linalg.norm(comp_normals[:, :3], axis=1).clip(1e-5)
                normals.append(comp_normals)

                uvs.append(mesh.uvs)

                # Increment triangle indices with index offset
                comp_triangles = mesh.triangles
                n_triangles = len(comp_triangles)
                for tri in comp_triangles:
                    v_idx = tuple(idx + v_offset for idx in tri[0])
                    uv_idx = tuple(idx + uv_offset for idx in tri[1])
                    n_idx = tuple(idx + n_offset for idx in tri[2])
                    triangles += [(v_idx, uv_idx, n_idx)]

                # Create submesh or take over submeshes for each component
                if len(mesh.sub_meshes_raw) > 0:
                    for sub_mesh_texture in mesh.sub_meshes_raw:
                        sub_mesh, texture = sub_mesh_texture
                        triangle_idx = []
                        for f_i in sub_mesh.triangles:
                            start = mesh._triangulated_face_index[f_i]
                            end = n_triangles if f_i >= len(mesh._triangulated_face_index) - 1 else mesh._triangulated_face_index[f_i + 1]
                            triangle_idx += list(range(start + tri_offset, end + tri_offset))
                        sub_mesh.triangles = triangle_idx
                        if texture in sub_mesh_dict:
                            sub_mesh_dict[texture].append(sub_mesh)
                        else:
                            sub_mesh_dict[texture] = [sub_mesh]
                else:
                    sub_mesh = SubMesh(triangles=(np.arange(n_triangles) + tri_offset).tolist())
                    texture = mesh._texture_path
                    if texture in sub_mesh_dict:
                        sub_mesh_dict[texture].append(sub_mesh)
                    else:
                        sub_mesh_dict[texture] = [sub_mesh]

                # Offset for triangle references
                v_offset += mesh.vertices.shape[0]
                uv_offset += mesh.uvs.shape[0]
                n_offset += mesh.vertex_normals.shape[0]
                tri_offset += n_triangles

        # Create mesh
        mesh = Mesh(
            vertices=np.concatenate(vertices, axis=0).tolist(),
            colors=[Palette.GreenA.as_color()],
            normals=np.concatenate(normals, axis=0).tolist(),
            uvs=np.concatenate(uvs, axis=0).tolist(),
            triangles=triangles
        )

        # Add submeshes to mesh
        textures_sub_meshes = list(sub_mesh_dict.items())
        if len(textures_sub_meshes) > 0:
            texture, sub_meshes = textures_sub_meshes[0]
            sub_mesh = self.merge_sub_meshes(sub_meshes, name=texture)
            mesh.update_sub_mesh(0, sub_mesh, texture=texture)
        if len(textures_sub_meshes) > 1:
            for i in range(1, len(textures_sub_meshes)):
                texture, sub_meshes = textures_sub_meshes[i]
                sub_mesh = self.merge_sub_meshes(sub_meshes, name=texture)
                mesh.append_sub_mesh(sub_mesh, texture=texture)
        return mesh

    def merge_sub_meshes(self, sub_meshes, name=''):
        triangles = []
        for sub_mesh in sub_meshes:
            triangles += sub_mesh.triangles
        return SubMesh(triangles=triangles, name=name)
