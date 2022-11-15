import numpy as np
import collections.abc
import enum
import hashlib
from dataclasses import dataclass
from typing import List
from .. import gl
from ..assets.manager import default_asset_mgr, default_name_mgr
from ..assets.resolver import default_resolver
from ..graphics.array import VertexArrayObject
from ..graphics.buffer import VertexBuffer, IndexBuffer
from ..graphics.material import Material
from ..graphics.vertex_layout import VertexLayout, VertexAttrib, VertexAttribDescriptor, VertexAttribFormat
from ..math import Mat4, Vec3
from ..misc import PaletteDefault, Color


# todo: face normals
# todo: sub-meshes and render records


@dataclass(frozen=True)
class Face:
    __slots__ = ('vs', 'vts', 'vns')
    vs: [int]  # vertex indices
    vts: [int]  # vertex texture indices
    vns: [int]  # vertex normal indices

    @property
    def is_triangle(self):
        return len(self.vs) == 3

    @property
    def vertex_count(self):
        return len(self.vs)


@dataclass(frozen=False)
class VertexAttribData:
    """
    Stores the data of a vertex attribute.
    """
    __slots__ = ('count', 'data')
    count: int
    data: np.ndarray

    @property
    def is_empty(self):
        return self.count == 0 and (not self.data.any())

    def __getitem__(self, i):
        return self.data[i]


class MeshTopology(enum.Enum):
    Triangles = gl.GL_TRIANGLES
    Lines = gl.GL_LINES
    LineStrip = gl.GL_LINE_STRIP
    Points = gl.GL_POINTS


@dataclass(frozen=True)
class MtlRenderRecord:
    """
    A render record for a material.
    """
    __slots__ = ['mtl_idx', 'vao_idx', 'vbo_idx', 'vertex_count', 'pipeline_idx']
    mtl_idx: int
    vao_idx: int
    vbo_idx: int
    vertex_count: int
    pipeline_idx: int


class SubMesh:
    """
    Each sub-mesh corresponds to a Material. A sub-mesh consists of a list of triangles,
    which refer to a set of vertices(referenced by index). Vertices can be shared between
    multiple sub-meshes.

    Note: this is a structure maintained inside `Mesh` class, and should not be used directly.
    """

    __slots__ = ['name', 'faces', 'vertex_count', 'topology', 'vertex_layout',
                 'normal_map_enabled', 'bump_map_enabled', 'parallax_map_enabled']

    def __init__(self,
                 faces=None,
                 name='',
                 topology=MeshTopology.Triangles,
                 vertex_layout=VertexLayout.default(),
                 enable_normal_map=False,
                 enable_bump_map=False,
                 enable_parallax_map=False):
        self.name: str = name
        self.faces: [] = [] if faces is None else faces  # index of faces
        self.vertex_count: int = len(faces) * 3 if faces is not None else 0
        self.topology: MeshTopology = topology
        self.vertex_layout: VertexLayout = vertex_layout
        self.normal_map_enabled: bool = enable_normal_map
        self.bump_map_enabled: bool = enable_bump_map
        self.parallax_map_enabled: bool = enable_parallax_map

    @staticmethod
    def from_sub_meshes(sub_meshes, name=''):
        triangles = []
        for sub_mesh in sub_meshes:
            triangles += sub_mesh.faces
        return SubMesh(faces=triangles, name=name)


