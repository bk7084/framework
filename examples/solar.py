import os.path as osp
from bk7084 import Window, app
from bk7084.app.input import KeyCode, KeyModifier
from bk7084.geometry import Triangle, Ray, Line, Box
from bk7084.math import Vec3, Mat3, Mat4
from bk7084.misc import PaletteSvg, PaletteDefault
from bk7084.graphics import draw
from bk7084.scene import Mesh


window = Window("Solar", width=1024, height=1024)
window.create_camera(Vec3(-100.0, 50.0, 0.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, zoom_enabled=True, zoom_speed=3.0)

planets = {
    'neptune': {
        'model': Mesh(osp.join('models/Neptune.obj')),
        'to_sun': 450.0,
    },
    'uranus': {
        'model': Mesh(osp.join('models/Uranus.obj')),
        'to_sun': 400.0,
    },
    'saturn': {
        'model': Mesh(osp.join('models/Saturn.obj')),
        'to_sun': 350.0,
    },
    'jupiter': {
        'model': Mesh(osp.join('models/Jupiter.obj')),
        'to_sun': 300.0,
    },
    'mars': {
        'model': Mesh(osp.join('models/Mars.obj')),
        'to_sun': 250.0,
    },
    'earth': {
        'model': Mesh(osp.join('models/earth.obj')),
        'to_sun': 200.0,
    },
    'moon': {
        'model': Mesh(osp.join('models/moon.obj')),
        'to_earth': 50.0,
    },
    'venus': {
        'model': Mesh(osp.join('models/Venus.obj')),
        'to_sun': 100.0,
    },
    'mercury': {
        'model': Mesh(osp.join('models/Mercury.obj')),
        'to_sun': 50.0,
    },
}

for planet in planets.values():
    planet['model'].texture_enabled = True

sun = Mesh(osp.join('models/sun.obj'))
sun.init_transform = Mat4.from_scale(Vec3(0.5))

earth = planets['earth']['model']
moon = planets['moon']['model']
moon.texture_enabled = False


time = 0.0


@window.event
def on_draw(dt):
    draw(sun)

    # neptune.reset_transform()
    # neptune.apply_transform(neptune_translation).then(Mat4.from_rotation_y(time * 35.0, True))
    # draw(neptune)
    #
    # uranus.reset_transform()
    # uranus.apply_transform(uranus_translation).then(Mat4.from_rotation_y(time * 30.0, True))
    # draw(uranus)

    earth.reset_transform()
    earth.apply_transform(Mat4.from_rotation_y(time))\
        .then(Mat4.from_translation(Vec3(planets['earth']['to_sun'], 0, 0)))\
        .then(Mat4.from_rotation_y(time))
    draw(earth)

    moon.reset_transform()
    moon.apply_transform(Mat4.from_rotation_y(time))\
        .then(Mat4.from_translation(Vec3(planets['moon']['to_earth'])))\
        .then(Mat4.from_rotation_y(time)).then(earth.transform)
    draw(moon)


@window.event
def on_update(dt):
    global time
    time += dt


app.init(window)
app.run()
