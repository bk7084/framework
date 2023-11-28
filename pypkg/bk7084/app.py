__all__ = ['App', 'Window', 'LightType', 'Color', 'IllumModel']


import enum
import inspect
from bk7084.bkfw import Window, Color, IllumModel
from bk7084.bkfw import PyAppState
from bk7084.bkfw import run_main_loop

from bk7084.math import Vec3, Mat4
from bk7084.mesh import Material, Mesh


class LightType(enum.Enum):
    Directional = 0
    Point = 1


class App(PyAppState):
    """The main application class."""
    def __new__(cls):
        return super().__new__(cls)

    def event(self, *args):
        """Decorator for attaching event handlers to the window.

        Usage:

        @app.event
        def on_update(dt, input):
            pass

        @app.event('on_resize')
        def random_name(width, height):
            pass
        """
        if len(args) == 0:  # @window.event()
            def decorator(fn):
                self.attach_event_handler(fn.__name__, fn)
                return fn
            return decorator
        elif inspect.isroutine(args[0]):  # @window.event
            fn = args[0]
            self.attach_event_handler(fn.__name__, fn)
            return args[0]
        elif type(args[0]) in (str,):  # @window.event('on_resize')
            def decorator(fn):
                self.attach_event_handler(args[0], fn)
            return decorator

    def add_point_light(self, pos: Vec3, color: Color, show_light: bool = False):
        """Adds a point light to the scene.

        Args:
            pos (Vec3): Position of the light.
            color (Color): Color of the light.
            show_light (bool, optional): Whether to show the light object. Defaults to False.
        """
        entity = super().add_point_light_py(pos, color)
        if show_light:
            mat = Material()
            mat.diffuse = color
            mat.illum_model = IllumModel.DiffuseNoShading
            mesh = Mesh.create_sphere(0.1, 16, 16)
            mesh.set_material(mat)
            sphere = self.add_mesh(mesh)
            sphere.set_visible(True)
            sphere.set_transform(Mat4.from_translation(pos))
        return entity

    def run(self, builder: Window):
        """Starts the main loop."""
        run_main_loop(self, builder)
