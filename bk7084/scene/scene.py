from collections.abc import Sequence

from .entity import Entity
from ..camera import Camera
from ..math import Vec3
from ..misc import PaletteDefault


class Scene:
    """
    Scene manages the rendering of meshes, lights and cameras.
    """

    def __init__(self, entities: Sequence[Entity], camera=None, light=None, bg_color=PaletteDefault.Background.as_color(), **kwargs):
        """
        Initialisation of a Scene object.

        Args:
            meshes (list of Meshes): all of Mesh instances that you want to view in the Scene
            camera:
            light:
            bg_color:
            **kwargs:
        """
        self._entities = entities
        self._lights = [light] if light is not None else []
        self._cameras = [camera] if camera is not None else []
        self._main_camera = 0

    def draw(self, camera_index=0):
        """Draw every visible meshes in the scene."""
        for e in self._entities:
            if e.drawable:
                e.draw()
