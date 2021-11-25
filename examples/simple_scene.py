from bk7084 import Window, app
from bk7084.app.window.input import KeyCode
from bk7084.geometry import Triangle, Ray, Line, Box
from bk7084.math import Vec3, Mat3, Mat4
from bk7084.misc import PaletteSvg, PaletteDefault
from bk7084.graphics import draw

# Setup window and add camera
from bk7084.scene import Mesh

window = Window("BK7084: 01-Intersection")
window.create_camera(Vec3(0, 0.0, 10.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)

# Create a triangle and ray and set the animation flag
tri0 = Triangle([-2, -2, 1.5], [2, -2, 1.5], [0, 2, 1.5],
               colors=(PaletteDefault.RedA.as_color(), PaletteDefault.GreenA.as_color(), PaletteDefault.BrownB.as_color()))
tri1 = Triangle([-2, -2, -1.5], [2, -2, -1.5], [0, 2, -1.5],
               colors=(PaletteDefault.RedA.as_color(), PaletteDefault.GreenA.as_color(), PaletteDefault.BrownB.as_color()))
ray = Ray([1, 1, 1], [1, 1, 2])
box = Box(2.0, 2.0, 2.0)

# mesh = Mesh(shapes=[tri0, tri1, box, ray])

mesh = Mesh("./tree.obj")

animate = True


@window.event
def on_draw(dt):
    # Draw a grid of lines
    # for i in range(21):
    #     draw(Line([Vec3(-10, -10 + i, 0), Vec3(10, -10 + i, 0)]))
    #     draw(Line([Vec3(-10 + i, -10, 0), Vec3(-10 + i, 10, 0)]))
    # draw(mesh)
    draw(mesh)


# mat_rot = Mat4(
#     [
#         [0, 0, 0, 0],
#         [0, 0, 0, 0],
#         [0, 0, 0, 0],
#         [0, 0, 0, 0]
#     ]
# )


@window.event
def on_key_press(key, mods):
    global animate
    if key == KeyCode.A:
        animate = not animate
    if key == KeyCode.T:
        mesh.apply_transformation(Mat4.from_translation(Vec3(2.0, 0.0, 0.0)))
    if key == KeyCode.S:
        mesh.apply_transformation(Mat4.from_scale(Vec3(0.5, 0.5, 0.5)))
    if key == KeyCode.R:
        mesh.apply_transformation(Mat4.from_axis_angle(Vec3.unit_y(), 45.0, True))
    if key == KeyCode.I:
        mesh.reset_transformation()


@window.event
def on_update(dt):
    mesh.apply_transformation(Mat4.from_axis_angle(Vec3.unit_y(), 45.0 * dt, True))


app.init(window)
app.run()
