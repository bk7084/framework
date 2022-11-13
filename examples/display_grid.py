import imgui

from bk7084 import Window, app, Camera
from bk7084.app import ui
from bk7084.app.input import KeyCode
from bk7084.graphics import draw
from bk7084.math import Vec3, Mat4

# Setup window and add camera
from bk7084.scene import Mesh, Scene

window = Window("BK7084: Simple Scene", width=600, height=600)

model = Mesh("./models/spot_cow.obj")
scene = Scene(window, [model], draw_light=True)
scene.create_camera(Vec3(2, 1.0, 2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, zoom_enabled=True, safe_rotations=False)
scene.create_camera(Vec3(-2, 1.0, 2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, zoom_enabled=True, safe_rotations=False)
scene.create_camera(Vec3(2, 1.0, -2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, zoom_enabled=True, safe_rotations=False)

animate = False


@window.event
def on_draw(dt):
    # scene.draw()
    scene.draw_v2()


@window.event
def on_key_press(key, mods):
    global animate
    if key == KeyCode.A:
        animate = not animate


@window.event
def on_update(dt):
    if animate:
        pass
        model.apply_transform(Mat4.from_axis_angle(Vec3.unit_y(), 45.0 * dt, True))


@window.event
def on_gui():
    if ui.tree_node('Mesh'):
        if ui.radio_button('normal map', model.normal_map_enabled):
            model.normal_map_enabled = not model.normal_map_enabled

        if ui.radio_button('bump map', model.bump_map_enabled):
            model.bump_map_enabled = not model.bump_map_enabled

        if ui.radio_button('parallax map', model.parallax_map_enabled):
            model.parallax_map_enabled = not model.parallax_map_enabled
        ui.tree_pop()


app.init(window)
app.run()
