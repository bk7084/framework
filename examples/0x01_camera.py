import bk7084 as bk
from bk7084.math import *

win = bk.Window()
win.set_title("BK7084 - 0x01: Camera")
win.set_size(800, 600)
win.set_resizable(True)

app = bk.App()
projection = bk.Projection.perspective(60.0)
camera = app.create_camera(projection, Vec3(0.0, 0.0, 5.0))

# cube = bk.Mesh.create_cube()
# cube_handle = app.add_mesh(cube)

# material = bk.Material()
# material_handle = app.add_material(material)
#
# app.create_renderable(cube_handle, material_handle)


@app.event
def on_update(dt, input):
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