def _load_pipeline_uniforms(pipeline, excluded=('camera', 'shadow_map'), **kwargs):
    for key, value in kwargs.items():
        if key not in excluded:
            pipeline[key] = value


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
    """
    __slots__ = ['_vertex_layout', '_index_count', '_verts', '_indices', '_faces',
                 '_faces_triangulation', '_triangle_count', '_triangles', '_materials',
                 '_render_records', '_name', '_sub_meshes_raw', '_sub_meshes', '_sub_mesh_count',
                 '_pipelines', '_init_transform', '_transform', '_shading_enabled',
                 '_normal_map_enabled', '_bump_map_enabled', '_parallax_map_enabled', '_vertex_buffers',
                 '_index_buffers', '_vertex_array_objects', '_cast_shadow', '_material_enabled', '_texture_enabled'
                 ]
    DEFAULT_COLOR = PaletteDefault.BrownB.as_color()

    def __init__(self, name, filepath=None, resolver=default_resolver, **kwargs):
        """
        Creation and initialisation of a Mesh. A mesh can be created by:

        1. reading a OBJ file
        2. providing vertices and its attributes

        Materials(including textures) are specified by specifying the sub-meshes.

        Note:
            If `filepath` is specified, this function will skip arguments inside `kwargs`.

        Args:
            filepath (str):
                Specifies the path of the model file to be loaded.

            **kwargs:
                vertices: (array like):
                    Specifies the position of vertices.

                colors: (Sequence[Color]):
                    Specifies the color of vertices.

                normals: (array like):
                    Specifies the normal of vertices.

                faces: (array like):
                    Specifies the faces of the mesh. Each face is a list consisting of
                    indices of vertices, texture coordinates, and normals. For example,
                    a face can be [(v0, v1, v2), (vt0, vt1, vt2), (vn0, vn1, vn2)], where
                    v0, v1, v2 are indices of vertices, vt0, vt1, vt2 are indices of texture
                    coordinates, and vn0, vn1, vn2 are indices of normals.
                    A list of faces is represented as
                    [
                        [(v0, v1, v2), (vt0, vt1, vt2), (vn0, vn1, vn2)],
                        [(v3, v4, v5, v6), (vt3, vt4, vt5, vt6), (vn3, vn4, vn5, vt6)],
                    ]
                    Faces don't have to be triangles, they can be quads or other polygons,
                    but they have to be convex. Behind the scene, the faces will be triangulated.

                uvs: (array like):
                    Specifies the uv coordinates of the vertices.

                init_transform (Mat4):
                    Specifies the initial transform of the mesh. Can also be specified after the mesh is created.

                transform (Mat4):
                    Specifies the transform of the mesh. Can also be specified after the mesh is created.

                shading_enabled (bool):
                    Specifies whether the shading is enabled (using Phong model).

                material_enabled (bool):
                    Specifies whether the material is enabled. If enabled, the mesh will be rendered with the
                    material loaded from the model file or the default material if no material is loaded. In
                    case the material is not enabled, the mesh will be rendered with vertex colors.

                texture_enabled (bool):
                    Specifies whether the diffuse map will be used.

                cast_shadow (bool):
                    Specifies whether the object will generate shadow.

                vertex_shader (str):
                    Specifies the vertex shader going to be used for rendering. In case of non specified,
                    use framework's default shader (declared inside of Window).

                pixel_shader (str):
                    Specifies the pixel shader going to be used for rendering. In case of non specified,
                    use framework's default shader (declared inside of Window).
        """
        assert name is not None, 'Mesh name cannot be None'
        self._vertex_layout = VertexLayout.default()
        self._index_count = 0
        self._verts = {
            'position': VertexAttribData(0, np.array([], dtype=np.float32)),
            'color': VertexAttribData(0, np.array([], dtype=np.float32)),
            'normal': VertexAttribData(0, np.array([], dtype=np.float32)),
            'texcoord': VertexAttribData(0, np.array([], dtype=np.float32)),
            'tangent': np.array([], dtype=np.float32),
            # stores the indices of triangles that each vertex belongs to; used for calculating tangent and bi-tangent.
            'topology': [[0]],
        }
        self._indices = np.array([], dtype=np.uint32)  # indices for draw whole mesh
        # Face data consists of vertex indices, texture indices, and normal indices.
        # [[(vertices indices), (vertex uvs indices), (vertex normals indices)], ...]
        # Used only when the mesh is created by providing vertices and its attributes.
        self._faces: List[Face] = []

        # List keeping track of triangulated faces in case the mesh is created by providing vertices and their
        # attributes. Stores the first index of triangles of a face after the triangulation in the self._triangles
        # array. Take example of a list of two faces, first face is a quad and second face is a triangle. This list
        # will be [0, 3] after triangulation, where 0 is the index of the first triangle of triangulation of the
        # first face, and 3 is the index of the first triangle of triangulation of the second face.
        self._faces_triangulation = []
        self._triangle_count = 0
        self._triangles = []

        self._materials = [Material("bk7084_default_material")]
        self._render_records = {}  # records the rendering information, (sub_mesh_index: mtl_render_record)
        self._sub_meshes = []
        self._sub_meshes_raw = []  # stores the unprocessed submesh inputs, for easy merging of meshes
        self._sub_mesh_count = 0
        self._pipelines = [
            default_asset_mgr.get_or_create_pipeline('default_pipeline',
                                                     vertex_shader='shaders/default.vert',
                                                     pixel_shader='shaders/default.frag')
        ]
        # initial transformation applied to the mesh
        self._init_transform: Mat4 = kwargs.get('init_transform', Mat4.identity())
        # transformation applied to the mesh
        self._transform: Mat4 = Mat4.identity()

        self._shading_enabled = kwargs.get('shading_enabled', True)
        self._material_enabled = kwargs.get('material_enabled', True)  # by default, use material if available
        self._texture_enabled = kwargs.get('texture_enabled', False)  # by default, texture is not enabled
        self._normal_map_enabled = False
        self._bump_map_enabled = False
        self._parallax_map_enabled = False

        self._vertex_buffers: [VertexBuffer] = []
        self._index_buffers: [IndexBuffer] = []
        self._vertex_array_objects: [VertexArrayObject] = []

        self._cast_shadow = kwargs.get('cast_shadow', True)

        colors = kwargs.get('colors', [Mesh.DEFAULT_COLOR])
        if filepath:  # load from file
            self._name = f"{name}@{default_name_mgr.get_name(hashlib.md5(filepath.encode('utf-8')).hexdigest())}"
            self._load_from_file(filepath, colors, resolver)
        else:  # load from vertices, uvs, normals, faces
            self._name = f"{name}@{default_name_mgr.get_name('customized')}"
            self._verts['position'].data = np.asarray(kwargs['vertices'], dtype=np.float32).reshape(
                (-1, 3)) if 'vertices' in kwargs else None
            self._verts['position'].count = len(self._verts['position'].data)
            self._verts['texcoord'].data = np.asarray(kwargs['uvs'], dtype=np.float32).reshape(
                (-1, 2)) if 'uvs' in kwargs else None
            self._verts['texcoord'].count = len(self._verts['texcoord'].data)
            self._verts['normal'].data = np.asarray(kwargs['normals'], dtype=np.float32).reshape(
                (-1, 3)) if 'normals' in kwargs else None
            self._verts['normal'].count = len(self._verts['normal'].data)
            _faces = kwargs.get('faces', None)

            # Check if the necessary data is provided for creating the mesh.
            if not (self._verts['position'] is not None
                    and colors
                    and self._verts['texcoord'] is not None
                    and self._verts['normal'] is not None
                    and self._faces is not None):
                missing_attributes = [name for attrib, name in ((self._verts['position'], 'vertices'),
                                                                (colors, 'colors'),
                                                                (self._verts['texcoord'], 'uvs'),
                                                                (self._verts['normal'], 'normals'),
                                                                (_faces, 'faces')) if attrib is None]
                raise ValueError(f"Mesh creation - missing vertex attributes: {missing_attributes}.")

            # Check if the input faces are valid.
            for face in _faces:
                if len(face) != 3:
                    raise ValueError(f"Mesh creation - a face must have vertices indices, "
                                     f"texture coordinates indices, and normal indices: {face}.")
                if len(face[0]) < 3:
                    raise ValueError(f"Mesh creation - a face must have at least 3 vertices: {face}.")
                self._faces.append(Face(*face))
            self._verts['color'] = Mesh._process_color(self._verts['position'].count, colors)
            self._verts['topology'] = [[] for _ in range(self._verts['position'].count)]

            # Triangulate faces into triangles and store the indices of triangles of each face.
            _triangles = []
            self._triangle_count = 0
            for f in self._faces:
                if f.is_triangle:
                    self._faces_triangulation.append(len(_triangles))
                    _triangles.append(
                        (
                            (f.vs[0], f.vs[1], f.vs[2]),
                            (f.vts[0], f.vts[1], f.vts[2]),
                            (f.vns[0], f.vns[1], f.vns[2]),
                        )
                    )
                    self._triangle_count += 1
                    for p in f.vs:
                        self._verts['topology'][p].append(self._triangle_count - 1)
                else:
                    self._faces_triangulation.append(len(_triangles))
                    triangulated = []
                    for i in range(0, f.vertex_count - 2):
                        triangulated.append(
                            (
                                (f.vs[0], *f.vs[i + 1: i + 3]),  # vertices
                                (f.vts[0], *f.vts[i + 1: i + 3]),  # texture coordinates
                                (f.vns[0], *f.vns[i + 1: i + 3])  # normals
                            )
                        )
                        self._triangle_count += 1
                        for p in triangulated[-1][0]:
                            self._verts['topology'][p].append(self._triangle_count - 1)
                    _triangles.extend(triangulated)
            self._triangles = np.asarray(_triangles, dtype=np.uint32).reshape((-1, 3, 3))
            self._indices = np.asarray(self._triangles, dtype=np.uint32).ravel()
            self._index_count = len(self._indices)
            if not self._verts['texcoord'].is_empty:
                self._verts['tangents'] = compute_all_tangents(self._verts['position'], self._verts['texcoord'],
                                                               self._triangles, self._verts['topology'])
            else:
                self._verts['tangents'] = np.zeros((self._verts['position'].count, 3), dtype=np.float32)
            self._sub_mesh_count = 1
            sub_mesh = SubMesh(
                name=f"{self._name}_sub_{self._materials[0].name}",
                topology=MeshTopology.Triangles,
                vertex_layout=self._vertex_layout,
                faces=self._faces
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
                            v_positions.append(self._verts['position'][v_i])
                            v_colors.append(self._verts['color'][v_i])
                            v_tangents.append(self._verts['tangents'][v_i])
                    if i == 1:
                        for vt_i in content:
                            v_uvs.append(self._verts['texcoord'][vt_i])
                    if i == 2:
                        for vn_i in content:
                            v_normals.append(self._verts['normal'][vn_i])

            v_vertices = np.array(
                [z for x in zip(v_positions, v_colors, v_uvs, v_normals, v_tangents) for y in x for z in y],
                dtype=np.float32).ravel()

            vao = VertexArrayObject()
            vbo = VertexBuffer(len(v_positions), sub_mesh.vertex_layout)
            vbo.set_data(v_vertices)
            vao.bind_vertex_buffer(vbo, [0, 1, 2, 3, 4])

            self._sub_meshes = [sub_mesh]
            # TODO: store sub-meshes pipelines
            self._render_records = {
                0: MtlRenderRecord(0, len(self._vertex_array_objects), len(self._vertex_buffers), len(v_positions), 0)
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
        return VertexAttribData(count, output_colors)

    @property
    def rendering_info(self):
        info = {}
        for sub_mesh_idx, record in self._render_records.items():
            pipeline = self._pipelines[record.pipeline_idx]
            vao = self._vertex_array_objects[record.vao_idx]
            vertex_count = record.vertex_count
            mtl = self._materials[record.mtl_idx]
            topology = self._sub_meshes[sub_mesh_idx].topology.value
            diffuse_map = mtl.diffuse_map
            transform = self._transform * self._init_transform
            if pipeline.uuid not in info:
                info[pipeline.uuid] = []
            info[pipeline.uuid].append((vao, topology, vertex_count, mtl,
                                        diffuse_map,
                                        self._shading_enabled,
                                        self._material_enabled,
                                        self._texture_enabled,
                                        self._normal_map_enabled,
                                        self._bump_map_enabled,
                                        self._parallax_map_enabled,
                                        transform))
        return info

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
    def cast_shadow(self):
        return self._cast_shadow

    @cast_shadow.setter
    def cast_shadow(self, value):
        self._cast_shadow = value

    def _load_from_file(self, filepath, color, resolver=default_resolver):
        mesh_data = default_asset_mgr.get_or_load_wavefront_obj(filepath, resolver)
        self._verts['position'].data = np.asarray(mesh_data['positions'], dtype=np.float32).reshape((-1, 3))
        self._verts['position'].count = len(mesh_data['positions'])
        self._verts['normal'].data = np.asarray(mesh_data['normals'], dtype=np.float32).reshape((-1, 3))
        self._verts['normal'].count = len(mesh_data['normals'])
        self._faces = mesh_data['faces']
        self._triangles = np.array(self._faces).reshape((-1, 3, 3))
        self._triangle_count = len(self._triangles)
        self._verts['topology'] = [[] for _ in range(0, self._verts['position'].count)]
        for i, tri in enumerate(self._triangles):
            for p in tri[0]:
                self._verts['topology'][p].append(i)

        self._index_count = self._triangle_count * 3
        self._indices = np.asarray([f[0] for f in mesh_data['faces']], dtype=np.uint32).reshape((-1, 3))
        self._verts['texcoord'].count = len(mesh_data['texcoords'])
        self._verts['texcoord'].data = np.asarray(mesh_data['texcoords'], dtype=np.float32).reshape((-1, 2))
        self._verts['color'] = Mesh._process_color(self._verts['position'].count, color)
        if not self._verts['texcoord'].is_empty:
            self._verts['tangent'] = compute_all_tangents(self._verts['position'], self._verts['texcoord'],
                                                          self._triangles, self._verts['topology'])
        else:
            self._verts['tangent'] = np.zeros((self._verts['position'].count, 3), dtype=np.float32)

        # Loaded model has a name like "{hash(filepath)}_000", to avoid creating multiple times the same materials,
        # we remove the _000 suffix from the name, so that the material is shared between all meshes with the same name.
        for name, mat in mesh_data['materials'].items():
            mtl_name = f"{'_'.join(self._name.split('_')[:-1])}_{name}"
            self._materials.append(
                Material(
                    mtl_name,
                    ka=mat.get('Ka', [0.8, 0.8, 0.8]),
                    kd=mat.get('Kd', [0.8, 0.8, 0.8]),
                    ks=mat.get('Ks', [1.0, 1.0, 1.0]),
                    shininess=mat.get('Ns', 1.0),
                    ior=mat.get('Ni', 1.0),
                    dissolve=mat.get('d', 1.0),
                    illum=mat.get('illum', 0),
                    diffuse_map_path=mat['map_Kd']['path'] if 'map_Kd' in mat else None,
                    bump_map_path=mat['map_bump']['path'] if 'map_bump' in mat else None,
                    normal_map_path=mat['map_norm']['path'] if 'map_norm' in mat else None
                )
            )

        for sub_obj in mesh_data['objects']:
            sub_obj_materials = []
            for mtl in sub_obj['materials']:
                mtl_name = f"{'_'.join(self._name.split('_')[:-1])}_{mtl['name']}"
                f_start, f_end = tuple(mtl['f_range'])
                mtl_idx = self._materials.index([x for x in self._materials if x.name == mtl_name][0])
                if mtl_idx != -1 and mtl_idx < len(self._materials):
                    sub_obj_materials.append((mtl_idx, f_start, f_end))

            # will populate a default color
            color = self._verts['color'][0]
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
                    faces=list(range(f_start, f_end))
                )
                sub_mesh.vertex_layout = VertexLayout(*vertex_attribs)
                v_positions = []
                v_normals = []
                v_texcoords = []
                v_tangents = []
                for face in self._faces[f_start: f_end]:
                    i = 0
                    for v_i in face[i]:
                        v_positions.append(self._verts['position'][v_i])
                        v_tangents.append(self._verts['tangent'][v_i])
                    i += 1
                    if sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0):
                        for vt_i in face[i]:
                            v_texcoords.append(self._verts['texcoord'][vt_i])
                        i += 1
                    if sub_mesh.vertex_layout.has(VertexAttrib.Normal):
                        for vn_i in face[i]:
                            v_normals.append(self._verts['normal'][vn_i])

                # create one interleaved buffer
                vertex_count = len(v_positions)
                v_colors = [color] * vertex_count

                if sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
                    vertices = np.array(
                        [z for x in zip(v_positions, v_colors, v_texcoords, v_normals, v_tangents) for y in x for z in
                         y],
                        dtype=np.float32).ravel()
                elif sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and not sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
                    vertices = np.array(
                        [z for x in zip(v_positions, v_colors, v_texcoords, v_tangents) for y in x for z in y],
                        dtype=np.float32).ravel()
                elif not sub_mesh.vertex_layout.has(VertexAttrib.TexCoord0) and sub_mesh.vertex_layout.has(
                        VertexAttrib.Normal):
                    vertices = np.array(
                        [z for x in zip(v_positions, v_colors, v_normals, v_tangents) for y in x for z in y],
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

                self._render_records[sub_mesh_index] = MtlRenderRecord(mtl_idx, vao_idx, vbo_idx, vertex_count,
                                                                       0)
        self._sub_mesh_count = len(self._sub_meshes)

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
        return self._verts['position'].data

    @vertices.setter
    def vertices(self, positions):
        self._verts['position'].data = np.asarray(positions, dtype=np.float32).ravel()
        self._verts['position'].count = len(self._verts['position'].data) / 3
        print("Vertices: ", self._verts['position'].data)
        print("Vertices count: ", self._verts['position'].count)

    @property
    def texcoords(self):
        """Returns the vertex uvs."""
        return self._verts['texcoord'].data

    @property
    def vertex_count(self):
        """Vertex count of the mesh."""
        return self._verts['position'].count

    @property
    def colors(self):
        """Vertex colors of the mesh."""
        return self._verts['color']

    @colors.setter
    def colors(self, colors):
        """Vertex colors of the mesh."""
        self._verts['color'] = np.asarray(colors, dtype=np.float32).ravel()

    @property
    def normals(self):
        """Vertex normals of the mesh."""
        return self._verts['normal'].data

    @property
    def triangles(self):
        """Triangles of the mesh."""
        return self._triangles

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

    @property
    def sub_meshes_raw(self):
        return self._sub_meshes_raw

    def update_sub_mesh(self, index, new: SubMesh, texture: str = None, normal_map: str = None,
                        vertex_shader: str = None, pixel_shader: str = None, create: bool = False):
        if len(self.sub_meshes_raw) == 0:
            self._sub_meshes_raw.append((new, texture, normal_map))

        sub_mesh = self._sub_meshes[index]
        sub_mesh.name = new.name
        sub_mesh.vertex_count = new.vertex_count
        sub_mesh.faces = new.faces
        sub_mesh.topology = new.topology
        sub_mesh.normal_map_enabled = new.normal_map_enabled

        tri_list = []
        vertex_triangles = {}
        for f_i in sub_mesh.faces:
            start = self._faces_triangulation[f_i]
            end = len(self._triangles) if f_i >= len(self._faces_triangulation) - 1 else \
                self._faces_triangulation[f_i + 1]
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
                        v_positions.append(self._verts['position'][v_i])
                        v_colors.append(self._verts['color'][v_i])
                        v_tangents.append(tangents[v_i])
                if i == 1:
                    for vt_i in tri[i]:  # uvs
                        v_uvs.append(self._verts['texcoord'][vt_i])
                if i == 2:
                    for vn_i in tri[i]:  # normals
                        v_normals.append(self._verts['normal'][vn_i])

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
            self._materials.append(
                Material(
                    f'sub_mesh_material_{hashlib.md5(texture.encode("utf-8")).hexdigest()}',
                    diffuse_map_path=texture,
                    normal_map_path=normal_map,
                )
            )
            self._texture_enabled = True

        pipeline = 0
        if vertex_shader is not None or pixel_shader is not None:
            self._pipelines.append(
                default_asset_mgr.get_or_create_pipeline(
                    f'{id(self)}_sub_mesh_{id(sub_mesh)}_pipeline',
                    vertex_shader, pixel_shader
                )
            )
            pipeline = len(self._pipelines) - 1

        self._render_records[index] = MtlRenderRecord(mtl_idx,
                                                      vao_idx,
                                                      vbo_idx,
                                                      sub_mesh.vertex_count, pipeline)

    def append_sub_mesh(self, sub_mesh: SubMesh, texture: str = '', normal_map: str = None, vertex_shader: str = None,
                        pixel_shader: str = None):
        self._sub_meshes_raw.append((sub_mesh, texture, normal_map))
        self._sub_meshes.append(SubMesh())
        self.update_sub_mesh(len(self._sub_meshes) - 1, sub_mesh, texture, normal_map, vertex_shader, pixel_shader,
                             create=True)

    def _compute_tangent_of_vertex(self, vertex_id, vertex_triangles):
        """
        Compute tangent from Tangent space for given vertex (over whole mesh).
        """
        tangent = np.array([0.0, 0.0, 0.0], dtype=np.float32)

        if self._verts['texcoord'].is_empty:
            return np.array([0.0, 0.0, 0.0], dtype=np.float32)
        for tri_idx in vertex_triangles[vertex_id]:
            vertices = [self._verts['position'][i] for i in self._triangles[tri_idx][0]]
            uvs = [self._verts['texcoord'][i] for i in self._triangles[tri_idx][1]]
            # edges (delta vertex positions)
            delta_pos = [vertices[i] - vertices[0] for i in (1, 2)]
            # delta uvs
            delta_uv = [uvs[i] - uvs[0] for i in (1, 2)]
            det = delta_uv[0][0] * delta_uv[1][1] - delta_uv[0][1] * delta_uv[1][0]
            if det != 0.0:
                tangent += (delta_pos[0] * delta_uv[1][1] - delta_pos[1] * delta_uv[0][1]) / det
        return tangent / np.sqrt(np.sum(tangent ** 2))

    def apply_transform(self, matrix: Mat4):
        self._transform = matrix * self._transform
        return self

    def then(self, matrix: Mat4):
        self._transform = matrix * self._transform
        return self

    def reset_transform(self):
        self._transform = Mat4.identity()

    @property
    def transform(self):
        return self._transform

    @transform.setter
    def transform(self, value: Mat4):
        self._transform = value

    @property
    def init_transform(self):
        return self._init_transform

    @init_transform.setter
    def init_transform(self, value: Mat4):
        self._init_transform = value

    @property
    def vertex_array_objects(self):
        return self._vertex_array_objects

    def compute_energy(self, shader, transform, light, viewport_size, depth_map):
        model_mat = transform * self._transform * self._init_transform
        if len(self._sub_meshes) > 0:
            with shader:
                shader['model_mat'] = model_mat
                shader['depth_map'] = 0
                shader['light_mat'] = light.matrix
                shader['light_pos'] = light.position
                shader['light_view_mat'] = light.view_matrix
                shader['resolution'] = viewport_size
                for idx, record in self._render_records.items():
                    sub_mesh = self._sub_meshes[idx]
                    vao = self._vertex_array_objects[record.vao_idx]
                    shader.active_texture_unit(0)
                    with depth_map:
                        with vao:
                            gl.glDrawArrays(sub_mesh.topology.value, 0, record.vertex_count)

    def draw(self, matrix=Mat4.identity(), shader=None, **kwargs):
        if self._sub_mesh_count > 0:
            for idx, record in self._render_records.items():
                sub_mesh = self._sub_meshes[idx]
                mtl = self._materials[record.mtl_idx]
                mat = matrix * self._transform * self._init_transform
                vao = self._vertex_array_objects[record.vao_idx]
                pipeline = self._pipelines[record.pipeline_idx] if shader is None else shader

                camera_enabled = kwargs.get('camera_enabled', True)
                from .. import app
                camera = kwargs.get('camera', app.current_window().camera)

                with pipeline:
                    for key, value in kwargs.items():
                        if key != 'camera' and key != 'shadow_map':
                            pipeline[key] = value

                    if camera_enabled:
                        pipeline['view_mat'] = camera.view_matrix
                        pipeline['proj_mat'] = camera.projection_matrix

                    pipeline['model_mat'] = mat
                    pipeline['shading_enabled'] = self._shading_enabled
                    pipeline['in_light_pos'] = kwargs.get('in_light_pos', Vec3(600.0, 600.0, 600.0))
                    pipeline['is_directional'] = kwargs.get('is_directional', False)
                    pipeline['in_light_dir'] = kwargs.get('in_light_dir', Vec3(1.0, 1.0, 1.0).normalised)
                    pipeline['light_color'] = kwargs.get('light_color', Vec3(0.8, 0.8, 0.8))

                    pipeline['mtl.diffuse'] = mtl.diffuse
                    pipeline['mtl.ambient'] = mtl.ambient
                    pipeline['mtl.specular'] = mtl.specular
                    pipeline['mtl.shininess'] = mtl.shininess
                    pipeline['mtl.enabled'] = self._material_enabled
                    pipeline['time'] = app.current_window().elapsed_time
                    pipeline['mtl.use_diffuse_map'] = self._texture_enabled

                    if self._sub_mesh_count > 1:
                        pipeline['mtl.use_normal_map'] = sub_mesh.normal_map_enabled
                    else:
                        pipeline['mtl.use_normal_map'] = self._normal_map_enabled

                    pipeline['mtl.use_bump_map'] = self._bump_map_enabled
                    pipeline['mtl.use_parallax_map'] = self._parallax_map_enabled
                    pipeline['mtl.diffuse_map'] = 0
                    pipeline['mtl.bump_map'] = 1
                    pipeline['mtl.normal_map'] = 2

                    shadow_map_enabled = kwargs.get('shadow_map_enabled', False)
                    depth_map = kwargs.get('shadow_map', None)

                    pipeline.active_texture_unit(0)
                    with mtl.diffuse_map:
                        pipeline.active_texture_unit(1)
                        with mtl.bump_map:
                            pipeline.active_texture_unit(2)
                            with mtl.normal_map:
                                if shadow_map_enabled and depth_map is not None:
                                    pipeline.active_texture_unit(3)
                                    pipeline['shadow_map'] = 3
                                    with depth_map:
                                        with vao:
                                            gl.glDrawArrays(sub_mesh.topology.value, 0, record.vertex_count)
                                else:
                                    with vao:
                                        gl.glDrawArrays(sub_mesh.topology.value, 0, record.vertex_count)


def compute_all_tangents(positions: VertexAttribData,
                         texcoords: VertexAttribData,
                         triangles: np.ndarray,
                         topology: List[List[int]]) -> np.ndarray:
    """
    Compute the tangent vector for all vertices in the mesh.
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

    Args:
        positions (np.ndarray): The positions of all vertices in the mesh.
        texcoords (np.ndarray): The texture coordinates of all vertices in the mesh.
        triangles (np.ndarray): The triangles in the mesh, it is a 3D array with shape (n, 3, 3). First dimension is the
                                triangle index, second dimension is the attributes of the vertex in the triangle,
                                third dimension is the attribute values. For example, triangles[0, 0] is the vertices
                                of the first triangle, triangles[0, 1] is the texture coordinates of the first triangle
                                and triangles[0, 2] is the normals of the first triangle.
        topology (List[List[int]]): The topology information of the mesh. This is list containing indices of all
                                    triangles adjacent to a vertex.
    Returns:
        A list of tangent vectors for all vertices in the mesh.
    """
    count = positions.count
    tangents = np.zeros((count, 3), dtype=np.float32)
    for i in range(count):
        tangent = np.array([0.0, 0.0, 0.0], dtype=np.float32)
        for adj_tri in topology[i]:
            verts_pos = triangles[adj_tri][0]
            verts_tex = triangles[adj_tri][1]
            vs = [positions[verts_pos[0]], positions[verts_pos[1]], positions[verts_pos[2]]]
            uvs = [texcoords[verts_tex[0]], texcoords[verts_tex[1]], texcoords[verts_tex[2]]]
            delta_pos = [vs[1] - vs[0], vs[2] - vs[0]]
            delta_uv = [uvs[1] - uvs[0], uvs[2] - uvs[0]]
            det = delta_uv[0][0] * delta_uv[1][1] - delta_uv[0][1] * delta_uv[1][0]
            if det != 0.0:
                tangent += (delta_pos[0] * delta_uv[1][1] - delta_pos[1] * delta_uv[0][1]) / det
        norm = np.sqrt(np.sum(tangent ** 2))
        if norm != 0.0:
            tangents[i] = tangent / norm
    return tangents
