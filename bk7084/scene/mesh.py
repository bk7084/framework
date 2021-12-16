import collections.abc
import enum
import os
from collections import namedtuple

import numpy as np

from .loader.obj import WavefrontReader
from .. import gl
from ..assets import default_resolver, default_asset_mgr
from ..graphics.array import VertexArrayObject
from ..graphics.buffer import VertexBuffer, IndexBuffer
from ..graphics.vertex_layout import VertexLayout, VertexAttrib, VertexAttribDescriptor, VertexAttribFormat
from ..math import Mat4, Vec3
from ..misc import PaletteDefault, Color


# todo: face normals

class MeshTopology(enum.Enum):
    Triangles = gl.GL_TRIANGLES
    Lines = gl.GL_LINES
    LineStrip = gl.GL_LINE_STRIP
    Points = gl.GL_POINTS


MtlRenderRecord = namedtuple('MtlRenderRecord', ['mtl_idx', 'vao_idx', 'vbo_idx', 'vertex_count'])


class SubMesh:
    """
    Each sub-mesh corresponds to a Material. A sub-mesh consists of a list of triangles,
    which refer to a set of vertices(referenced by index). Vertices can be shared between
    multiple sub-meshes.

    Structure maintained inside of `Mesh` class.
    """

    def __init__(self,
                 triangles=None,
                 name='',
                 topology=MeshTopology.Triangles,
                 vertex_layout=VertexLayout.default(),
                 normal_map_enabled=False):
        self.name: str = name
        self.triangles: [] = [] if triangles is None else triangles  # index of faces
        self.vertex_count: int = len(triangles) * 3 if triangles is not None else 0
        self.topology: MeshTopology = topology
        self.vertex_layout: VertexLayout = vertex_layout
        self.normal_map_enabled: bool = normal_map_enabled


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

    todo:
      1. calculate normals
      2. calculate tangent
      3. calculate bitangent
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

                uvs: (array like):
                    Specifies the uv coordinates of the vertices.

                texture: (str):
                    Specifies the texture image file.

                shapes (list, tuple):
                    Specifies a sequence of shapes used to construct a mesh.

                shader (ShaderProgram):
                    Specifies the shader going to be used for rendering. In case of non specified,
                    use framework's default shader (declared inside of Window).
        """
        self._vertex_layout = VertexLayout.default()

        self._vertex_count = 0
        self._index_count = 0
        self._triangle_count = 0
        self._normal_count = 0
        self._uv_count = 0

        self._vertices = np.array([], dtype=np.float32)  # vertices
        self._colors = np.array([], dtype=np.float32)  # vertex colors
        self._uvs = np.array([], dtype=np.float32)  # vertex texture coordinates
        self._normals = np.array([], dtype=np.float32)  # vertex normals
        self._indices = np.array([], dtype=np.uint32)  # indices for draw whole mesh
        self._tangents = np.array([], dtype=np.float32)  # vertex tangents

        # todo: clarify the input faces and triangles
        self._faces = []  # [(vertices indices), (vertex uvs indices), ( vertex normals indices)]
        self._triangulated_face_index = []  # stores the index of triangles of a face
        self._triangles = []
        self._vertex_triangles = []  # the triangle indices that correspond to each vertex

        self._materials = [default_asset_mgr.get_or_create_material('default_material')]
        texture_path = kwargs.get('texture', None)
        self._alternate_texture = None if texture_path is None else default_asset_mgr.get_or_create_texture(texture_path)

        self._render_records = {}  # records the rendering information, (sub_mesh_index: mtl_render_record)
        self._sub_meshes = []
        self._sub_mesh_count = 0

        self._initial_transformation: Mat4 = Mat4.identity()
        self._transformation: Mat4 = Mat4.identity()

        self._shading_enabled = True
        self._texture_enabled = True if self._alternate_texture else False
        self._material_enabled = True
        self._alternate_texture_enabled = True if self._alternate_texture else False
        self._normal_map_enabled = False
        self._bump_map_enabled = False
        self._parallax_map_enabled = False

        self._vertex_buffers: [VertexBuffer] = []
        self._index_buffers: [IndexBuffer] = []
        self._vertex_array_objects: [VertexArrayObject] = []

        from ..app.window import __current_window__
        self._shader = kwargs.get('shader', __current_window__.default_shader)

        shapes = kwargs.get('shapes', None)
        vertices = kwargs.get('vertices', None)
        colors = kwargs.get('colors', [Mesh.DEFAULT_COLOR])
        uvs = kwargs.get('uvs', None)
        normals = kwargs.get('normals', None)
        faces = kwargs.get('triangles', None)

        if filepath:
            self._name = str(filepath)
            if load_immediately:
                self._read_from_file(filepath, colors, kwargs.get('texture', None), resolver)
            else:
                self._tmp_colors = colors
        elif shapes is not None:
            self._name = 'unnamed_{}'.format(self.__class__.__name__)
            self._from_shapes(shapes)
        else:
            if not (vertices and colors and uvs and normals and faces):
                missing_attributes = [name for attrib, name in ((vertices, 'vertices'), (colors, 'colors'),
                                                                (uvs, 'uvs'), (normals, 'normals'),
                                                                (faces, 'triangles')) if attrib is None]
                raise ValueError(f"Mesh creation - missing vertex attributes: {missing_attributes}.")

            self._name = kwargs.get('name', 'unnamed_{}'.format(self.__class__.__name__))
            # fill vertices
            self._vertices = np.asarray(vertices, dtype=np.float32).reshape((-1, 3))
            self._vertex_count = len(self._vertices)
            self._vertex_triangles = [[] for i in range(self._vertex_count)]

            # fill colors
            self._colors = Mesh._process_color(self._vertex_count, colors)

            # fill uvs
            self._uvs = np.asarray(uvs, dtype=np.float32).reshape((-1, 2))
            self._uv_count = len(self._uvs)

            # fill normals
            self._normals = np.asarray(normals, dtype=np.float32).reshape((-1, 3))
            self._normal_count = len(self._normals)

            # fill indices and triangulation
            self._triangle_count = 0
            for f in faces:
                vertex_count = len(f[0])
                if vertex_count < 3:
                    continue
                elif vertex_count == 3:
                    self._triangulated_face_index.append(len(self._triangles))
                    self._triangles.append(f)
                    self._triangle_count += 1
                    for p in f[0]:
                        self._vertex_triangles[p].append(self._triangle_count - 1)
                else:
                    self._triangulated_face_index.append(len(self._triangles))
                    triangulated = []
                    for i in range(0, vertex_count - 2):
                        triangulated.append(((f[0][0], *f[0][i+1: i+3]),
                                             (f[1][0], *f[1][i+1: i+3]),
                                             (f[2][0], *f[2][i+1: i+3])))
                        self._triangle_count += 1
                        for p in triangulated[-1][0]:
                            self._vertex_triangles[p].append(self._triangle_count - 1)
                    self._triangles.extend(triangulated)

            self._indices = np.asarray(self._triangles, dtype=np.uint32).ravel()
            self._index_count = len(self._indices)

            self._faces = faces
            self._tangents = self._compute_tangents()
            self._sub_mesh_count = 1
            sub_mesh = SubMesh(
                name=f"{self._name}-{self._materials[0].name}",
                topology=MeshTopology.Triangles,
                vertex_layout=self._vertex_layout,
                triangles=self._faces
            )
            # create rendering record
            v_positions = []
            v_normals = []
            v_uvs = []
            v_colors = []
            v_tangents = []

            for tri in self._triangles:
                for i, content in enumerate(tri):
                    if i == 0:
                        for v_i in content:
                            v_positions.append(self._vertices[v_i])
                            v_colors.append(self._colors[v_i])
                            v_tangents.append(self._tangents[v_i])
                    if i == 1:
                        for vt_i in content:
                            v_uvs.append(self._uvs[vt_i])
                    if i == 2:
                        for vn_i in content:
                            v_normals.append(self._normals[vn_i])

            v_vertices = np.array([z for x in zip(v_positions, v_colors, v_uvs, v_normals, v_tangents) for y in x for z in y],
                                  dtype=np.float32).ravel()

            vao = VertexArrayObject()
            vbo = VertexBuffer(len(v_positions), sub_mesh.vertex_layout)
            vbo.set_data(v_vertices)
            vao.bind_vertex_buffer(vbo, [0, 1, 2, 3, 4])

            self._sub_meshes = [sub_mesh]
            self._render_records = {
                0: MtlRenderRecord(0, len(self._vertex_array_objects), len(self._vertex_buffers), len(v_positions))
            }

            self._vertex_array_objects.append(vao)
            self._vertex_buffers.append(vbo)

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
                output_colors = np.asarray(
                    [c.rgba for color in [colors, [Mesh.DEFAULT_COLOR] * num] for c in color]).reshape((count, 4))
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

    @property
    def normal_map_enabled(self):
        return self._normal_map_enabled

    @normal_map_enabled.setter
    def normal_map_enabled(self, value):
        self._normal_map_enabled = value

    @property
    def bump_map_enabled(self):
        return self._bump_map_enabled

    @bump_map_enabled.setter
    def bump_map_enabled(self, value):
        self._bump_map_enabled = value

    @property
    def parallax_map_enabled(self):
        return self._parallax_map_enabled

    @parallax_map_enabled.setter
    def parallax_map_enabled(self, value):
        self._parallax_map_enabled = value

    @property
    def material_enabled(self):
        return self._material_enabled

    @material_enabled.setter
    def material_enabled(self, value):
        self._material_enabled = value

    @property
    def alternate_texture_enabled(self):
        return self._alternate_texture_enabled

    @alternate_texture_enabled.setter
    def alternate_texture_enabled(self, value):
        self._alternate_texture_enabled = value

    def load(self):
        if os.path.exists(self._name):
            self._read_from_file(self._name, self._tmp_colors)

    def _read_from_file(self, filepath, color, texture=None, resolver=default_resolver):
        reader = WavefrontReader(filepath, resolver)
        mesh_data = reader.read()

        self._vertex_count = len(mesh_data['vertices'])
        self._vertices = np.asarray(mesh_data['vertices'], dtype=np.float32).reshape((-1, 3))

        self._normal_count = len(mesh_data['normals'])
        self._normals = np.asarray(mesh_data['normals'], dtype=np.float32).reshape((-1, 3))

        self._faces = mesh_data['faces']
        self._triangles = [tuple(f) for f in self._faces]
        self._triangle_count = len(self._triangles)
        self._vertex_triangles = [[] for i in range(0, self._vertex_count)]
        for i, tri in enumerate(self._triangles):
            for p in tri[0]:
                self._vertex_triangles[p].append(i)

        self._index_count = self._triangle_count * 3
        self._indices = np.asarray([f[0] for f in mesh_data['faces']], dtype=np.uint32).reshape((-1, 3))

        self._uv_count = len(mesh_data['texcoords'])
        self._uvs = np.asarray(mesh_data['texcoords'], dtype=np.float32).reshape((-1, 2))

        self._colors = Mesh._process_color(self._vertex_count, color)

        self._tangents = self._compute_tangents()

        for name, mat in mesh_data['materials'].items():
            self._materials.append(
                default_asset_mgr.get_or_create_material(
                    name,
                    ambient=mat.get('Ka', [0.8, 0.8, 0.8]),
                    diffuse=mat.get('Kd', [0.8, 0.8, 0.8]),
                    specular=mat.get('Ks', [1.0, 1.0, 1.0]),
                    shininess=mat.get('Ns', 1.0),
                    ior=mat.get('Ni', 1.0),
                    dissolve=mat.get('d', 1.0),
                    illum=mat.get('illum', 0),
                    diffuse_map_path=mat.get('map_Kd', None),
                    bump_map_path=mat.get('map_Bump', None),
                    normal_map_path=mat.get('map_Normal', None))
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

            vertex_attribs.append(VertexAttribDescriptor(VertexAttrib.Tangent, VertexAttribFormat.Float32, 3))

            # pack the data together for rendering
            for mtl in sub_obj_materials:
                mtl_idx, f_start, f_end = mtl

                sub_mesh = SubMesh(
                    name=f"{sub_obj['name']}-{self._materials[mtl[0]].name}",
                    triangles=list(range(f_start, f_end))
                )

                sub_mesh.vertex_layout = VertexLayout(*vertex_attribs)

                v_positions = []
                v_normals = []
                v_texcoords = []
                v_tangents = []
                for face in self._faces[f_start: f_end]:
                    i = 0
                    for v_i in face[i]:
                        v_positions.append(self._vertices[v_i])
                        v_tangents.append(self._tangents[v_i])
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

                if sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
                    vertices = np.array(
                        [z for x in zip(v_positions, v_colors, v_texcoords, v_normals, v_tangents) for y in x for z in y],
                        dtype=np.float32).ravel()
                elif sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and not sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
                    vertices = np.array([z for x in zip(v_positions, v_colors, v_texcoords, v_tangents) for y in x for z in y],
                                        dtype=np.float32).ravel()
                elif not sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
                    vertices = np.array([z for x in zip(v_positions, v_colors, v_normals, v_tangents) for y in x for z in y],
                                        dtype=np.float32).ravel()
                else:
                    vertices = np.array([z for x in zip(v_positions, v_colors, v_tangents) for y in x for z in y],
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
                    vao.bind_vertex_buffer(vbo, [0, 1, 3, 4])
                else:
                    vao.bind_vertex_buffer(vbo, [0, 1, 2, 3, 4])

                self._sub_meshes.append(sub_mesh)

                sub_mesh_index = len(self._sub_meshes) - 1

                self._render_records[sub_mesh_index] = MtlRenderRecord(mtl_idx, vao_idx, vbo_idx, vertex_count)

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

    def _compute_tangents(self):
        """Computes tangent for each vertex over whole mesh."""
        tangents = []
        for i in range(0, self._vertex_count):
            tangents.append(self._compute_tangent_of_vertex(i, self._vertex_triangles))
        return np.asarray(tangents, dtype=np.float32).reshape((-1, 3))

    def _compute_tangent_of_vertex(self, vertex_id, vertex_triangles):
        """
        Compute tangent from Tangent space for given vertex (over whole mesh).
        Note that only tangent is needed, bi-tangent can be computed in shader by cross product: N*T

        e = edge in triangle space
        s = edge in texture space

        we want to find the transformation from texture space to triangle space

        [T_x B_x]   [s1_x s2_x]   [e1_x e2_x]
        [T_y B_y] * [s1_y s2_y] = [e1_y e2_y]
        [T_z B_z]                 [e1_z e2_z]

        by inverting the 2x2 matrix

        [T_x B_x]   [e1_x e2_x]   [ s2_y -s2_x]
        [T_y B_y] = [e1_y e2_y] * [-s1_y  s1_x] * 1/det
        [T_z B_z]   [e1_z e2_z]

        where:
        det = s1_x*s2_y - s1_y*s2_x

        now we can solve for T:

        T_x = e1_x * s2_y - e2_x * s1_y
        T_y = e1_y * s2_y - e2_y * s1_y
        T_z = e1_z * s2_y - e2_z * s1_y

        or, in short:

        T = e1 * s2_y - e2 * s1_y
        """
        tangent = np.array([0.0, 0.0, 0.0], dtype=np.float32)

        for tri_idx in vertex_triangles[vertex_id]:
            vertices = [self._vertices[i] for i in self._triangles[tri_idx][0]]
            uvs = [self._uvs[i] for i in self._triangles[tri_idx][1]]
            # edges (delta vertex positions)
            delta_pos = [vertices[i] - vertices[0] for i in (1, 2)]
            # delta uvs
            delta_uv = [uvs[i] - uvs[0] for i in (1, 2)]
            det = delta_uv[0][0] * delta_uv[1][1] - delta_uv[0][1] * delta_uv[1][0]
            if det != 0.0:
                tangent += (delta_pos[0] * delta_uv[1][1] - delta_pos[1] * delta_uv[0][1]) / det
            # bitangent += (delta_pos[1] * delta_uv[0][0] - delta_pos[0] * delta_uv[1][0]) * det

        return tangent / np.sqrt(np.sum(tangent ** 2))

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
        return tuple(self._sub_meshes)

    @property
    def sub_mesh_count(self):
        return self._sub_mesh_count

    def update_sub_mesh(self, index, new: SubMesh, texture: str = None, normal_map: str = None, create: bool = False):
        sub_mesh = self._sub_meshes[index]
        sub_mesh.name = new.name
        sub_mesh.vertex_count = new.vertex_count
        sub_mesh.triangles = new.triangles
        sub_mesh.topology = new.topology
        sub_mesh.normal_map_enabled = new.normal_map_enabled

        tri_list = []
        vertex_triangles = {}
        for f_i in sub_mesh.triangles:
            start = self._triangulated_face_index[f_i]
            end = len(self._triangles) if f_i >= len(self._triangulated_face_index) - 1 else self._triangulated_face_index[f_i + 1]
            tri_list.extend(self._triangles[start: end])
            for i, tri in enumerate(self._triangles[start: end]):
                for v_i in tri[0]:
                    if v_i not in vertex_triangles:
                        vertex_triangles[v_i] = []
                    vertex_triangles[v_i].append(start + i)

        tangents = {}
        for v_i in vertex_triangles.keys():
            tangents[v_i] = self._compute_tangent_of_vertex(v_i, vertex_triangles)

        # create new vbo with new data
        v_positions = []
        v_normals = []
        v_uvs = []
        v_colors = []
        v_tangents = []
        for tri in tri_list:
            for i in range(0, 3):
                if i == 0:
                    for v_i in tri[i]:  # positions
                        v_positions.append(self._vertices[v_i])
                        v_colors.append(self._colors[v_i])
                        v_tangents.append(tangents[v_i])
                if i == 1:
                    for vt_i in tri[i]:  # uvs
                        v_uvs.append(self._uvs[vt_i])
                if i == 2:
                    for vn_i in tri[i]:  # normals
                        v_normals.append(self._normals[vn_i])

        sub_mesh.vertex_count = len(v_positions)

        vertices = np.array([z for x in zip(v_positions, v_colors, v_uvs, v_normals, v_tangents) for y in x for z in y],
                            dtype=np.float32).ravel()

        mtl_idx = 0

        if create:
            vao = VertexArrayObject()
            vao_idx = len(self._vertex_array_objects)
            self._vertex_array_objects.append(vao)
        else:
            old_record = self._render_records[index]
            self._vertex_buffers[old_record.vbo_idx].delete()
            vao = self._vertex_array_objects[old_record.vao_idx]
            vao_idx = old_record.vao_idx
            mtl_idx = old_record.mtl_idx

        layout = VertexLayout(
            VertexAttribDescriptor(VertexAttrib.Position, VertexAttribFormat.Float32, 3),
            VertexAttribDescriptor(VertexAttrib.Color0, VertexAttribFormat.Float32, 4),
            VertexAttribDescriptor(VertexAttrib.TexCoord0, VertexAttribFormat.Float32, 2),
            VertexAttribDescriptor(VertexAttrib.Normal, VertexAttribFormat.Float32, 3),
            VertexAttribDescriptor(VertexAttrib.Tangent, VertexAttribFormat.Float32, 3)
        )

        vbo = VertexBuffer(sub_mesh.vertex_count, layout)
        vbo.set_data(vertices)

        vbo_idx = len(self._vertex_buffers)
        self._vertex_buffers.append(vbo)

        vao.bind_vertex_buffer(vbo, [0, 1, 2, 3, 4])

        if texture is not None or normal_map is not None:
            mtl_idx = len(self._materials)
            self._materials.append(default_asset_mgr.get_or_create_material(
                f'material_[{texture}]',
                diffuse_map_path=texture,
                normal_map_path=normal_map
            ))
            self._texture_enabled = True

        self._render_records[index] = MtlRenderRecord(mtl_idx,
                                                      vao_idx,
                                                      vbo_idx,
                                                      sub_mesh.vertex_count)

    def append_sub_mesh(self, sub_mesh: SubMesh, texture: str = ''):
        self._sub_meshes.append(SubMesh())
        self.update_sub_mesh(len(self._sub_meshes) - 1, sub_mesh, texture, create=True)

    @property
    def shader(self):
        return self._shader

    @shader.setter
    def shader(self, shader):
        if not shader.is_valid():
            raise ValueError("Invalid shader.")
        self._shader = shader

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

    def draw(self, matrix=Mat4.identity(), shader=None, **kwargs):
        _shader = shader if shader is not None and shader.is_valid() else self._shader
        sub_mesh_count = len(self._sub_meshes)
        if len(self._sub_meshes) > 0:
            with _shader:
                mat = matrix * self._transformation * self._initial_transformation
                _shader['model_mat'] = mat
                _shader['shading_enabled'] = self._shading_enabled
                _shader['in_light_pos'] = kwargs.get('in_light_pos', Vec3(600.0, 600.0, 600.0))
                _shader['light_color'] = kwargs.get('light_color', Vec3(0.8, 0.8, 0.8))

                for idx, record in self._render_records.items():
                    sub_mesh = self._sub_meshes[idx]
                    mtl = self._materials[record.mtl_idx]
                    vao = self._vertex_array_objects[record.vao_idx]

                    _shader['mtl.diffuse'] = mtl.diffuse
                    _shader['mtl.ambient'] = mtl.ambient
                    _shader['mtl.specular'] = mtl.specular
                    _shader['mtl.shininess'] = mtl.shininess
                    _shader['mtl.enabled'] = self._material_enabled
                    from bk7084.app import current_window
                    _shader['time'] = current_window().elapsed_time
                    _shader['mtl.use_diffuse_map'] = self._texture_enabled

                    if sub_mesh_count > 1:
                        _shader['mtl.use_normal_map'] = sub_mesh.normal_map_enabled
                    else:
                        _shader['mtl.use_normal_map'] = self._normal_map_enabled

                    _shader['mtl.use_bump_map'] = self._bump_map_enabled
                    _shader['mtl.use_parallax_map'] = self._parallax_map_enabled

                    diffuse_map = mtl.diffuse_map

                    if self._alternate_texture_enabled and self._alternate_texture is not None:
                        diffuse_map = self._alternate_texture

                    _shader['mtl.diffuse_map'] = 0
                    _shader['mtl.bump_map'] = 1
                    _shader['mtl.normal_map'] = 2

                    _shader.active_texture_unit(0)
                    with diffuse_map:
                        _shader.active_texture_unit(1)
                        with mtl.bump_map:
                            _shader.active_texture_unit(2)
                            with mtl.normal_map:
                                with vao:
                                    gl.glDrawArrays(sub_mesh.topology.value, 0, record.vertex_count)
