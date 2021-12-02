from pprint import pprint

from bk7084 import Window, app
from bk7084.app.window.input import KeyCode
from bk7084.geometry import Triangle, Ray, Line, Box
from bk7084.math import Vec3, Mat3, Mat4
from bk7084.misc import PaletteSvg, PaletteDefault
from bk7084.graphics import draw

# Setup window and add camera
from bk7084.scene import Mesh

from bk7084.scene.loader.obj import WavefrontReader

window = Window("BK7084: Simple Scene", width=1024, height=1024)
window.create_camera(Vec3(0, 0.0, 10.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)
window._camera.zoom_enabled = True

bus = Mesh("./models/bus.obj")
car = Mesh('./models/car.obj', color=PaletteDefault.BlueB.as_color())
car.apply_transformation(Mat4.from_translation(Vec3(-4.0, 0.0, 0.0)))
cow = Mesh("./models/spot_cow.obj")
cow.apply_transformation(Mat4.from_translation(Vec3(4.0, 0.0, 0.0)))

animate = True


@window.event
def on_draw(dt):
    draw(car, cow, bus)


@window.event
def on_key_press(key, mods):
    global animate
    if key == KeyCode.A:
        animate = not animate


@window.event
def on_update(dt):
    if animate:
        bus.apply_transformation(Mat4.from_axis_angle(Vec3.unit_y(), 45.0 * dt, True))
        car.apply_transformation(Mat4.from_axis_angle(Vec3.unit_y(), 45.0 * dt, True))
        cow.apply_transformation(Mat4.from_axis_angle(Vec3.unit_y(), 45.0 * dt, True))



app.init(window)
app.run()
