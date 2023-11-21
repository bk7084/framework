import bk7084 as bk
from bk7084.math import Vec3

win = bk.Window()
win.set_title("BK7084 - 0x00: Window")
win.set_size(800, 600)
win.set_resizable(True)

app = bk.App()
print(type(Vec3(1, 2, 3)))
camera = app.create_camera(pos=Vec3(5, 5, 5), look_at=Vec3(0, 0, 0), fov_v=60.0)

counter = 0


@app.event
def on_update(input, dt, t):
    global counter
    counter += dt
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
