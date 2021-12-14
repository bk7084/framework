from bk7084 import Window, app, Camera
from bk7084.app.input import KeyCode
from bk7084.math import Vec3, Mat4

# Setup window and add camera
from bk7084.scene import Mesh, Scene

window = Window("BK7084: Simple Scene", width=1024, height=1024)

cow = Mesh("./models/spot_cow.obj")

scene = Scene(window, [cow])
scene.create_camera(Vec3(2, 1.0, 2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)
scene.create_camera(Vec3(-2, 1.0, 2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)
scene.create_camera(Vec3(2, 1.0, -2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)

animate = False


@window.event
def on_draw(dt):
    scene.draw()


@window.event
def on_key_press(key, mods):
    global animate
    if key == KeyCode.A:
        animate = not animate


@window.event
def on_update(dt):
    if animate:
        pass
        cow.apply_transformation(Mat4.from_axis_angle(Vec3.unit_y(), 45.0 * dt, True))


app.init(window)
app.run()
