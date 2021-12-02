import ctypes
from dataclasses import dataclass

import numpy as np

from .loader.obj import WavefrontReader
from .. import gl
from collections import namedtuple
from ..geometry.shape import Shape
from ..graphics.array import VertexArrayObject
from ..graphics.buffer import VertexBuffer, IndexBuffer
from ..graphics.material import Material
from ..graphics.util import DrawingMode
from ..graphics.vertex_layout import VertexLayout, VertexAttrib, VertexAttribDescriptor, VertexAttribFormat
from ..math import Mat4


# if num_vertices == num_tex_coords then use glDrawElements
# if num_vertices == num_tex_coords then pack all the data inside of a buffer and use glDrawArrays


class MeshTopology:
    Triangles = 0
    Lines = 1
    LineStrip = 2
    Points = 3


@dataclass
class SubMesh:
    # index where to find the vao inside of a mesh's vao list.
    vao_idx: int = -1
    vbo_idx: int = -1
    drawing_mode: DrawingMode = None
    vertex_count: int = 0
    index_count: int = 0
    vertices_range: (int, int) = (0, 0)
    indices_range: (int, int) = (0, 0)
    colors_range: (int, int) = (0, 0)
    normals_range: (int, int) = (0, 0)


MtlRenderRecord = namedtuple('MtlRenderRecord', ['mtl_idx', 'vao_idx', 'vbo_idx', 'vertex_count'])


