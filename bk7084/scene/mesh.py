import collections.abc
import enum
import os
from collections import namedtuple
from dataclasses import dataclass

import numpy as np

from .loader.obj import WavefrontReader
from .. import gl
from ..assets import default_resolver
from ..graphics.array import VertexArrayObject
from ..graphics.buffer import VertexBuffer, IndexBuffer
from ..graphics.material import Material
from ..graphics.texture import Texture
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
                 vertex_layout=VertexLayout.default()):
        self.name: str = name
        self.triangles: [] = [] if triangles is None else triangles  # index of triangles (faces)
        self.vertex_count: int = len(triangles) * 3 if triangles is not None else 0
        self.topology: MeshTopology = topology
        self.vertex_layout: VertexLayout = vertex_layout


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
        self._triangles = []  # [(vertices indices), (vertex uvs indices), ( vertex normals indices)]

        self._materials = [Material.default()]  # loaded materials from OBJ file
        texture_path = kwargs.get('texture', None)
        self._alternate_texture = None if texture_path is None else Texture(resolver.resolve(texture_path))

        self._render_records = {}  # records the rendering information, (sub_mesh_index: mtl_render_record)
        self._sub_meshes = []
        self._sub_mesh_count = 0

        self._initial_transformation: Mat4 = Mat4.identity()
        self._transformation: Mat4 = Mat4.identity()

        self._shading_enabled = True
        self._texture_enabled = True if self._alternate_texture else False
        self._material_enabled = True
        self._alternate_texture_enabled = True if self._alternate_texture else False

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
        triangles = kwargs.get('triangles', None)

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
            if not (vertices and colors and uvs and normals and triangles):
                missing_attributes = [name for attrib, name in ((vertices, 'vertices'), (colors, 'colors'),
                                                                (uvs, 'uvs'), (normals, 'normals'),
                                                                (triangles, 'triangles')) if attrib is None]
                raise ValueError(f"Mesh creation - missing vertex attributes: {missing_attributes}.")

            self._name = kwargs.get('name', 'unnamed_{}'.format(self.__class__.__name__))
            # fill vertices
            self._vertices = np.asarray(vertices, dtype=np.float32).reshape((-1, 3))
            self._vertex_count = len(self._vertices)

            # fill colors
            self._colors = Mesh._process_color(self._vertex_count, colors)

            # fill uvs
            self._uvs = np.asarray(uvs, dtype=np.float32).reshape((-1, 2))
            self._uv_count = len(self._uvs)

            # fill normals
            self._normals = np.asarray(normals, dtype=np.float32).reshape((-1, 3))
            self._normal_count = len(self._normals)

            # fill indices and triangulation
            triangle_list = []
            for f in triangles:
                vertex_count = len(f[0])
                if vertex_count < 3:
                    continue
                elif vertex_count == 3:
                    triangle_list.append(f)
                else:
                    for i in range(0, vertex_count - 2):
                        triangle_list.append([(f[0][0], *f[0][i+1: i+3]),
                                              (f[1][0], *f[1][i+1: i+3]),
                                              (f[2][0], *f[2][i+1: i+3])])
            self._indices = np.asarray(triangle_list, dtype=np.uint32).ravel()
            self._index_count = len(self._indices)

            self._triangles = triangles
            self._triangle_count = len(self._triangles)

            self._sub_mesh_count = 1
            sub_mesh = SubMesh(
                name=f"{self._name}-{self._materials[0].name}",
                topology=MeshTopology.Triangles,
                vertex_layout=self._vertex_layout,
                triangles=self._triangles
            )
            # create rendering record
            v_positions = []
            v_normals = []
            v_uvs = []
            v_colors = []

            for face in triangle_list:
                for i, content in enumerate(face):
                    if i == 0:
                        for v_i in content:
                            v_positions.append(self._vertices[v_i])
                            v_colors.append(self._colors[v_i])
                    if i == 1:
                        for vt_i in content:
                            v_uvs.append(self._uvs[vt_i])
                    if i == 2:
                        for vn_i in content:
                            v_normals.append(self._normals[vn_i])

            v_vertices = np.array([z for x in zip(v_positions, v_colors, v_uvs, v_normals) for y in x for z in y],
                                  dtype=np.float32).ravel()

            vao = VertexArrayObject()
            vbo = VertexBuffer(len(v_positions), sub_mesh.vertex_layout)
            vbo.set_data(v_vertices)
            vao.bind_vertex_buffer(vbo, [0, 1, 2, 3])

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

        self._triangle_count = len(self._triangles)
        self._triangles = mesh_data['faces']

        self._index_count = self._triangle_count * 3
        self._indices = np.asarray([f[0] for f in mesh_data['faces']], dtype=np.uint32).reshape((-1, 3))

        self._uv_count = len(mesh_data['texcoords'])
        self._uvs = np.asarray(mesh_data['texcoords'], dtype=np.float32).reshape((-1, 2))

        self._colors = Mesh._process_color(self._vertex_count, color)

        for name, mat in mesh_data['materials'].items():
            self._materials.append(
                Material(
                    name,
                    mat.get('map_Kd', texture),
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
                mtl_idx, f_start, f_end = mtl

                sub_mesh = SubMesh(
                    name=f"{sub_obj['name']}-{self._materials[mtl[0]].name}",
                    triangles=list(range(f_start, f_end))
                )

                sub_mesh.vertex_layout = VertexLayout(*vertex_attribs)

                v_positions = []
                v_normals = []
                v_texcoords = []
                for face in self._triangles[f_start: f_end]:
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

                if sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
                    vertices = np.array(
                        [z for x in zip(v_positions, v_colors, v_texcoords, v_normals) for y in x for z in y],
                        dtype=np.float32).ravel()
                elif sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and not sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
                    vertices = np.array([z for x in zip(v_positions, v_colors, v_texcoords) for y in x for z in y],
                                        dtype=np.float32).ravel()
                elif not sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
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

    def update_sub_mesh(self, index, new: SubMesh, texture: str = '', create: bool = False):
        sub_mesh = self._sub_meshes[index]
        sub_mesh.name = new.name
        sub_mesh.vertex_count = new.vertex_count
        sub_mesh.triangles = new.triangles
        sub_mesh.topology = new.topology

        # create new vbo with new data
        v_positions = []
        v_normals = []
        v_uvs = []
        v_colors = []
        for idx in sub_mesh.triangles:
            # [(vertex index), (uv index), (normal index)]
            face = self._triangles[idx]
            vertex_count = len(face[0])
            triangle_list = []
            if vertex_count < 3:
                continue
            elif vertex_count == 3:
                triangle_list.append(face)
            else:
                for i in range(0, vertex_count - 2):
                    triangle_list.append([(face[0][0], *face[0][i+1: i+3]),
                                          (face[1][0], *face[1][i+1: i+3]),
                                          (face[2][0], *face[2][i+1: i+3])])
            for f in triangle_list:
                for i in range(0, 3):
                    if i == 0:
                        for v_i in f[i]:
                            v_positions.append(self._vertices[v_i])
                            v_colors.append(self._colors[v_i])
                    if i == 1:
                        for vt_i in f[i]:
                            v_uvs.append(self._uvs[vt_i])
                    if i == 2:
                        for vn_i in f[i]:
                            v_normals.append(self._normals[vn_i])

        sub_mesh.vertex_count = len(v_positions)

        vertices = np.array([z for x in zip(v_positions, v_colors, v_uvs, v_normals) for y in x for z in y],
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

        vbo = VertexBuffer(sub_mesh.vertex_count, sub_mesh.vertex_layout)
        vbo.set_data(vertices)

        vbo_idx = len(self._vertex_buffers)
        self._vertex_buffers.append(vbo)

        vao.bind_vertex_buffer(vbo, [0, 1, 2, 3])

        if texture != '':
            mtl_idx = len(self._materials)
            self._materials.append(Material.default(texture))
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

    def draw(self, matrix=Mat4.identity(), shader=None):
        _shader = shader if shader is not None and shader.is_valid() else self._shader
        if len(self._sub_meshes) > 0:
            with _shader:
                mat = matrix * self._transformation * self._initial_transformation
                _shader['model_mat'] = mat
                _shader['shading_enabled'] = self._shading_enabled
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

                    _shader.active_texture_unit(0)
                    texture = mtl.texture_diffuse

                    if self._alternate_texture_enabled and self._alternate_texture is not None:
                        texture = self._alternate_texture

                    with texture:
                        with vao:
                            gl.glDrawArrays(sub_mesh.topology.value, 0, record.vertex_count)
