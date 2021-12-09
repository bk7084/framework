import os.path
import re

from ...assets import default_resolver


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
    def __init__(self, filepath, resolver=default_resolver):
        self._filepath = resolver.resolve(filepath)

    def read(self):
        with open(self._filepath) as file:
            return self._load_obj(file, resolver=PathResolver(self._filepath))

    @staticmethod
    def _load_obj(file, resolver=None, split_object=False, group_material=True, skip_materials=False):
        """
        Load a Wavefront OBJ file (only contain triangles).

        Only accepts objects with V/VT/VN

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
                'vertices': [],
                'texcoords': [],
                'normals': [],
                'indices': [],  # vertex index to draw the object as whole
                'faces': [
                    [
                        (0, 1, 2),  # vertex index
                        (0, 1, 2),  # vertex tex coord idx
                        (0, 1, 2),  # vertex normal idx
                    ],
                    ...
                ],
                'materials': [],
                'objects': [
                    {
                        'name: 'object_000'
                        'vertex_range': (start, end),
                        'index_range': (start, end),
                        'texcoord_range': (start, end),
                        'normal_range': (start, end),
                        'face_range': (start, end)
                        'materials': [
                            { 'name': 'material_name_000', f_range=[face_idx_start, face_idx_end] },
                            { 'name': 'material_name_001', f_range=[face_idx_start, face_idx_end] }
                        ],
                        'vertex_format': 'V', 'V_T', 'V_N', 'V_T_N'
                    },
                    {
                        'name': 'object_001'
                        ...
                    },
                ]
            }
        """
        positions = []
        texcoords = []
        indices = []
        normals = []
        faces = []
        objects = []

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

        default_object = 'default_object_{}'

        c_obj_idx = -1
        c_mtl_idx = -1

        for line in content.splitlines():
            # split by whitespace
            line = line.split()

            # skip blank line
            if len(line) == 0:
                continue

            key, values_str = line[0], line[1:]

            diff = lambda arr: arr[1] - arr[0]

            if key in ('o', 'g'):
                c_obj_idx += 1
                # parse object name.
                obj_name = '_'.join(values_str)
                # in case of an empty name get from the default object name
                obj_name = obj_name if len(obj_name) > 0 else default_object.format(c_obj_idx)
                # initialise a new object
                objects.append({
                    'name': obj_name,
                    'vertex_range': [len(positions), -1],
                    'index_range': [len(indices) * 3, -1],
                    'texcoord_range': [len(texcoords), -1],
                    'normal_range': [len(normals), -1],
                    'face_range': [len(faces), -1],
                    'materials': [],
                    'vertex_format': ''
                })

                # save information for last parsed object before start reading next one
                if c_obj_idx >= 1:
                    objects[c_obj_idx - 1]['vertex_range'][1] = len(positions)
                    objects[c_obj_idx - 1]['index_range'][1] = len(indices) * 3
                    objects[c_obj_idx - 1]['texcoord_range'][1] = len(texcoords)
                    objects[c_obj_idx - 1]['normal_range'][1] = len(normals)
                    objects[c_obj_idx - 1]['face_range'][1] = len(faces)

                    # if the current object doesn't have a material
                    if len(objects[c_obj_idx - 1]['materials']) == 0:
                        objects[c_obj_idx - 1]['materials'].append({
                            'name': 'default_material',
                            'f_range': objects[c_obj_idx - 1]['face_range']
                        })

                    objects[c_obj_idx - 1]['materials'][c_mtl_idx]['f_range'][1] = len(faces)

                    vertex_format = 'V'

                    if diff(objects[c_obj_idx - 1]['texcoord_range']) > 0:
                        vertex_format += '_T'

                    if diff(objects[c_obj_idx - 1]['normal_range']) > 0:
                        vertex_format += '_N'

                    objects[c_obj_idx - 1]['vertex_format'] = vertex_format

                # reset material index
                c_mtl_idx = -1

            elif key == 'v':
                positions.append([float(x) for x in values_str])

            elif key == 'vt':
                texcoords.append([float(x) for x in values_str])

            elif key == 'vn':
                normals.append([float(x) for x in values_str])

            elif key == 'f':
                # parse line into a list
                parsed = [tuple(int(e) - 1 for e in filter(lambda x: x != '', c.split('/'))) for c in values_str]
                vertex_count = len(parsed)
                if vertex_count < 3:
                    # skip the face with less 3 vertices
                    continue
                else:
                    # in case of a face with more than 3 points, apply the simplest triangulation
                    for i in range(0, len(parsed) - 2):
                        faces.append(list(zip(*[parsed[0], *parsed[i + 1: i + 3]])))

            elif key == 'usemtl':
                c_mtl_idx += 1
                mtl_name = '_'.join(values_str)
                mtl_name = mtl_name if len(mtl_name) > 0 else default_object.format(c_obj_idx)
                objects[c_obj_idx]['materials'].append({
                    'name': mtl_name,
                    'f_range': [len(faces), -1]
                })

                # save information for last mtl before start reading next one
                if c_mtl_idx >= 1:
                    objects[c_obj_idx]['materials'][c_mtl_idx - 1]['f_range'][1] = len(faces)

            else:
                # ignoring s
                pass

        objects[c_obj_idx]['vertex_range'][1] = len(positions)
        objects[c_obj_idx]['index_range'][1] = len(indices)
        objects[c_obj_idx]['texcoord_range'][1] = len(texcoords)
        objects[c_obj_idx]['normal_range'][1] = len(normals)
        objects[c_obj_idx]['face_range'][1] = len(faces)
        objects[c_obj_idx]['materials'][c_mtl_idx]['f_range'][1] = len(faces)

        vertex_format = 'V'

        if diff(objects[c_obj_idx - 1]['texcoord_range']) > 0:
            vertex_format += '_T'

        if diff(objects[c_obj_idx - 1]['normal_range']) > 0:
            vertex_format += '_N'

        objects[c_obj_idx]['vertex_format'] = vertex_format

        return {
            'vertices': positions,
            'texcoords': texcoords,
            'normals': normals,
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



