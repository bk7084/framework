import bk7084 as bk
from bk7084.math import *

win = bk.Window()
win.set_title("BK7084 - 0x01: Camera")
win.set_size(800, 600)
win.set_resizable(True)

app = bk.App()
camera = app.create_camera(pos=Vec3(5, 5, 5), look_at=Vec3(0, 0, 0), fov_v=60.0)

tri_mesh = bk.Mesh.create_triangle(Vec3(-2, -2, 1), Vec3(2, -2, 1), Vec3(0, 2, 1))
tri = app.add_mesh(tri_mesh)
tri.set_visible(True)

# cube_mesh = bk.Mesh.create_cube()
# cube = app.add_mesh(cube_mesh)
#
# cube.set_transform()

# material = bk.Material()
# material_handle = app.add_material(material)
#
# app.create_renderable(cube_handle, material_handle)


@app.event
def on_update(input, dt, t):
    if input.is_key_pressed(bk.KeyCode.Space):
        print("Space key is pressed")
    if input.is_mouse_pressed(bk.MouseButton.Left):
        print("Left mouse button is pressed")
    if input.is_shift_pressed():
        print("Shift key is pressed")


@app.event
def on_resize(width, height):
    print("Window resized to %dx%d" % (width, height))


app.run(win)
