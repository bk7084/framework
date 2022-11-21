import itertools

# from bk7084 import app
from .. import app
from ..math import Vec4
from numba import njit, prange

from .building import Component, Building
from .mesh import Mesh
from .entity import Entity
from .. import gl
from ..app.input import KeyCode, KeyModifier
from ..app import ui
from ..assets import default_asset_mgr
from ..camera import Camera
from ..graphics.lights.light import Light
from ..graphics.framebuffer import Framebuffer
from ..graphics.lights import PointLight, DirectionalLight
from ..misc import PaletteDefault
from ..math import Mat4, Vec3


class MeshEntity(Entity):
    def __init__(self, mesh: Mesh, cast_shadow=True):
        super().__init__()
        self.mesh = mesh

    def draw(self, shader=None, **kwargs):
        """
        Args:
            shader (ShaderProgram): Shader to use. (Override the shader of mesh)
            **kwargs: Extra uniforms that are going to be passed to shader
        """
        self.mesh.draw(shader=shader, **kwargs)

    @property
    def meshes(self):
        return [(self.mesh, Mat4.identity())]


class Scene:
    """
    Scene manages the rendering of meshes, lights and cameras.
    """
    def __init__(self, window, entities=(), camera=None, num_lights=12, draw_light=False,
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

        self._lights = []
        delta = 180.0 / num_lights
        for i in range(num_lights):
            position = Mat4.from_rotation_z(delta * i, True) * Vec4(15.0, 1.0, 15.0, 1.0)
            self._lights.append(PointLight(Vec3(position)))
        self._current_light = 0

        self._draw_light = draw_light
        self._light_sphere = Mesh('light_sphere', 'models/uv_sphere.obj', texture_enabled=False,
                                  init_transform=Mat4.from_scale(Vec3(0.1, 0.1, 0.1)))
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
        self._rendering_info = self.extract_rendering_info()

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
    def current_light(self):
        return self._lights[self._current_light]

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

    def extract_rendering_info(self):
        # True --> cast shadow
        # False --> do not cast shadow
        info = {
            True: {},
            False: {},
        }
        for entity in self._entities:
            cast_shadow = entity.cast_shadow
            if entity.drawable:
                for mesh, transform in entity.meshes:
                    for pipeline_uuid, records in mesh.rendering_info.items():
                        if pipeline_uuid not in info[cast_shadow]:
                            info[cast_shadow][pipeline_uuid] = []
                        for record in records:
                            info[cast_shadow][pipeline_uuid].append((*record, transform))
        return info

    def load_mesh_entity(self, name, filename):
        assert filename is not None and name is not None
        entity = MeshEntity(Mesh(name, filename))
        self._entities.append(entity)
        return entity

    def on_gui(self):
        if ui.tree_node('Light'):
            if self._lights[self._current_light].is_directional:
                ui.drag_float3('Position', *self._lights[0].position)
            else:
                _, self._lights[0].position = ui.drag_float3('Position', *self._lights[0].position)
            _, self._lights[0].color.rgb = ui.color_edit3('Color', *self._lights[0].color.rgb)
            if self._lights[0].is_directional:
                _, self._lights[0].direction = ui.drag_float3('Direction', *self._lights[0].direction)
            if ui.button('Next light'):
                self._current_light += 1
                self._current_light %= len(self._lights)
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

    def on_update(self, dt):
        self._rendering_info = self.extract_rendering_info()

    def on_key_press(self, key, mods):
        if key == KeyCode.D and mods == KeyModifier.Shift:
            main_cam = self._cameras[self._main_camera]
            self._depth_map_framebuffer.save_depth_attachment(main_cam.near, main_cam.far, self._lights[0].is_perspective)

        if key == KeyCode.C and mods == KeyModifier.Shift:
            self._depth_map_framebuffer.save_color_attachment()

    @staticmethod
    def _compute_energy(energy_map):
        energy_map = energy_map.reshape((-1, 4))
        visible_count, occluded_count, received_energy = _calc_energy(energy_map)
        visibility_ratio = visible_count / (occluded_count + visible_count)
        energy_ratio = received_energy / (occluded_count + visible_count)
        # print('occluded pixel count: ', occluded_count)
        # print('visible pixel count: ', visible_count)
        # print('visibility ratio: ', visibility_ratio)
        # print('energy ratio: ', energy_ratio)
        # print('energy: ', received_energy)
        return energy_ratio

    def energy_of_building_component(self, building: Building, comp: Component, light: Light = None, save_energy_map=False):
        light = self._lights[self._current_light] if light is None else light
        with self._energy_framebuffer:
            self._energy_framebuffer.enable_depth_test()
            self._energy_framebuffer.clear((1.0, 1.0, 1.0, 1.0))
            comp.compute_energy(self._energy_pipeline, light, self._window.size,
                                self._depth_map_framebuffer.depth_attachments[0],
                                [building.transform_of(comp)])

        energy_map = self._energy_framebuffer.save_color_attachment(save_as_image=save_energy_map)
        return self._compute_energy(energy_map)

    def energy_of_building(self, building: Building, light: Light = None, save_energy_map=False, mesh_transform=Mat4.identity()):
        light = self._lights[self._current_light] if light is None else light

        with self._energy_framebuffer:
            self._energy_framebuffer.enable_depth_test()
            self._energy_framebuffer.clear((1.0, 1.0, 1.0, 1.0))
            if isinstance(building, Building):
                building.compute_energy(self._energy_pipeline, light,
                                        self._window.size,
                                        self._depth_map_framebuffer.depth_attachments[0])
            elif isinstance(building, Mesh):
                building.compute_energy(self._energy_pipeline, mesh_transform, light, self._window.size,
                                        self._depth_map_framebuffer.depth_attachments[0])
        energy_map = self._energy_framebuffer.save_color_attachment(save_as_image=save_energy_map)
        return self._compute_energy(energy_map)

    def draw_v2(self, shader=None, auto_shadow=False, **kwargs):
        light = self._lights[self._current_light]
        camera = self._cameras[self._main_camera]
        old_polygon_mode = gl.glGetIntegerv(gl.GL_POLYGON_MODE)[0]

        if auto_shadow:
            # two passes
            # 1st pass
            if old_polygon_mode != gl.GL_LINE:
                with self._depth_map_framebuffer:
                    gl.glEnable(gl.GL_DEPTH_TEST)
                    self._depth_map_framebuffer.clear(PaletteDefault.Background.as_color().rgba)
                    with self._depth_map_pipeline:
                        for _, records in self._rendering_info[True].items():
                            self._bind_light(self._depth_map_pipeline, light)
                            for record in records:
                                vao, topology, count, _, _, _, _, _, _, _, _, mesh_transform, transform = record
                                self._depth_map_pipeline['model_mat'] = transform * mesh_transform
                                with vao:
                                    gl.glDrawArrays(topology, 0, count)
            # second pass
            gl.glClearColor(*PaletteDefault.BlueA.as_color().rgba)
            gl.glEnable(gl.GL_DEPTH_TEST)
            gl.glClear(gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT)
            if self._is_wire_frame:
                gl.glPolygonMode(gl.GL_FRONT_AND_BACK, gl.GL_LINE)
            else:
                gl.glPolygonMode(gl.GL_FRONT_AND_BACK, gl.GL_FILL)

            for cast_shadow, info in self._rendering_info.items():
                for pipeline_uuid, records in info.items():
                    pipeline = default_asset_mgr.get_pipeline(pipeline_uuid)
                    with pipeline:
                        pipeline['time'] = app.current_window().elapsed_time
                        self._bind_light(pipeline, light)
                        self._bind_camera(pipeline, camera)
                        for record in records:
                            vao, topology, count, mtl, diffuse_map, shading_enabled, \
                            mtl_enabled, tex_enabled, normal_map_enabled, bump_map_enabled, \
                            parallax_map_enabled, mesh_transform, transform = record
                            pipeline['shading_enabled'] = shading_enabled
                            pipeline['model_mat'] = transform * mesh_transform
                            if cast_shadow:
                                pipeline['shadow_map_enabled'] = True
                            else:
                                pipeline['shadow_map_enabled'] = True
                            self._bind_material(pipeline, mtl, mtl_enabled, tex_enabled, normal_map_enabled,
                                                bump_map_enabled, parallax_map_enabled)
                            with vao:
                                pipeline.active_texture_unit(0)
                                with diffuse_map:
                                    pipeline.active_texture_unit(1)
                                    with mtl.bump_map:
                                        pipeline.active_texture_unit(2)
                                        with mtl.normal_map:
                                            pipeline.active_texture_unit(3)
                                            with self._depth_map_framebuffer.depth_attachments[0]:
                                                gl.glDrawArrays(topology, 0, count)

            gl.glPolygonMode(gl.GL_FRONT_AND_BACK, old_polygon_mode)
        else:
            # single pass
            if shader is not None:
                with shader:
                    self._bind_light(shader, light)
                    self._bind_camera(shader, camera)

            if self._is_wire_frame:
                gl.glPolygonMode(gl.GL_FRONT_AND_BACK, gl.GL_LINE)
            else:
                gl.glPolygonMode(gl.GL_FRONT_AND_BACK, gl.GL_FILL)

            for pipeline_uuid, records in itertools.chain(self._rendering_info[True].items(), self._rendering_info[False].items()):
                pipeline = default_asset_mgr.get_pipeline(pipeline_uuid)
                with pipeline:
                    pipeline['time'] = app.current_window().elapsed_time
                    self._bind_light(pipeline, light)
                    self._bind_camera(pipeline, camera)
                    for record in records:
                        vao, topology, count, mtl, diffuse_map, shading_enabled, \
                        mtl_enabled, tex_enabled, normal_map_enabled, bump_map_enabled,\
                        parallax_map_enabled, mesh_transform, transform = record
                        pipeline['shading_enabled'] = shading_enabled
                        pipeline['model_mat'] = transform * mesh_transform
                        self._bind_material(pipeline, mtl, mtl_enabled, tex_enabled, normal_map_enabled, bump_map_enabled, parallax_map_enabled)
                        with vao:
                            pipeline.active_texture_unit(0)
                            with diffuse_map:
                                pipeline.active_texture_unit(1)
                                with mtl.bump_map:
                                    pipeline.active_texture_unit(2)
                                    with mtl.normal_map:
                                        gl.glDrawArrays(topology, 0, count)
            gl.glPolygonMode(gl.GL_FRONT_AND_BACK, old_polygon_mode)

        if self._draw_light:
            self._light_sphere.transform = Mat4.from_translation(self._lights[self._current_light].position)
            self._light_sphere.draw(camera=self._cameras[self._main_camera])

    def _bind_light(self, pipeline, light):
        pipeline['in_light_pos'] = light.position
        pipeline['in_light_dir'] = light.direction if light.is_directional else Vec3(0.0)
        pipeline['is_directional'] = light.is_directional
        pipeline['light_color'] = light.color.rgb
        pipeline['light_mat'] = light.matrix
        pipeline['near'] = light._sm_near
        pipeline['far'] = light._sm_far
        pipeline['is_persepective'] = light._sm_is_perspective

    def _bind_material(self, pipeline, mtl, mtl_enabled, tex_enabled, normal_map_enabled, bump_map_enabled, parallax_map_enabled):
        pipeline['mtl.diffuse'] = mtl.diffuse
        pipeline['mtl.ambient'] = mtl.ambient
        pipeline['mtl.specular'] = mtl.specular
        pipeline['mtl.shininess'] = mtl.shininess
        pipeline['mtl.enabled'] = mtl_enabled
        pipeline['mtl.use_diffuse_map'] = tex_enabled
        pipeline['mtl.use_normal_map'] = normal_map_enabled
        pipeline['mtl.use_bump_map'] = bump_map_enabled
        pipeline['mtl.use_parallax_map'] = parallax_map_enabled

        pipeline['mtl.diffuse_map'] = 0
        pipeline['mtl.bump_map'] = 1
        pipeline['mtl.normal_map'] = 2
        pipeline['shadow_map'] = 3

    def _bind_camera(self, pipeline, camera):
        pipeline['view_mat'] = camera.view_matrix
        pipeline['proj_mat'] = camera.projection_matrix


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
