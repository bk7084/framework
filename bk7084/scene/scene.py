from .mesh import Mesh
from .entity import Entity
from ..app import ui
from ..camera import Camera
from ..graphics.lights import PointLight
from ..misc import PaletteDefault
from ..math import Mat4, Vec3


class MeshEntity(Entity):
    def __init__(self, mesh: Mesh):
        super().__init__()
        self.mesh = mesh
        self._is_drawable = True

    def draw(self, shader=None, **kwargs):
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

        self._lights = [light] if light is not None else [PointLight()]
        self._draw_light = draw_light
        if draw_light:
            self._light_boxes = [Mesh('../assets/models/cube.obj') for l in self._lights]
        self._cameras = []
        self._main_camera = -1
        if camera is not None:
            self._cameras.append(camera)
            self._main_camera = 0
            self._window.attach_listeners(camera)

        self._window.attach_listeners(self)
        self._values = 1.0, 2.0, 3.0

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

    def on_gui(self):
        ui.new_frame()

        ui.begin("Controls")
        changed, self._lights[0].position = ui.drag_float3('Light', *self._lights[0].position)

        if ui.button("Next Camera"):
            if len(self._cameras) > 0:
                self._switch_to_camera((self._main_camera + 1) % len(self._cameras))
                print(self._main_camera)
        ui.end()

        ui.end_frame()
        ui.render()

    def draw(self):
        """Draw every visible meshes in the scene."""
        if self._draw_light:
            for i, l in enumerate(self._lights):
                self._light_boxes[i].transformation = Mat4.from_translation(l.position)
                self._light_boxes[i].draw()
        for e in self._entities:
            if e.drawable:
                e.draw(in_light_pos=self._lights[0].position)
