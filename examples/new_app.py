import bk7084 as bk

app = bk.App("BK7084", 400, 400, True, False)
counter = 0


@app.event
def on_update(dt, input):
    global counter
    counter += dt
    # print(f"on_update_from_python {dt} | counter {counter}")
    # if app.is_key_pressed(bk.KeyCode.Space):
    #     print("Space key is pressed")
    if input.is_key_pressed(bk.KeyCode.Space):
        print("Space key is pressed")
    if input.is_mouse_pressed(bk.MouseButton.Left):
        print("Left mouse button is pressed")
    if input.is_shift_pressed():
        print("Shift key is pressed")


app.run()
