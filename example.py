import bk7084 as bk

app = bk.App("Bk7084", 600, 600, True, False)

a = 0

@app.event
def on_update(dt):
    global a
    a += 10 * dt
    print(a)

@app.event
def on_draw(dt):



app.init()
app.run()