class SubMesh2:
    def __init__(self):
        self.name: str = ''
        self.drawing_mode: DrawingMode = DrawingMode.Triangles
        self.vertex_count: int = 0
        self.vertex_range: (int, int) = (-1, -1)

        self.index_count: int = 0
        self.index_range: (int, int) = (-1, -1)

        self.normal_count: int = 0
        self.normal_range: (int, int) = (-1, -1)

        self.texcoord_count: int = 0
        self.texcoord_range: (int, int) = (-1, -1)

        self.material_count: int = 0
        self.material_records: list[MtlRenderRecord] = []

        self.vertex_layout: VertexLayout = None


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

    def __init__(self, filepath=None, **kwargs):
        """
        Args:
            **kwargs:
                shapes (list, tuple):
                    Specifies a sequence of shapes used to construct a mesh.
        """
        # self._bounding_box = None
        # self._bounding_sphere = None
        self._vertex_layout = VertexLayout.default()
        self._name = filepath

        self._color = kwargs.get('color', None)
        self._positions = np.array([], dtype=np.float32)
        self._indices = np.array([], dtype=np.uint32)  # indices for draw whole mesh without apply materials
        self._materials = [Material.default()]
        self._normals = np.array([], dtype=np.float32)
        self._texcoords = np.array([], dtype=np.float32)
        self._faces = []

        self._face_normals = np.array([], dtype=np.float32)

        self._sub_meshes = []

        self._default_material = None  # todo: default material

        self._vertex_count = 0
        self._index_count = 0

        self._vertex_buffers: [VertexBuffer] = []
        self._index_buffers: [IndexBuffer] = []
        self._vertex_array_objects: [VertexArrayObject] = []

        self._initial_transformation: Mat4 = Mat4.identity()
        self._transformation: Mat4 = Mat4.identity()
        self._do_shading = False
        self._contain_geometries = False

        if filepath is not None:
            self.read_from_file(filepath)
            # self.read_from_file(filepath, kwargs.get('color', PaletteDefault.BrownB.as_color()))
            self._contain_geometries = False
        else:
            shapes = kwargs.get('shapes', None)
            if shapes is not None:
                self.from_geometry(*shapes)
                self._contain_geometries = True

    def read_from_file(self, filepath, color=None):
        reader = WavefrontReader(filepath)
        mesh_data = reader.read()

        self._vertex_count = len(mesh_data['vertices'])
        self._positions = np.asarray(mesh_data['vertices'], dtype=np.float32).reshape((-1, 3))
        self._normals = np.asarray(mesh_data['normals'], dtype=np.float32).reshape((-1, 3))
        self._faces = mesh_data['faces']
        self._indices = np.asarray([f[0] for f in mesh_data['faces']], dtype=np.uint32).ravel()
        self._index_count = len(self._indices)
        self._texcoords = np.asarray(mesh_data['texcoords'], dtype=np.float32).reshape((-1, 2))

        for name, mat in mesh_data['materials'].items():
            self._materials.append(
                Material(
                    name,
                    mat.get('map_Kd', None),
                    mat.get('Ka', [1.0, 1.0, 1.0]),
                    mat.get('Kd', [1.0, 1.0, 1.0]) if color is None else color.rgb(),
                    mat.get('Ks', [1.0, 1.0, 1.0]),
                    mat.get('Ns', 0),
                    mat.get('Ni', 1.0),
                    mat.get('d', 1.0),
                    mat.get('illum', 0.0)
                )
            )

        for sub_obj in mesh_data['objects']:
            sub_mesh = SubMesh2()
            sub_mesh.name = sub_obj['name']
            sub_mesh.drawing_mode = DrawingMode.Triangles
            sub_mesh.vertex_range = sub_obj['vertex_range']
            sub_mesh.vertex_count = sub_mesh.vertex_range[1] - sub_mesh.vertex_range[0]
            sub_mesh.index_range = sub_obj['index_range']
            sub_mesh.index_count = sub_mesh.index_range[1] - sub_mesh.index_range[0]
            sub_mesh.normal_range = sub_obj['normal_range']
            sub_mesh.normal_count = sub_mesh.normal_range[1] - sub_mesh.normal_range[0]
            sub_mesh.texcoord_range = sub_obj['texcoord_range']
            sub_mesh.texcoord_count = sub_mesh.texcoord_range[1] - sub_mesh.texcoord_range[0]
            sub_mesh.material_count = len(sub_obj['materials'])

            sub_obj_materials = []
            for mtl in sub_obj['materials']:
                mtl_name = mtl['name']
                f_start, f_end = tuple(mtl['f_range'])
                mtl_idx = self._materials.index([x for x in self._materials if x.name == mtl_name][0])
                if mtl_idx != -1 and mtl_idx < len(self._materials):
                    sub_obj_materials.append((mtl_idx, f_start, f_end))

            vertex_attribs = []
            for attrib in sub_obj['vertex_format']:
                if attrib == 'V':
                    vertex_attribs.append(VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3))
                if attrib == 'T':
                    vertex_attribs.append(VertexAttribDescriptor(VertexAttrib.TexCoord0, VertexAttribFormat.Float32, 2))
                if attrib == 'N':
                    vertex_attribs.append(VertexAttribDescriptor(VertexAttrib.Normal, VertexAttribFormat.Float32, 3))

            sub_mesh.vertex_layout = VertexLayout(*vertex_attribs)

            # pack the data together for rendering
            for mtl in sub_obj_materials:
                mtl_idx, f_start, f_end = mtl
                v_positions = []
                v_normals = []
                v_texcoords = []
                for face in self._faces[f_start: f_end]:
                    i = 0
                    for v_i in face[i]:
                        v_positions.append(self._positions[v_i])
                    i += 1
                    if sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0):
                        for vt_i in face[i]:
                            v_texcoords.append(self._texcoords[vt_i])
                        i += 1
                    if sub_mesh.vertex_layout.has(VertexAttrib.Normal):
                        for vn_i in face[i]:
                            v_normals.append(self._normals[vn_i])

                # create one interleaved buffer
                vertex_count = len(v_positions)
                vertices = np.array([z for x in zip(v_positions, v_texcoords, v_normals) for y in x for z in y],
                                    dtype=np.float32).ravel()

                # bind vertex buffers
                vao = VertexArrayObject()
                vbo = VertexBuffer(vertex_count, sub_mesh.vertex_layout)
                vbo.set_data(vertices)

                vbo_idx = len(self._vertex_buffers)
                self._vertex_buffers.append(vbo)
                vao_idx = len(self._vertex_array_objects)
                self._vertex_array_objects.append(vao)

                vao.bind_vertex_buffer(vbo, attrib_locations=[0, 2, 3])

                record = MtlRenderRecord(mtl_idx, vao_idx, vbo_idx, vertex_count)
                sub_mesh.material_records.append(record)

            self._sub_meshes.append(sub_mesh)

        self._do_shading = True

    # def read_from_file(self, filepath, color):
    #     trimesh.util.attach_to_log()
    #     mesh = trimesh.load(filepath, force='mesh', skip_texture=False)
    #
    #     # texture = mesh.visual.to_texture()
    #
    #     # resolver = trimesh.visual.resolvers.FilePathResolver(filepath)
    #     # with open(filepath, 'r') as f:
    #     #     content = trimesh.exchange.obj.load_obj(f, resolver=resolver)
    #     #
    #     # for k, v in content['geometry'].items():
    #     #     print(k)
    #
    #     self._vertex_count = len(mesh.vertices)
    #     self._positions = mesh.vertices.ravel()
    #     self._normals = mesh.vertex_normals.ravel()
    #
    #     self._vertex_array_objects.append(VertexArrayObject())
    #     self._indices = np.asarray(mesh.faces, dtype=np.uint32).ravel()
    #     self._index_count = len(self._indices)
    #     self._colors = np.tile(color.rgba, self._vertex_count).ravel()
    #
    #     # todo: deal with texture, normal ...
    #     self._sub_meshes.append(SubMesh(vertices_range=(0, len(self._positions)),
    #                                     indices_range=(0, len(self._indices)),
    #                                     vao_idx=0,
    #                                     colors_range=(0, len(self._colors)),
    #                                     drawing_mode=DrawingMode.Triangles,
    #                                     vertex_count=self._vertex_count,
    #                                     index_count=self._index_count,
    #                                     normals_range=(0, len(self._normals))))
    #
    #     self._index_buffer = IndexBuffer(self._index_count)
    #     self._index_buffer.set_data(self._indices)
    #
    #     # todo: generate layout from object file vertex format
    #     self._vertex_layout = VertexLayout(
    #         VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3),
    #         VertexAttribDescriptor(VertexAttrib.Color0, VertexAttribFormat.Float32, 4),
    #         VertexAttribDescriptor(VertexAttrib.Normal, VertexAttribFormat.Float32, 3)
    #     )
    #
    #     vertices = np.zeros(10 * self._sub_meshes[0].vertex_count, dtype=np.float32)
    #     # iterate over each vertex
    #     for i in range(0, self._sub_meshes[0].vertex_count):
    #         index = i * 10
    #         vertices.put(list(range(index, index + 3)),
    #                      self._positions[i * 3: i * 3 + 3])
    #         vertices.put(list(range(index + 3, index + 7)),
    #                      self._colors[i * 4: i * 4 + 4])
    #         vertices.put(list(range(index + 7, index + 10)),
    #                      self._normals[i * 3: i * 3 + 3])
    #
    #     vbo = VertexBuffer(self._sub_meshes[0].vertex_count, self._vertex_layout)
    #     self._vertex_buffers.append(vbo)
    #     vbo.set_data(vertices)
    #     self._sub_meshes[0].vbo_idx = len(self._vertex_buffers) - 1
    #
    #     vao = self._vertex_array_objects[self._sub_meshes[0].vao_idx]
    #     vao.bind_vertex_buffer(vbo)
    #
    #     self._do_shading = True

    def from_geometry(self, *shapes):
        """Construct a mesh from a collection of geometry objects."""
        # todo: improve performance by merge buffers of objects with same draw type or of the same type
        if len(shapes) == 0:
            raise ValueError('Geometry objects are empty when trying to construct a mesh.')

        if not isinstance(shapes[0], Shape):
            raise ValueError('Can construct mesh from non-geometry object(s).')

        for shape in shapes:
            self._vertex_array_objects.append(VertexArrayObject())

            self._vertex_count += shape.vertex_count
            self._index_count += shape.index_count
            v_range = (len(self._positions), (len(self._positions) + shape.vertex_count * 3))
            i_range = (len(self._indices), (len(self._indices) + shape.index_count))
            c_range = (len(self._colors), (len(self._colors) + shape.vertex_count * 4))

            sub_mesh = SubMesh(vertices_range=v_range,
                               indices_range=i_range,
                               vao_idx=len(self._vertex_array_objects) - 1,
                               colors_range=c_range,
                               drawing_mode=shape.drawing_mode,
                               vertex_count=shape.vertex_count,
                               index_count=shape.index_count)

            # fill the data
            self._positions = np.concatenate([self._positions, shape.vertices.astype(np.float32)])
            self._indices = np.concatenate([self._indices, shape.indices.astype(np.uint32)])
            self._colors = np.concatenate([self._colors, shape.colors.astype(np.float32)])

            self._sub_meshes.append(sub_mesh)

        self._index_buffer = IndexBuffer(self._index_count)
        self._index_buffer.set_data(self._indices)

        self._vertex_layout = VertexLayout(
            VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3),
            VertexAttribDescriptor(VertexAttrib.Color0, VertexAttribFormat.Float32, 4),
        )

        for sub_mesh in self._sub_meshes:
            # todo: deal with different vertex layout
            vertices = np.zeros(7 * sub_mesh.vertex_count, dtype=np.float32)
            position_offset = sub_mesh.vertices_range[0]
            color_offset = sub_mesh.colors_range[0]
            for i in range(0, sub_mesh.vertex_count):
                index = i * 7
                vertices.put(list(range(index, index + 3)),
                             self._positions[i * 3 + position_offset: i * 3 + 3 + position_offset])
                vertices.put(list(range(index + 3, index + 7)),
                             self._colors[i * 4 + color_offset: i * 4 + 4 + color_offset])

            vbo = VertexBuffer(sub_mesh.vertex_count, self._vertex_layout)
            self._vertex_buffers.append(vbo)
            vbo.set_data(vertices)
            sub_mesh.vbo_idx = len(self._vertex_buffers) - 1

            vao = self._vertex_array_objects[sub_mesh.vao_idx]
            vao.bind_vertex_buffer(vbo)

        self._do_shading = False

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
        return NotImplementedError

    @property
    def vertices(self):
        """Returns the vertex positions."""
        raise NotImplementedError

    @vertices.setter
    def vertices(self, new_positions):
        raise NotImplementedError

    @property
    def vertex_count(self):
        """Vertex count of the mesh."""
        return NotImplementedError

    @property
    def face_normals(self):
        """Triangle face normals of the mesh."""
        raise NotImplementedError

    @property
    def vertex_normals(self):
        """Vertex normals of the mesh."""
        return NotImplementedError

    @property
    def tangents(self):
        raise NotImplementedError

    @property
    def colors(self):
        """Vertex colors of the mesh."""
        raise NotImplementedError

    def set_attribute(self, attrib: VertexAttrib, data):
        pass

    @property
    def sub_meshes(self):
        return self._sub_meshes

    # @property
    # def bounding_box(self):
    #     return self._bounding_box
    #
    # @property
    # def bounding_sphere(self):
    #     return self._bounding_sphere
    # def compute_bounding_box(self):
    #     """Computes bounding box of the geometry.
    #
    #     Note:
    #         Bounding boxes are not computed by default.
    #     """
    #     raise NotImplementedError
    #
    # def compute_bounding_sphere(self):
    #     """Computes bounding sphere of the geometry.
    #
    #     Note:
    #         Bounding spheres are not computed by default.
    #     """
    #     raise NotImplementedError
    #
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

    #
    # def compute_tangents(self):
    #     """Calculates and adds a tangent attribute to the geometry."""
    #     raise NotImplementedError
    #
    # def compute_vertex_normals(self):
    #     """Computes vertex normals by averaging face normals.
    #     """
    #     raise NotImplementedError
    #
    # def compute_face_normals(self):
    #     """Computes triangle face normals by averaging vertex normals.
    #     """
    #     raise NotImplementedError
    #

    def draw_with_shader(self, shader):
        if len(self._sub_meshes) > 0:
            with shader:
                mat = self._transformation * self._initial_transformation
                shader.model_mat = mat
                shader.do_shading = int(self._do_shading)
                for sub_mesh in self._sub_meshes:
                    for record in sub_mesh.material_records:
                        mtl = self._materials[record.mtl_idx]
                        vao = self._vertex_array_objects[record.vao_idx]
                        shader['mtl.diffuse'] = mtl.diffuse
                        shader['mtl.ambient'] = mtl.ambient
                        shader['mtl.specular'] = mtl.specular
                        shader['mtl.shininess'] = mtl.glossiness
                        shader['mtl.enabled'] = True
                        if self._color is None:
                            shader['mtl.use_diffuse_map'] = True
                        else:
                            if not mtl.is_default:
                                shader['mtl.use_diffuse_map'] = True
                            else:
                                shader['mtl.use_diffuse_map'] = False

                        shader['shading_enabled'] = True
                        shader['toon_enabled'] = False
                        shader.active_texture_unit(0)
                        mtl.texture_diffuse.bind()
                        with vao:
                            gl.glDrawArrays(sub_mesh.drawing_mode.value, 0, record.vertex_count)
