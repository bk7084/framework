from ..camera import Camera
from ..math import Vec3
from ..misc import PaletteDefault


class Scene:
    """
    Scene manages the rendering of meshes, lights and cameras.
    """

    def __init__(self, entities, camera=None, light=None, bg_color=PaletteDefault.Background.as_color(), **kwargs):
        """
        Initialisation of a Scene object.

        Args:
            meshes (list of Meshes): all of Mesh instances that you want to view in the Scene
            camera:
            light:
            bg_color:
            **kwargs:
        """
        self._entities = []
        self._lights = []
        self._cameras = []
        self._main_camera = 0

    def draw(self, camera_index=0):
        """Draw every visible meshes in the scene."""
        pass
