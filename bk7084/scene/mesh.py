import collections.abc
import enum
import os
from dataclasses import dataclass

import numpy as np

from .loader.obj import WavefrontReader
from .. import gl
from collections import namedtuple
from ..assets import default_resolver

from ..geometry.shape import Shape
from ..graphics.array import VertexArrayObject
from ..graphics.buffer import VertexBuffer, IndexBuffer
from ..graphics.material import Material
from ..graphics.util import DrawingMode
from ..graphics.vertex_layout import VertexLayout, VertexAttrib, VertexAttribDescriptor, VertexAttribFormat
from ..math import Mat4
from ..misc import PaletteDefault, Color


# todo: face normals

class MeshTopology(enum.Enum):
    Triangles = gl.GL_TRIANGLES
    Lines = gl.GL_LINES
    LineStrip = gl.GL_LINE_STRIP
    Points = gl.GL_POINTS


MtlRenderRecord = namedtuple('MtlRenderRecord', ['mtl_idx', 'vao_idx', 'vbo_idx', 'vertex_count'])


@dataclass
class SubMeshDescriptor:
    name: str
    first_vertex: int
    vertex_count: int
    first_index: int
    index_count: int
    first_normal: int
    normal_count: int
    first_uv: int
    uv_count: int
    material: int
    topology: MeshTopology


class SubMesh:
    """
    Each sub-mesh corresponds to a Material. A sub-mesh consists of a list of triangles,
    which refer to a set of vertices(referenced by index). Vertices can be shared between
    multiple sub-meshes.

    Structure maintained inside of `Mesh` class.
    """
    def __init__(self,
                 mesh=None,
                 name='',
                 vertex_range=(-1, -1),
                 index_range=(-1, -1),
                 normal_range=(-1, -1),
                 uv_range=(-1, -1),
                 render_record=-1,
                 topology=MeshTopology.Triangles,
                 vertex_layout=VertexLayout.default()):
        self.mesh = mesh
        self.name: str = name
        self.vertex_count: int = vertex_range[1] - vertex_range[0]
        self.vertex_range: (int, int) = vertex_range

        self.index_count: int = index_range[1] - index_range[0]
        self.index_range: (int, int) = index_range

        self.normal_count: int = normal_range[1] - normal_range[0]
        self.normal_range: (int, int) = normal_range

        self.uv_count: int = uv_range[1] - uv_range[0]
        self.uv_range: (int, int) = uv_range

        # created and initialised by the Mesh class, should not be modified directly
        self.render_record: int = render_record

        self.topology: MeshTopology = topology
        self.vertex_layout: VertexLayout = vertex_layout

    def descriptor(self) -> SubMeshDescriptor:
        return SubMeshDescriptor(
            name=self.name,
            first_vertex=self.vertex_range[0],
            vertex_count=self.vertex_count,
            first_index=self.index_range[0],
            index_count=self.index_count,
            first_normal=self.normal_range[0],
            normal_count=self.normal_count,
            first_uv=self.uv_range[0],
            uv_count=self.uv_count,
            material=self.mesh.render_records[self.render_record].mtl_idx,
            topology=self.topology
        )

    def is_same(self, desc: SubMeshDescriptor):
        return self.name == desc.name and \
               self.vertex_count == desc.vertex_count and self.vertex_range[0] == desc.first_vertex and \
               self.index_count == desc.index_count and self.index_range[0] == desc.first_index and \
               self.normal_count == desc.normal_count and self.normal_range[0] == desc.first_normal and \
               self.uv_count == desc.uv_count and self.uv_range[0] == desc.first_uv and \
               self.mesh.render_records[self.render_record].mtl_idx == desc.material and \
               self.topology == desc.topology


