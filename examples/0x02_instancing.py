import bk7084 as bk
from bk7084.math import *

win = bk.Window()
win.set_title("BK7084 - 0x00: Window")
win.set_size(800, 600)
win.set_resizable(True)

app = bk.App()
camera = app.create_camera(pos=Vec3(0, 0, 90), look_at=Vec3(0, 0, 0), fov_v=60.0, near=0.1, far=200.0)
camera.set_as_main_camera()

cube_mesh = bk.Mesh.load_from(bk.res_path("../data/blender_cube/cube.obj"))
sphere_mesh = bk.Mesh.load_from(bk.res_path("../data/blender_sphere/sphere.obj"))

app.add_directional_light(Vec3(0.0, 0.0, -1.0), bk.Color.WHITE)

objs = []

for i in range(0, 16):
    for j in range(0, 16):
        if j % 2 == 0:
            for k in range(0, 16):
                cube = app.add_mesh(cube_mesh)
                cube.set_visible(True)
                objs.append(cube)
        else:
            for k in range(0, 16):
                sphere = app.add_mesh(sphere_mesh)
                sphere.set_visible(True)
                objs.append(sphere)


@app.event
def on_update(input, dt, t):
    angle = t
    for i, c in enumerate(objs):
        axis = i % 3
        rot = Mat4.identity()
        if axis == 0:
            rot = Mat4.from_rotation_x(angle)
        elif axis == 1:
            rot = Mat4.from_rotation_y(angle)
        elif axis == 2:
            rot = Mat4.from_rotation_z(angle)

        z = ((i // 256) - 8) * 4.0
        x = (((i % 256) // 16) - 8) * 4.0
        y = (((i % 256) % 16) - 8) * 4.0

        c.set_transform(Mat4.from_translation(Vec3(x, y, z)) * rot)


app.run(win)
