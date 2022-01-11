import logging
import timeit

from numba import njit, prange
import numpy as np

from .building import Component, Building
from .mesh import Mesh
from .entity import Entity
from .. import gl
from ..app.input import KeyCode, KeyModifier
from ..app import ui
from ..assets import default_asset_mgr
from ..camera import Camera
from ..graphics.lights.light import Light
from ..graphics.vertex_layout import VertexLayout, VertexAttrib, VertexAttribFormat
from ..graphics.buffer import VertexBuffer
from ..graphics.array import VertexArrayObject
from ..graphics.framebuffer import Framebuffer
from ..graphics.lights import PointLight, DirectionalLight
from ..misc import PaletteDefault
from ..math import Mat4, Vec3


class MeshEntity(Entity):
    def __init__(self, mesh: Mesh, cast_shadow=True):
        super().__init__()
        self.mesh = mesh
        self._is_drawable = True
        self._cast_shadow = cast_shadow

    def draw(self, shader=None, **kwargs):
        """
        Args:
            shader (ShaderProgram): Shader to use. (Override the shader of mesh)
            **kwargs: Extra uniforms that are going to be passed to shader
        """
        self.mesh.draw(shader=shader, **kwargs)


class Scene:
    """
    Scene manages the rendering of meshes, lights and cameras.
    """
    def __init__(self, window, entities=(), camera=None, light=None, draw_light=False,
                 bg_color=PaletteDefault.Background.as_color(), **kwargs):
        """
        Initialisation of a Scene object.

        Args:
            window:
                Specifies the window in which the scene will be presented

            entities (list of Meshes or instances of subclasses of Entity):
                All entity instances that you want to have in the Scene

            camera (Camera):
                Main camera of the scene.

            light (Light):
                Light inside

            bg_color (Color): screen clear color
            **kwargs:
        """
        self._window = window
        self._entities = []
        if len(entities) > 0:
            for entity in entities:
                if isinstance(entity, Mesh):
                    self._entities.append(MeshEntity(entity))
                elif isinstance(entity, Entity):
                    self._entities.append(entity)
                elif isinstance(entity, Component):
                    self._entities.append(entity)

        self._lights = [light] if light is not None else [PointLight()]
        # self._lights = [light] if light is not None else [DirectionalLight()]
        self._draw_light = draw_light
        if draw_light:
            self._light_spheres = [
                Mesh('models/uv_sphere.obj',
                     texture_enabled=False,
                     initial_transformation=Mat4.from_scale(Vec3(0.1, 0.1, 0.1))) for _ in self._lights]
        self._cameras = []
        self._main_camera = -1
        if camera is not None:
            self._cameras.append(camera)
            self._main_camera = 0
            self._window.attach_listeners(camera)

        self._window.attach_listeners(self)
        self._values = 1.0, 2.0, 3.0
        self._depth_map_framebuffer = Framebuffer(self._window.width, self._window.height, depth_shader_accessible=True)
        self._depth_map_pipeline = default_asset_mgr.get_or_create_pipeline('depth_map',
                                                                            vertex_shader='shaders/depth_map.vert',
                                                                            pixel_shader='shaders/depth_map.frag')
        self._energy_framebuffer = Framebuffer(self._window.width, self._window.height)
        self._energy_pipeline = default_asset_mgr.get_or_create_pipeline('energy_pipeline',
                                                                         vertex_shader='shaders/energy_map.vert',
                                                                         pixel_shader='shaders/energy_map.frag')
        self._is_wire_frame = False

    def set_main_camera(self, index):
        """
        Specifies the camera going to be used as the main camera.

        Args:
            index (int): index of cameras in the scene.

        Returns:
            None
        """
        if index != self._main_camera:
            self._switch_to_camera(index)

    @property
    def lights(self):
        return self._lights

    @property
    def main_camera(self):
        return self._cameras[self._main_camera]

    def create_camera(self, pos, look_at, up, fov_v=45.0, near=0.1, far=1000., degrees=True, zoom_enabled=False, safe_rotations=True) -> int:
        """
        Creates a new camera and return its index inside the scene.

        Args:
            pos (array like):
                Specifies the position of the camera

            look_at (array like):
                Specifies look at of the camera

            up (array like):
                Specifies up-vector of the camera

            fov_v (float):
                Specifies vertical field of view of the camera

            near (float):
                Specifies near clip plane

            far (float):
                Specifies far clip plane

            degrees (bool):
                Specifies how to treat input angles

            zoom_enabled (bool):
                Specifies whether to enable camera zoom or not

            safe_rotations:
                Specifies whether to enable safe rotations by disable certain cases

        Returns:
            int, index of the newly created camera inside Scene
        """
        camera = Camera(pos, look_at, up, self._window.width / self._window.height, fov_v, near, far, degrees, zoom_enabled, safe_rotations)
        camera.on_resize(self._window.width, self._window.height)
        self._cameras.append(camera)
        if len(self._cameras) == 1:
            self._main_camera = 0
            self._window.attach_listeners(self._cameras[0])

        return len(self._cameras) - 1

    def _switch_to_camera(self, to_idx):
        idx = self._main_camera
        self._window.detach_listeners(self._cameras[idx])
        self._main_camera = to_idx
        self._window.attach_listeners(self._cameras[to_idx])
        self._cameras[to_idx].on_resize(self._window.width, self._window.height)
        
    @property
    def depth_map_framebuffer(self):
        return self._depth_map_framebuffer

    def on_gui(self):

        if ui.tree_node('Light'):
            if self._lights[0].is_directional:
                ui.drag_float3('Position', *self._lights[0].position)
            else:
                _, self._lights[0].position = ui.drag_float3('Position', *self._lights[0].position)
            _, self._lights[0].color.rgb = ui.color_edit3('Color', *self._lights[0].color.rgb)
            if self._lights[0].is_directional:
                _, self._lights[0].direction = ui.drag_float3('Direction', *self._lights[0].direction)
            ui.tree_pop()

        if ui.tree_node('Camera'):
            if ui.button("Next Camera"):
                if len(self._cameras) > 0:
                    self._switch_to_camera((self._main_camera + 1) % len(self._cameras))
                    print(self._main_camera)
            ui.tree_pop()

        if ui.tree_node('Rendering'):
            _, self._is_wire_frame = ui.checkbox('Wireframe', self._is_wire_frame)
            ui.tree_pop()

        # ui.end()

    def on_key_press(self, key, mods):
        if key == KeyCode.D and mods == KeyModifier.Shift:
            main_cam = self._cameras[self._main_camera]
            self._depth_map_framebuffer.save_depth_attachment(main_cam.near, main_cam.far, self._lights[0].is_perspective)

        if key == KeyCode.C and mods == KeyModifier.Shift:
            self._depth_map_framebuffer.save_color_attachment()

    def energy_ratio_of(self, building: Building, comp: Component, light: Light = None):
        light = self._lights[0] if light is None else light
        model_mat = building.transform_of(comp)
        light_mat = light.matrix
        light_view_mat = light.view_matrix
        light_pos = light.position
        with self._energy_framebuffer:
            self._energy_framebuffer.enable_depth_test()
            self._energy_framebuffer.clear((1.0, 1.0, 1.0, 1.0))
            if comp.mesh.sub_mesh_count > 0:
                with self._energy_pipeline:
                    self._energy_pipeline['model_mat'] = model_mat
                    self._energy_pipeline['depth_map'] = 0
                    self._energy_pipeline['light_mat'] = light_mat
                    self._energy_pipeline['light_pos'] = light_pos
                    self._energy_pipeline['light_view_mat'] = light_view_mat
                    self._energy_pipeline['resolution'] = (self._window.width, self._window.height)
                    for idx, record in comp.mesh.render_records.items():
                        self._energy_pipeline.active_texture_unit(0)
                        with self._depth_map_framebuffer.depth_attachments[0]:
                            with comp.mesh._vertex_array_objects[record.vao_idx]:
                                gl.glDrawArrays(comp.mesh.sub_meshes[idx].topology.value, 0, record.vertex_count)
        energy_map = self._energy_framebuffer.save_color_attachment(save_as_image=False)
        energy_map = energy_map.reshape((-1, 4))
        visible_count, occluded_count, received_energy = _calc_energy(energy_map)
        visibility_ratio = visible_count / (occluded_count + visible_count)
        energy_ratio = received_energy / (occluded_count + visible_count)
        # print('occluded pixel count: ', occluded_count)
        # print('visible pixel count: ', visible_count)
        # print('visibility ratio: ', visibility_ratio)
        # print('energy ratio: ', energy_ratio)
        # print('energy: ', received_energy)

    def draw(self, shader=None, auto_shadow=False, **kwargs):
        """Draw every visible meshes in the scene.

        Args:
            shader (ShaderProgram): If specified, this will override the assigned shader of each mesh.
            auto_shadow (Bool): Specifies if the default shadow is enabled or not.
            **kwargs: Extra uniforms that are going to be passed to shader
        """
        light = self._lights[0]

        with_shadow, without_shadow = [], []
        for e in self._entities:
            (without_shadow, with_shadow)[int(e.cast_shadow and e.drawable)].append(e)

        current_polygon_mode = gl.glGetIntegerv(gl.GL_POLYGON_MODE)[0]

        if auto_shadow:
            # 1st pass: render to depth map
            if current_polygon_mode != gl.GL_LINE:
                with self._depth_map_framebuffer:
                    self._depth_map_framebuffer.enable_depth_test()
                    self._depth_map_framebuffer.clear(PaletteDefault.Background.as_color().rgba)
                    for e in with_shadow:
                        e.draw(light_mat=light.matrix,
                               near=light._sm_near,
                               far=light._sm_far,
                               is_persepctive=light._sm_is_perspective,
                               camera=self._cameras[self._main_camera],
                               shader=self._depth_map_pipeline,
                               **kwargs)

            # 2nd pass: render object as normal with shadow mapping
            gl.glClearColor(*PaletteDefault.BlueA.as_color().rgba)
            gl.glEnable(gl.GL_DEPTH_TEST)
            gl.glClear(gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT)
            if self._is_wire_frame:
                gl.glPolygonMode(gl.GL_FRONT_AND_BACK, gl.GL_LINE)
            else:
                gl.glPolygonMode(gl.GL_FRONT_AND_BACK, gl.GL_FILL)
            for e in with_shadow:
                e.draw(in_light_pos=light.position,
                       in_light_dir=light.direction if light.is_directional else Vec3(0.0),
                       is_directional=light.is_directional,
                       light_color=light.color.rgb,
                       light_mat=light.matrix,
                       camera=self._cameras[self._main_camera],
                       shader=shader,
                       shadow_map=self._depth_map_framebuffer.depth_attachments[0],
                       shadow_map_enabled=True,
                       **kwargs)

            for e in without_shadow:
                e.draw(in_light_pos=light.position,
                       in_light_dir=light.direction if light.is_directional else Vec3(0.0),
                       is_directional=light.is_directional,
                       light_color=light.color.rgb,
                       light_mat=light.matrix,
                       camera=self._cameras[self._main_camera],
                       shader=shader,
                       shadow_map_enabled=False,
                       **kwargs)
            gl.glPolygonMode(gl.GL_FRONT_AND_BACK, current_polygon_mode)
        else:
            if self._is_wire_frame:
                gl.glPolygonMode(gl.GL_FRONT_AND_BACK, gl.GL_LINE)
            else:
                gl.glPolygonMode(gl.GL_FRONT_AND_BACK, gl.GL_FILL)
            for e in self._entities:
                e.draw(in_light_pos=light.position,
                       in_light_dir=light.direction if light.is_directional else Vec3(0.0),
                       is_directional=light.is_directional,
                       light_color=light.color.rgb,
                       light_mat=light.matrix,
                       near=light._sm_near,
                       far=light._sm_far,
                       is_persepctive=light._sm_is_perspective,
                       camera=self._cameras[self._main_camera],
                       shader=shader,
                       **kwargs)
            gl.glPolygonMode(gl.GL_FRONT_AND_BACK, current_polygon_mode)

        if self._draw_light:
            for i, l in enumerate(self._lights):
                self._light_spheres[i].transformation = Mat4.from_translation(l.position)
                self._light_spheres[i].draw(camera=self._cameras[self._main_camera])


@njit(parallel=True, fastmath=True)
def _calc_energy(data):
    occluded_count = 0
    visible_count = 0
    received_energy = 0.0
    for i in prange(data.shape[0]):  # iterate over rows
        if data[i][0] == 255 and data[i][1] == 0 and data[i][2] == 0:
            occluded_count += 1
        if data[i][0] == 0 and data[i][2] == 0:
            visible_count += 1
            received_energy += data[i][1] / 255.0
    return visible_count, occluded_count, received_energy


