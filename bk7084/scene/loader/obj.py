import os.path
import re


def remove_comments(comment_symbol, text):
    return re.sub(f'{comment_symbol}.*', '', text).strip()


class PathResolver:
    def __init__(self, filepath):
        self._base = os.path.dirname(os.path.abspath(filepath))

    def resolve(self, filepath) -> str:
        final_path = filepath

        if not os.path.isabs(filepath):
            final_path = os.path.abspath(os.path.join(self._base, filepath))

        return final_path


class WavefrontReader:
    def __init__(self, filepath):
        if not os.path.isfile(filepath):
            raise ValueError(f"File {filepath} does not exist.")
        self._filepath = os.path.abspath(filepath)

    def read(self):
        with open(self._filepath) as file:
            return self._load_obj(file, resolver=PathResolver(self._filepath))

    @staticmethod
    def _load_obj(file, resolver=None, split_object=False, group_material=True, skip_materials=False):
        """
        Load a Wavefront OBJ file (only contain triangles).

        Only accepts objects with V/VT/VN.

        Args:
            file (file like object):
                Contains OBJ data.

            resolver (PathResolver):
                Resolve the path related while loading the OBJ file.

            split_object (bool):
                Split meshes at each `o` declared in file.

            group_material (bool):
                Group faces that share the same material into the same mesh.

            skip_materials (bool):
                Don't load any materials.

        Returns:
            dict with following format:
            {
                # data are stored following the order of objects.
                'load_default_material': False
                'vertices': [],
                'texcoords': [],
                'normals': [],
                'indices': [],  # vertex index to draw the object as whole
                'face_normals': [],
                'faces': [
                    [
                        (0, 1, 2),  # vertex index
                        (0, 1, 2),  # vertex tex coord idx
                        (0, 1, 2),  # vertex normal idx
                    ],
                    ...
                ],
                'materials': [],
                'objects': {
                    'object_000': {
                        'vertex_range': (start, end),
                        'index_range': (start, end),
                        'texcoord_range': (start, end),
                        'normal_range': (start, end),
                        'fnormal_range': (start, end),
                        'face_range': (start, end)
                        'materials': {
                            'material_name_000': (face_idx_start, face_idx_end)
                            'material_name_001': (face_idx_start, face_idx_end)
                        },
                        'vertex_format': 'V', 'V_T', 'V_N', 'V_T_N'
                    },
                    'object_001': {
                        ...
                    },
                }
            }
        """
        positions = []
        texcoords = []
        indices = []
        normals = []
        faces = []
        objects = {}

        content = re.sub('#.*', '', file.read()).strip()  # remove the comments

        materials = {}

        for matched in re.finditer('mtllib\s(?P<mtl>.*)', content):
            if resolver:
                mtl_path = resolver.resolve(matched.group('mtl'))
            else:
                mtl_path = os.path.abspath(matched.group('mtl'))

            if os.path.exists(mtl_path):
                materials = WavefrontReader._parse_materials(mtl_path, resolver)
            else:
                print(f'Material file {mtl_path} not exists.')

        def fill_vertex_related(start, end, prop, objs, obj_name):
            # [start, end)
            objs[obj_name][prop] = (start, end)

        # parse vertices and form object
        last_obj_name, curr_obj_name, curr_mtl_name, curr_vtx_layout = 'default_object_000', 'default_object_000', 'default_material', ''
        v_start, vt_start, vn_start, f_start = 0, 0, 0, 0
        vertex_format = [False, False, False]

        for line in content.splitlines():
            components = line.split()

            if len(components) == 0:
                continue

            key = components[0]
            if key == 'v':
                positions.append([float(x) for x in components[1:]])
                if vertex_format[0] is not True:
                    vertex_format[0] = True
            elif key == 'vt':
                texcoords.append([float(x) for x in components[1:]])
                if vertex_format[1] is not True:
                    vertex_format[1] = True
            elif key == 'vn':
                normals.append([float(x) for x in components[1:]])
                if vertex_format[2] is not True:
                    vertex_format[2] = True
            elif key == 'o' or key == 'g':
                if curr_obj_name != 'default_object_000':
                    fill_vertex_related(v_start, len(positions), 'vertex_range', objects, curr_obj_name)
                    fill_vertex_related(vt_start, len(texcoords), 'texcoord_range', objects, curr_obj_name)
                    fill_vertex_related(vn_start, len(normals), 'normal_range', objects, curr_obj_name)
                    if len(positions) > v_start:
                        curr_vtx_layout += 'V'
                    if len(texcoords) > vt_start:
                        curr_vtx_layout += 'T'
                    if len(texcoords) > vn_start:
                        curr_vtx_layout += 'N'

                    objects[curr_obj_name]['vertex_layout'] = curr_vtx_layout

                    v_start = len(positions)
                    vt_start = len(texcoords)
                    vn_start = len(normals)

                last_obj_name = curr_obj_name
                curr_obj_name = '_'.join(components[1:])
                objects[curr_obj_name] = {}
                curr_vtx_layout = ''
                vertex_format = [False, False, False]

            elif key == 'usemtl':
                mtl_name = '_'.join(components[1:])

                if 'materials' not in objects[curr_obj_name]:
                    objects[curr_obj_name]['materials'] = {}

                # save material information for last read object.
                if mtl_name != curr_mtl_name and curr_mtl_name != 'default_material':
                    objects[last_obj_name]['materials'][curr_mtl_name] = (f_start, len(faces))
                    objects[last_obj_name]['index_range'] = (f_start * 3, len(faces) * 3)
                    f_start = len(faces)

                curr_mtl_name = mtl_name

            elif key == 'f':
                # TODO: triangulate
                # [(v0, t0, n0), (v1, t1, n1), (v2, t2, n2)]
                parsed = [tuple(int(e) - 1 for e in filter(lambda x: x != '', c.split('/'))) for c in components[1:]]
                n = vertex_format.count(True)
                if len(parsed) == 3:
                    # [(v0, v1, v2), (t0, t1, t2), (n0, n1, n2)]
                    face = [tuple(p[i] for p in parsed[:3]) for i in range(0, n)]
                    faces.append(face)
                    indices.append(face[0])
                elif len(parsed) == 4:
                    # [(v0, v1, v2), (t0, t1, t2), (n0, n1, n2)]
                    face0 = [tuple(p[i] for p in parsed[:3]) for i in range(0, n)]
                    face1 = [tuple(p[i] for p in [*parsed[2:], parsed[0]]) for i in range(0, n)]
                    faces.append(face0)
                    faces.append(face1)
                    indices.append(face0[0])
                    indices.append(face1[0])
                else:
                    raise ValueError(f'Cannot import face with {len(parsed)} points.')

        # register material for last read object
        if curr_obj_name not in objects:
            objects[curr_obj_name] = {}
            objects[curr_obj_name]['materials'] = {}

        objects[curr_obj_name]['materials'][curr_mtl_name] = (f_start, len(faces))
        objects[curr_obj_name]['index_range'] = (f_start * 3, len(faces) * 3)

        fill_vertex_related(v_start, len(positions), 'vertex_range', objects, curr_obj_name)
        fill_vertex_related(vt_start, len(texcoords), 'texcoord_range', objects, curr_obj_name)
        fill_vertex_related(vn_start, len(normals), 'normal_range', objects, curr_obj_name)

        if len(positions) > v_start:
            curr_vtx_layout += 'V'
        if len(texcoords) > vt_start:
            curr_vtx_layout += 'T'
        if len(normals) > vn_start:
            curr_vtx_layout += 'N'

        objects[curr_obj_name]['vertex_layout'] = curr_vtx_layout

        return {
            'load_default_material': True if len(materials) == 0 else False,
            'vertices': positions,
            'texcoords': texcoords,
            'normals': normals,
            'face_normals': [],
            'indices': indices,
            'faces': faces,
            'materials': materials,
            'objects': objects,
        }

    @staticmethod
    def _parse_materials(filepath, resolver: PathResolver = None) -> dict:
        """
        Parse a MTL file.

        Args:
            resolver (PathResolver):
                Help finding the correct file path to textured file (in needed).

            filepath (str):
                File path to mtl file.

        Returns:
            dict of materials indexed by its name:
            {
                'material_name_000': {
                    'Ns': ...,
                    'Ka': ...,
                    ...
                }
            }
        """
        mtl_props = ['Kd', 'Ka', 'Ks', 'Ns', 'Ni', 'Illum', 'd', ]

        materials = {}

        with open(filepath) as file:
            lines = remove_comments('#', file.read()).strip().splitlines()

            current_material_name = None
            # parse mtl file line by line
            for line in lines:
                components = line.strip().split()
                if len(components) <= 1:
                    continue
                key = components[0]
                if key == 'newmtl':
                    # new material
                    current_material_name = '_'.join(components[1:])
                    materials[current_material_name] = {}
                elif key == 'map_Kd':
                    # texture map
                    filepath = ''.join(components[1:])
                    materials[current_material_name]['map_Kd'] = filepath if resolver is None else resolver.resolve(filepath)
                else:
                    # other properties
                    value = [float(x) for x in components[1:]]
                    if len(value) == 1:
                        value = value[0]
                    materials[current_material_name][key] = value

            return materials



