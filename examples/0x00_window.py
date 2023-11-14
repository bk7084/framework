import bk7084 as bk

win = bk.Window()
win.set_title("BK7084 - 0x00: Window")
win.set_size(800, 600)
win.set_resizable(True)

app = bk.App()
counter = 0

mesh = bk.Mesh(bk.Topology.TriangleList)


@app.event
def on_update(dt, input):
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
