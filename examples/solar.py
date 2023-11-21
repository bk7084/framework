import bk7084 as bk
from bk7084.math import *
from bk7084 import res_path

window = bk.Window()
window.set_title('BK7084 - Solar System')
window.set_size(1024, 1024)
window.set_resizable(True)

app = bk.App()

camera = app.create_camera(Vec3(-500.0, 50.0, 0.0), Vec3(0, 0, 0), 60.0, near=1.0, far=2000.0)

camera.set_as_main_camera()

planets = {
    'neptune': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/neptune.obj'))),
        'to_sun': 450.0,
    },
    'uranus': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/uranus.obj'))),
        'to_sun': 400.0,
    },
    'saturn': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/saturn.obj'))),
        'to_sun': 350.0,
    },
    'jupiter': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/jupiter.obj'))),
        'to_sun': 300.0,
    },
    'mars': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/mars.obj'))),
        'to_sun': 400.0,
    },
    'earth': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/earth.obj'))),
        'to_sun': 200.0,
    },
    'moon': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/moon.obj'))),
        'to_earth': 80.0,
    },
    'venus': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/venus.obj'))),
        'to_sun': 120.0,
    },
    'mercury': {
        'model': app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/mercury.obj'))),
        'to_sun': 80.0,
    },
}

sun = app.add_mesh(bk.Mesh.load_from(res_path('models/solar_system/sun.obj')))
sun.set_visible(True)
sun.set_transform(Mat4.from_scale(Vec3(0.8)))

mercury = planets['mercury']['model']
# mercury.set_visible(True)
mercury.set_transform(Mat4.from_scale(Vec3(0.8)))

venus = planets['venus']['model']
venus.set_visible(True)

earth = planets['earth']['model']
earth.set_visible(True)

moon = planets['moon']['model']
moon.set_visible(True)

mars = planets['mars']['model']
# mars.set_visible(True)
mars.set_transform(Mat4.from_scale(Vec3(2.5)))

start = 0

@app.event
def on_update(input, dt, t):
    global start
    start += dt

    t = t * 0.5

    # neptune.apply_transform(neptune_translation).then(Mat4.from_rotation_y(time * 35.0, True))

    # uranus.reset_transform()
    # uranus.apply_transform(uranus_translation).then(Mat4.from_rotation_y(time * 30.0, True))
    # draw(uranus)

    earth_transform = Mat4.from_rotation_y(t) * \
                      Mat4.from_translation(Vec3(planets['earth']['to_sun'], 0, 0)) * \
                      Mat4.from_rotation_y(t) * Mat4.from_scale(Vec3(0.3, 0.3, 0.3))

    moon_transform = earth_transform * \
                      Mat4.from_rotation_y(t * 4.0) * \
                      Mat4.from_translation(Vec3(planets['moon']['to_earth'], 0, 0)) * \
                      Mat4.from_rotation_y(t)

    venus_transform = Mat4.from_rotation_y(t * 1.5) * \
                      Mat4.from_translation(Vec3(planets['venus']['to_sun'], 0, 0)) * \
                      Mat4.from_rotation_y(t * 1.5) * Mat4.from_scale(Vec3(2.5))
    #
    # mercury.reset_transform()
    # mercury.apply_transform(Mat4.from_rotation_y(time * 4.0))\
    #     .then(Mat4.from_translation(Vec3(planets['mercury']['to_sun'], 0, 0)))\
    #     .then(Mat4.from_rotation_y(time * 4.0))
    #
    # mars.reset_transform()
    # mars.apply_transform(Mat4.from_rotation_y(time * 0.5))\
    #     .then(Mat4.from_translation(Vec3(planets['mars']['to_sun'], 0, 0)))\
    #     .then(Mat4.from_rotation_y(time * 0.5))

    earth.set_transform(earth_transform)
    moon.set_transform(moon_transform)
    venus.set_transform(venus_transform)


app.run(window)
