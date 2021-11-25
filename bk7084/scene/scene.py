from ..camera import Camera
from ..math import Vec3
from ..misc import PaletteDefault


class Scene:
    """
    Scene manages the rendering of meshes, lights can cameras.
    """

    def __init__(self, camera=None, light=None, clear_color=PaletteDefault.Background.as_color(), **kwargs):
        """
        Initialization of a Scene object.

        Args:
            camera:
            light:
            **kwargs:
        """
        self._meshes = []
        self._camera = Camera(Vec3(0, 0.0, 10.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)
        self._lights = []

    def draw(self):
        """Draw every visible meshes in the scene."""
        with self._camera, self._lights:
            for mesh in self._meshes:
                try:
                    mesh.draw()
                except AttributeError:
                    pass
