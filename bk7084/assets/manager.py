import enum
import hashlib
import logging
from collections import namedtuple

from .resolver import default_resolver
from .image import Image
from .. import gl
from ..graphics.program import ShaderProgram
from ..graphics.shader import Shader, ShaderType
from ..graphics.material import MaterialData
from ..graphics.texture import TextureWrapMode, FilterMode, TextureKind, Texture
from ..scene import mesh


class AssetKind(enum.Enum):
    Image = 0
    Texture = 1
    Material = 2


PipelineRecord = namedtuple('PipelineRecord', ['pipeline', 'vs', 'ps'])


class NameManager:
    def __init__(self):
        self._name_map = {}

    def get_name(self, name):
        if name not in self._name_map:
            self._name_map[name] = 0
        else:
            self._name_map[name] += 1

        return f'{name}_{self._name_map[name]:03}'


class AssetManager:
    def __init__(self, resolver=default_resolver):
        self._resolver = resolver
        self._materials = {}
        self._textures = {}
        self._images = {}
        self._shaders = {}
        self._pipelines = {}
        self._models = {}

    def get_or_create_image(self, image_path):
        """
        Returns the requested image. Load the image if it doesn't exist.

        Args:
            image_path (str):

        Returns:
            Pillow.Image
        """
        path = self._resolver.resolve(image_path)
        if path not in self._images:
            logging.info(f'-- Create image {path}')
            image = Image.open(path)
            self._images[path] = image

        logging.info(f'Load image <{path}>')
        return self._images[path]

    def get_or_create_texture(self, image_path, kind=TextureKind.DiffuseMap, target=gl.GL_TEXTURE_2D,
                              wrap_mode=TextureWrapMode.Repeat, filter_mode=FilterMode.Linear):
        path = self._resolver.resolve(image_path)
        image = self.get_or_create_image(path)
        texture_name = f'{path}_{kind.name}'
        if texture_name not in self._textures:
            logging.info(f'-- Create texture with image <{path}>')
            texture = Texture(path, image, kind, target, wrap_mode, filter_mode)
            self._textures[texture_name] = texture

        logging.info(f'Load texture <{texture_name}>')
        return self._textures[texture_name]

    def get_or_create_material(self, name, ambient=(0.8, 0.8, 0.8), diffuse=(0.8, 0.8, 0.8), specular=(1.0, 1.0, 1.0),
                               shininess=1.0, ior=1.0, dissolve=1.0, illum=2, **kwargs) -> MaterialData:
        """
        Args:
            name (str): name of the material
            ambient (array of 3 elements): ambient color
            diffuse (array of 3 elements): diffuse color
            specular (array of 3 elements): specular color
            illum (int): illumination model
            shininess (float): shininess of specular component
            dissolve (float): opacity
            ior (float): refractive index
            **kwargs:
                diffuse_map_path (str):
                    Specifies where to find the texture used as diffuse map.
                bump_map_path (str):
                    Specifies where to find the texture used as bump map.
                normal_map_path (str):
                    Specifies where to find the texture used as normal map.
        Returns:

        """
        if name not in self._materials:
            logging.info(f'-- Create material <{name}>')
            diffuse_map_path = kwargs.get('diffuse_map_path', None)
            if diffuse_map_path is None:
                diffuse_map_path = self._resolver.resolve('textures/checker.png')
            else:
                diffuse_map_path = self._resolver.resolve(diffuse_map_path)

            bump_map_path = kwargs.get('bump_map_path')
            if bump_map_path is None:
                bump_map_path = self._resolver.resolve('textures/checker_bump.png')
            else:
                bump_map_path = self._resolver.resolve(bump_map_path)

            normal_map_path = kwargs.get('normal_map_path')
            if normal_map_path is None:
                normal_map_path = self._resolver.resolve('textures/checker_normal.png')
            else:
                normal_map_path = self._resolver.resolve(normal_map_path)

            diffuse_map = self.get_or_create_texture(diffuse_map_path)
            bump_map = self.get_or_create_texture(bump_map_path, TextureKind.BumpMap)
            normal_map = self.get_or_create_texture(normal_map_path, TextureKind.NormalMap)
            material = MaterialData(name, diffuse_map, bump_map, normal_map, ambient, diffuse, specular, shininess, ior,
                                    dissolve, illum)
            self._materials[name] = material

        logging.info(f'Load material <{name}>')
        return self._materials[name]

    def get_or_create_shader(self, shader: str, kind: ShaderType, is_file=False):
        _, shader = self._get_or_create_shader(shader, kind, is_file)
        return shader

    def _get_or_create_shader(self, shader: str, kind: ShaderType, is_file=False):
        key = ''
        if is_file:
            key = self._resolver.resolve(shader)
            if key not in self._shaders:
                logging.info(f'-- Create shader from file <{key}>')
                with open(key, 'rt') as file:
                    self._shaders[key] = Shader(kind, code=file.read(), origin=key, is_file=False)
        else:
            key = hashlib.sha256(shader.encode('utf-8')).hexdigest()
            if key not in self._shaders:
                logging.info(f'-- Create shader from src <{key}>')
                self._shaders[key] = Shader(kind, code=shader, origin=f'<string#{key}>', is_file=False)

        return key, self._shaders[key]

    def get_or_create_pipeline(self, name: str, vertex_shader: str = None, pixel_shader: str = None) -> PipelineRecord:
        """

        Args:
            name (str): Specifies the name of the pipline (shader program).
            vertex_shader (str): Path string or code string of a vertex shader.
            pixel_shader (str): Path string or code string of a vertex shader.

        Returns:
            ShaderProgram
        """
        if name not in self._pipelines:
            logging.info(f'-- Create pipeline <{name}>')

            if vertex_shader is not None and pixel_shader is not None:
                is_file_vs = ';' not in vertex_shader
                is_file_ps = ';' not in pixel_shader

                vs_name = hashlib.sha256(
                    vertex_shader.encode('utf-8')).hexdigest() if not is_file_vs else self._resolver.resolve(
                    vertex_shader)
                ps_name = hashlib.sha256(
                    pixel_shader.encode('utf-8')).hexdigest() if not is_file_ps else self._resolver.resolve(
                    pixel_shader)

                for n, r in self._pipelines.items():
                    if r.vs == vs_name and r.ps == ps_name:
                        logging.info(f'Shaders exist in pipeline <{n}>')
                        return self._pipelines[n].pipeline

            if vertex_shader is None:
                vs_name, vs = self._get_or_create_shader('shaders/default.vert', ShaderType.Vertex, True)
            else:
                is_file_vs = ';' not in vertex_shader
                vs_name, vs = self._get_or_create_shader(vertex_shader, ShaderType.Vertex, is_file_vs)

            if pixel_shader is None:
                ps_name, ps = self._get_or_create_shader('shaders/default.frag', ShaderType.Pixel, True)
            else:
                is_file_ps = ';' not in pixel_shader
                ps_name, ps = self._get_or_create_shader(pixel_shader, ShaderType.Fragment, is_file_ps)

            self._pipelines[name] = PipelineRecord(ShaderProgram(vs, ps), vs_name, ps_name)

        logging.info(f'Load pipeline <{name}>')
        return self._pipelines[name].pipeline

    def get_pipeline(self, uuid):
        if len(self._pipelines) > 0:
            for record in self._pipelines.values():
                if record.pipeline.uuid == uuid:
                    return record.pipeline
        return None

    def get_or_load_wavefront_obj(self, filepath: str, resolver=None) -> dict:
        """
        Load a model from a file path if it is not loaded yet.

        Args:
            resolver: Path resolver.
            filepath (str): Path string of a model file.

        Returns:
            Model
        """
        resolver = resolver if resolver is not None else self._resolver
        path = resolver.resolve(filepath)
        logging.info(f'-- Getting model <{path}>')
        if path not in self._models:
            logging.info(f'  -- Loading... <{path}>')
            from ..scene.loader.obj import WavefrontReader
            reader = WavefrontReader(path)
            self._models[path] = reader.read()
        return self._models[path]


default_asset_mgr: AssetManager = AssetManager()
default_name_mgr: NameManager = NameManager()
