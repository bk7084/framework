from bk7084 import app, Window

window = Window('template', width=800, height=800)


@window.event
def on_init():
    pass


@window.event
def on_draw(dt):
    pass


@window.event
def on_resize(width, height):
    pass


@window.event
def on_key_press(key, mods):
    pass


@window.event
def on_cursor_enter():
    pass


@window.event
def on_cursor_leave():
    pass


@window.event
def on_mouse_motion(x, y, dx, dy):
    pass


@window.event
def on_mouse_drag(x, y, dx, dy, button):
    pass


@window.event
def on_mouse_press(x, y, button):
    pass


@window.event
def on_mouse_release(x, y, button):
    pass


@window.event
def on_mouse_scroll(x, y, x_offset, y_offset):
    pass


@window.event
def on_key_press(key, mods):
    pass


@window.event
def on_key_release(key, mods):
    pass


@window.event
def on_update(dt):
    pass


@window.event
def on_init():
    pass


@window.event
def on_idle(dt):
    pass


@window.event
def on_show():
    pass


@window.event
def on_hide():
    pass


@window.event
def on_close():
    pass


app.init(window)
app.run()
