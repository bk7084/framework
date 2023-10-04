import bk7084 as bk

app = bk.App("Bk7084", 600, 600, True, False)
print('app cursor position', app.cursor_position())

a = 0

@app.event
def on_update(dt):
    print("delta: ", dt)
    global a
    a += 10 * dt
    print("a: ", a)
    print("[py] cursor pos: ", app.cursor_position())
    print("[pyrs] cursor pos: ", app.input.cursor_position())
    #if app.is_key_pressed(bk.KeyCode.Y):
        #print(a)

@app.event
def on_draw(dt):
    print("draw")


app.init()
app.run()
