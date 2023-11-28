import bk7084 as bk
from bk7084.math import *

win = bk.Window()
win.set_title("BK7084 - 0x01: Camera")
win.set_size(800, 600)
win.set_resizable(True)

app = bk.App()

cameras = [
    app.create_camera(pos=Vec3(5, 5, 5), look_at=Vec3(0, 0, 0), fov_v=60.0),
    app.create_camera(pos=Vec3(0, 0, 5), look_at=Vec3(0, 0, 0), fov_v=60.0),
    app.create_camera(pos=Vec3(0, 5, 5), look_at=Vec3(0, 0, 0), fov_v=60.0)
]

cameras[0].set_as_main_camera()

tri_mesh = bk.Mesh.create_triangle(Vec3(-2, -2, -1.5), Vec3(2, -2, -1.5), Vec3(0, 2, -1.5))
tri = app.add_mesh(tri_mesh)
tri.set_visible(True)

cube_mesh = bk.Mesh.load_from(bk.res_path("../data/blender_cube/cube.obj"))
cube = app.add_mesh(cube_mesh)
cube.set_visible(True)

i = 0
space_pressed = False

@app.event
def on_update(input, dt, t):
    global i
    global space_pressed

    if input.is_key_pressed(bk.KeyCode.Space):
        if space_pressed is False:
            space_pressed = not space_pressed
            i += 1
            cameras[i % 3].set_as_main_camera()

    if input.is_key_released(bk.KeyCode.Space):
        if space_pressed is True:
            space_pressed = not space_pressed


app.run(win)
