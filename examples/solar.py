import os.path as osp
from bk7084 import Window, app
from bk7084.app.input import KeyCode, KeyModifier
from bk7084.geometry import Triangle, Ray, Line, Box
from bk7084.math import Vec3, Mat3, Mat4
from bk7084.misc import PaletteSvg, PaletteDefault
from bk7084.graphics import draw
from bk7084.scene import Mesh


window = Window("Solar", width=1024, height=1024)
window.create_camera(Vec3(-500.0, 50.0, 0.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, far=2000.0, zoom_enabled=True, zoom_speed=3.0)

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
        'model': Mesh(osp.join('models/Mars.obj'), texture_enabled=True),
        'to_sun': 400.0,
    },
    'earth': {
        'model': Mesh(osp.join('models/earth.obj'), texture_enabled=False),
        'to_sun': 200.0,
    },
    'moon': {
        'model': Mesh(osp.join('models/moon.obj'), colors=(PaletteDefault.WhiteB.as_color(),), material_enabled=False),
        'to_earth': 50.0,
    },
    'venus': {
        'model': Mesh(osp.join('models/Venus.obj'), texture_enabled=True),
        'to_sun': 120.0,
    },
    'mercury': {
        'model': Mesh(osp.join('models/Mercury.obj'), texture_enabled=True),
        'to_sun': 80.0,
    },
}

sun = Mesh(osp.join('models/sun.obj'), texture_enabled=False)
sun.init_transform = Mat4.from_scale(Vec3(0.8))

mercury = planets['mercury']['model']
mercury.init_transform = Mat4.from_scale(Vec3(0.8))

venus = planets['venus']['model']
venus.init_transform = Mat4.from_scale(Vec3(2.5))

earth = planets['earth']['model']
earth.init_transform = Mat4.from_scale(Vec3(0.5))

moon = planets['moon']['model']
moon.init_transform = Mat4.from_scale(Vec3(0.5))

mars = planets['mars']['model']
mars.init_transform = Mat4.from_scale(Vec3(2.5))


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
        .then(Mat4.from_translation(Vec3(planets['moon']['to_earth'], 0, 0)))\
        .then(Mat4.from_rotation_y(time)).then(earth.transform)
    draw(moon)

    venus.reset_transform()
    venus.apply_transform(Mat4.from_rotation_y(time * 1.5))\
        .then(Mat4.from_translation(Vec3(planets['venus']['to_sun'], 0, 0)))\
        .then(Mat4.from_rotation_y(time * 1.5))
    draw(venus)

    mercury.reset_transform()
    mercury.apply_transform(Mat4.from_rotation_y(time * 4.0))\
        .then(Mat4.from_translation(Vec3(planets['mercury']['to_sun'], 0, 0)))\
        .then(Mat4.from_rotation_y(time * 4.0))
    draw(mercury)

    mars.reset_transform()
    mars.apply_transform(Mat4.from_rotation_y(time * 0.5))\
        .then(Mat4.from_translation(Vec3(planets['mars']['to_sun'], 0, 0)))\
        .then(Mat4.from_rotation_y(time * 0.5))
    draw(mars)


@window.event
def on_update(dt):
    global time
    time += dt


app.init(window)
app.run()