class Mesh:
    """
    Representation of a generic geometry including vertex positions, face indices, normals, colors, uvs,
    and custom attributes within buffers.

    Mesh contains vertices and multiple triangle arrays. The triangle arrays are indices into the vertex
    arrays; three indices for each triangle.

    For each vertex there can be a normal, eight texture coordinates, color, tangent, and bone weight.
    These are optional and can be removed. All vertex information is stored in separate arrays of the
    same size, so if your mesh has 10 vertices, you would also have 10-size arrays for normals and other
    attributes.

    The mesh face data (triangles) is made of three vertex indices for each triangle.

    Accepts numpy array as input:
    vertices = np.array([((-0.5, -0.5, 0.0), (0.933, 0.376, 0.333)),
                     ((0.5, -0.5, 0.0), (0.376, 0.827, 0.580)),
                     ((0.0, 0.5, 0.0), (0.000, 0.686, 0.725))],
                    dtype=[('position', (np.float32, 3)), ('color', (np.float32, 3))])
    """

    DEFAULT_COLOR = PaletteDefault.BrownB.as_color()

    def __init__(self, filepath=None, load_immediately=True, resolver=default_resolver, **kwargs):
        """
        Creation and initialisation of a Mesh. A mesh can be created by:

        1. reading a OBJ file
        2. compositing geometry shapes
        3. providing vertices and its attributes

        Note:
            If `filepath` is specified, this function will skip arguments inside of `kwargs`.

        Args:
            filepath (str):
                Specifies the path of the model file to be loaded.

            load_immediately (bool):
                Whether to load immediately the mesh when initialised by a OBJ file.

            **kwargs:
                vertices: (array like):
                    Specifies the position of vertices.

                colors: (Sequence[Color]):
                    Specifies the color of vertices.

                normals: (array like):
                    Specifies the normal of vertices.

                triangles: (array like):
                    Specifies the indices to create triangles.

                shapes (list, tuple):
                    Specifies a sequence of shapes used to construct a mesh.
        """
        self._vertex_layout = VertexLayout(
            (VertexAttrib.Position, VertexAttribFormat.Float32, 3),
            (VertexAttrib.Color0, VertexAttribFormat.Float32, 4),
            (VertexAttrib.TexCoord0, VertexAttribFormat.Float32, 2),
            (VertexAttrib.Normal, VertexAttribFormat.Float32, 3),
        )

        self._vertex_count = 0
        self._index_count = 0
        self._triangle_count = 0

        self._vertices = np.array([], dtype=np.float32)  # vertices
        self._colors = np.array([], dtype=np.float32)  # vertex colors
        self._uvs = np.array([], dtype=np.float32)  # vertex texture coordinates
        self._normals = np.array([], dtype=np.float32)  # vertex normals
        self._triangles = np.array([], dtype=np.uint32)  # indices for draw whole mesh

        self._materials = [Material.default()]  # loaded materials from OBJ file
        self._render_records = {}  # records the rendering information
        self._faces = []  # [(vertices indices), (vertex uvs indices), ( vertex normals indices)]
        self._sub_meshes = []
        self._sub_mesh_count = 0

        self._initial_transformation: Mat4 = Mat4.identity()
        self._transformation: Mat4 = Mat4.identity()

        self._shading_enabled = True
        self._texture_enabled = None

        self._vertex_buffers: [VertexBuffer] = []
        self._index_buffers: [IndexBuffer] = []
        self._vertex_array_objects: [VertexArrayObject] = []

        shapes = kwargs.get('shapes', None)
        vertices = kwargs.get('vertices', None)
        colors = kwargs.get('colors', [Mesh.DEFAULT_COLOR])
        uvs = kwargs.get('uvs', None)
        normals = kwargs.get('normals', None)
        triangles = kwargs.get('triangles', None)

        if filepath:
            self._name = str(filepath)
            if load_immediately:
                self._read_from_file(filepath, colors, resolver)
            else:
                self._tmp_colors = colors
        elif shapes is not None:
            self._name = 'unnamed_{}'.format(self.__class__.__name__)
            self._from_shapes(shapes)
        else:
            # fill vertices
            self._vertices = np.asarray(vertices, dtype=np.float32).ravel()
            self._vertex_count = self._vertices / 3
            # fill colors
            self._colors = Mesh._process_color(self._vertex_count, colors)
            # fill uvs
            self._uvs = np.asarray(uvs, dtype=np.float32).ravel()
            # fill normals
            self._normals = np.asarray(normals, dtype=np.float32).ravel()
            # fill triangles
            self._triangles = np.asarray(triangles, dtype=np.uint32).ravel()
            self._index_count = len(self._triangles)
            self._triangle_count = self._index_count / 3
            self._sub_mesh_count = 1
            self._sub_meshes = [
                SubMesh()
            ]

    @staticmethod
    def _process_color(count, colors):
        output_colors = None
        if isinstance(colors, Color):
            output_colors = np.tile(colors.rgba, (count, 1))
        elif isinstance(colors, collections.abc.Sequence):
            colors_count = len(colors)
            if colors_count == 1:
                output_colors = np.tile([colors[0].rgba], (count, 1))
            elif colors_count >= count:
                output_colors = np.asarray([c.rgba for c in colors[:count]]).reshape((count, 4))
            else:
                num = count - colors_count
                output_colors = np.asarray([c.rgba for color in [colors, [Mesh.DEFAULT_COLOR] * num] for c in color]).reshape((count, 4))
        return output_colors

    @property
    def shading_enabled(self):
        return self._shading_enabled

    @shading_enabled.setter
    def shading_enabled(self, value):
        self._shading_enabled = value

    @property
    def texture_enabled(self):
        return self._texture_enabled

    @texture_enabled.setter
    def texture_enabled(self, value):
        self._texture_enabled = value

    def load(self):
        if os.path.exists(self._name):
            self._read_from_file(self._name, self._tmp_colors)

    def _read_from_file(self, filepath, color, resolver=default_resolver):
        reader = WavefrontReader(filepath, resolver)
        mesh_data = reader.read()

        self._vertex_count = len(mesh_data['vertices'])
        self._vertices = np.asarray(mesh_data['vertices'], dtype=np.float32).reshape((-1, 3))
        self._normals = np.asarray(mesh_data['normals'], dtype=np.float32).reshape((-1, 3))
        self._faces = mesh_data['faces']
        self._triangles = np.asarray([f[0] for f in mesh_data['faces']], dtype=np.uint32).ravel()
        self._index_count = len(self._triangles)
        self._uvs = np.asarray(mesh_data['texcoords'], dtype=np.float32).reshape((-1, 2))
        self._colors = Mesh._process_color(self._vertex_count, color)

        for name, mat in mesh_data['materials'].items():
            self._materials.append(
                Material(
                    name,
                    mat.get('map_Kd', None),
                    mat.get('Ka', [1.0, 1.0, 1.0]),
                    mat.get('Kd', [1.0, 1.0, 1.0]),
                    mat.get('Ks', [1.0, 1.0, 1.0]),
                    mat.get('Ns', 0),
                    mat.get('Ni', 1.0),
                    mat.get('d', 1.0),
                    mat.get('illum', 0.0)
                )
            )

        for sub_obj in mesh_data['objects']:
            sub_obj_materials = []
            for mtl in sub_obj['materials']:
                mtl_name = mtl['name']
                f_start, f_end = tuple(mtl['f_range'])
                mtl_idx = self._materials.index([x for x in self._materials if x.name == mtl_name][0])
                if mtl_idx != -1 and mtl_idx < len(self._materials):
                    sub_obj_materials.append((mtl_idx, f_start, f_end))

            # will populate a default color
            color = self._colors[0]
            vertex_attribs = [VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3),
                              VertexAttribDescriptor(VertexAttrib.Color0, VertexAttribFormat.Float32, 4)]

            for attrib in sub_obj['vertex_format']:
                if attrib == 'T':
                    vertex_attribs.append(VertexAttribDescriptor(VertexAttrib.TexCoord0, VertexAttribFormat.Float32, 2))
                if attrib == 'N':
                    vertex_attribs.append(VertexAttribDescriptor(VertexAttrib.Normal, VertexAttribFormat.Float32, 3))

            # pack the data together for rendering
            for mtl in sub_obj_materials:
                sub_mesh = SubMesh(
                    name=f"{sub_obj['name']}-{self._materials[mtl[0]].name}",
                    vertex_range=sub_obj['vertex_range'],
                    index_range=sub_obj['index_range'],
                    normal_range=sub_obj['normal_range'],
                    uv_range=sub_obj['texcoord_range']
                )

                sub_mesh.vertex_layout = VertexLayout(*vertex_attribs)

                mtl_idx, f_start, f_end = mtl
                v_positions = []
                v_normals = []
                v_texcoords = []
                for face in self._faces[f_start: f_end]:
                    i = 0
                    for v_i in face[i]:
                        v_positions.append(self._vertices[v_i])
                    i += 1
                    if sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0):
                        for vt_i in face[i]:
                            v_texcoords.append(self._uvs[vt_i])
                        i += 1
                    if sub_mesh.vertex_layout.has(VertexAttrib.Normal):
                        for vn_i in face[i]:
                            v_normals.append(self._normals[vn_i])

                # create one interleaved buffer
                vertex_count = len(v_positions)
                v_colors = [color] * vertex_count

                if sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and sub_mesh.vertex_layout.has(VertexAttrib.Normal):
                    vertices = np.array([z for x in zip(v_positions, v_colors, v_texcoords, v_normals) for y in x for z in y],
                                        dtype=np.float32).ravel()
                elif sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and not sub_mesh.vertex_layout.has(VertexAttrib.Normal):
                    vertices = np.array([z for x in zip(v_positions, v_colors, v_texcoords) for y in x for z in y],
                                        dtype=np.float32).ravel()
                elif not sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and sub_mesh.vertex_layout.has(VertexAttrib.Normal):
                    vertices = np.array([z for x in zip(v_positions, v_colors, v_normals) for y in x for z in y],
                                        dtype=np.float32).ravel()
                else:
                    vertices = np.array([z for x in zip(v_positions, v_colors) for y in x for z in y],
                                        dtype=np.float32).ravel()

                # bind vertex buffers
                vao = VertexArrayObject()
                vbo = VertexBuffer(vertex_count, sub_mesh.vertex_layout)
                vbo.set_data(vertices)

                vbo_idx = len(self._vertex_buffers)
                self._vertex_buffers.append(vbo)
                vao_idx = len(self._vertex_array_objects)
                self._vertex_array_objects.append(vao)

                if not sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0):
                    vao.bind_vertex_buffer(vbo, [0, 1, 3])
                else:
                    vao.bind_vertex_buffer(vbo, [0, 1, 2, 3])

                sub_mesh.render_record = len(self._render_records.keys())
                self._render_records[sub_mesh.render_record] = MtlRenderRecord(mtl_idx, vao_idx, vbo_idx, vertex_count)

                self._sub_meshes.append(sub_mesh)

        self._sub_mesh_count = len(self._sub_meshes)
        self._texture_enabled = True

    def _from_shapes(self, *shapes):
        """Construct a mesh from a collection of geometry objects."""
        # todo: improve performance by merge buffers of objects with same draw type or of the same type
        if len(shapes) == 0:
            raise ValueError('Geometry objects are empty when trying to construct a mesh.')

        for shape in shapes:
            # todo
            pass

    @property
    def vertex_layout(self) -> VertexLayout:
        return self._vertex_layout

    @vertex_layout.setter
    def vertex_layout(self, layout: VertexLayout):
        self._vertex_layout = layout
        # TODO: update vertex attribute pointer

    @property
    def vertex_attrib_count(self):
        """Returns the number of vertex attributes that the mesh has."""
        return self._vertex_layout.attrib_count

    @property
    def vertex_buffer_count(self):
        """Returns the number of vertex buffers present in the mesh."""
        return len(self._vertex_buffers)

    @property
    def vertices(self):
        """Returns the vertex positions."""
        raise self._vertices

    @vertices.setter
    def vertices(self, positions):
        self._vertices = np.asarray(positions, dtype=np.float32).ravel()
        self._vertex_count = len(self._vertices) / 3

    @property
    def vertex_count(self):
        """Vertex count of the mesh."""
        return self._vertex_count

    @property
    def colors(self):
        """Vertex colors of the mesh."""
        return self._colors

    @colors.setter
    def colors(self, colors):
        """Vertex colors of the mesh."""
        self._colors = np.asarray(colors, dtype=np.float32).ravel()

    @property
    def vertex_normals(self):
        """Vertex normals of the mesh."""
        return self._normals

    @vertex_normals.setter
    def vertex_normals(self, normals):
        """Vertex normals of the mesh."""
        self._normals = normals

    @property
    def materials(self):
        return self._materials

    @property
    def material_count(self):
        return len(self._materials)

    @property
    def render_records(self):
        return self._render_records

    @property
    def sub_meshes(self):
        return [sm.descriptor() for sm in self._sub_meshes]

    def set_sub_mesh(self, index, **kwargs):
        if 0 <= index < self.sub_mesh_count:
            self._update_sub_mesh(index, **kwargs)

    def add_sub_mesh(self, descriptor):
        self._append_sub_mesh(descriptor)

    @property
    def sub_mesh_count(self):
        return self._sub_mesh_count

    def _update_sub_mesh(self, index, desc: SubMeshDescriptor):
        sub_mesh = self._sub_meshes[index]
        if not sub_mesh.is_same(desc):
            sub_mesh.name = desc.name
            sub_mesh.vertex_count = desc.vertex_count
            sub_mesh.vertex_range = (desc.first_vertex, desc.first_vertex + desc.vertex_count)
            sub_mesh.index_count = desc.index_count
            sub_mesh.index_range = (desc.first_index, desc.first_index + desc.index_count)
            sub_mesh.normal_count = desc.normal_count
            sub_mesh.normal_range = (desc.first_normal, desc.first_normal + desc.normal_count)
            sub_mesh.uv_count = desc.uv_count
            sub_mesh.uv_range = (desc.first_uv, desc.first_uv + desc.uv_count)
            sub_mesh.topology = desc.topology

            old_record = self._render_records[sub_mesh.render_record]
            # destroy old vbo
            self._vertex_buffers[old_record.vbo_idx].delete()
            self._vertex_array_objects[old_record.vao_idx].delete()
            # create new vbo with new data
            v_positions = self._vertices[sub_mesh.vertex_range[0]: sub_mesh.vertex_range[1]]
            v_normals = self._normals[sub_mesh.normal_range[0]: sub_mesh.normal_range[1]]
            v_uvs = self._uvs[sub_mesh.uv_range[0]: sub_mesh.uv_range[1]]
            v_colors = self._colors[sub_mesh.vertex_range[0]: sub_mesh.vertex_range[1]]

            vertices = np.array([z for x in zip(v_positions, v_colors, v_uvs, v_normals) for y in x for z in y],
                                dtype=np.float32).ravel()

            vao = self._vertex_array_objects[old_record.vao_idx]
            vbo = VertexBuffer(sub_mesh.vertex_count, sub_mesh.vertex_layout)
            vbo.set_data(vertices)

            vbo_idx = len(self._vertex_buffers)
            self._vertex_buffers.append(vbo)

            # update vao
            if not sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0):
                vao.bind_vertex_buffer(vbo, [0, 1, 3])
            else:
                vao.bind_vertex_buffer(vbo, [0, 1, 2, 3])

            # update with the new record
            self._render_records[sub_mesh.render_record] = MtlRenderRecord(old_record.mtl_idx,
                                                                           old_record.vao_idx,
                                                                           vbo_idx,
                                                                           sub_mesh.vertex_count)

    def _append_sub_mesh(self, descriptor):
        self._sub_meshes.append(SubMesh())
        self._update_sub_mesh(len(self._sub_meshes), descriptor)

    def apply_transformation(self, matrix: Mat4):
        self._transformation = matrix * self._transformation
        return self

    def then(self, matrix: Mat4):
        self._transformation = matrix * self._transformation
        return self

    def reset_transformation(self):
        self._transformation = Mat4.identity()

    @property
    def transformation(self):
        return self._transformation

    @transformation.setter
    def transformation(self, value: Mat4):
        self._transformation = value

    @property
    def initial_transformation(self):
        return self._initial_transformation

    @initial_transformation.setter
    def initial_transformation(self, value: Mat4):
        self._initial_transformation = value

    def draw_with_shader(self, shader):
        if len(self._sub_meshes) > 0:
            with shader:
                mat = self._transformation * self._initial_transformation
                shader.model_mat = mat
                shader['shading_enabled'] = self._shading_enabled
                for sub_mesh in self._sub_meshes:
                    record = self._render_records[sub_mesh.render_record]
                    mtl = self._materials[record.mtl_idx]
                    vao = self._vertex_array_objects[record.vao_idx]

                    shader['mtl.diffuse'] = mtl.diffuse
                    shader['mtl.ambient'] = mtl.ambient
                    shader['mtl.specular'] = mtl.specular
                    shader['mtl.shininess'] = mtl.glossiness
                    shader['mtl.enabled'] = True
                    from bk7084.app import current_window
                    shader['time'] = current_window().elapsed_time
                    shader['mtl.use_diffuse_map'] = self._texture_enabled

                    shader.active_texture_unit(0)
                    mtl.texture_diffuse.bind()

                    with vao:
                        gl.glDrawArrays(sub_mesh.topology.value, 0, record.vertex_count)
