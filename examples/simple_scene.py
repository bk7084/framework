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

mesh0 = Mesh(shapes=[tri0])

mesh1 = Mesh("./abc000.obj")
# mesh1.initial_transformation = Mat4.from_translation(Vec3(0, -100, 0))

animate = True


@window.event
def on_draw(dt):
    # Draw a grid of lines
    # for i in range(21):
    #     draw(Line([Vec3(-10, -10 + i, 0), Vec3(10, -10 + i, 0)]))
    #     draw(Line([Vec3(-10 + i, -10, 0), Vec3(-10 + i, 10, 0)]))
    # draw(mesh)
    draw(mesh1)


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

    if key == KeyCode.Up:
        mesh1.apply_transformation(Mat4.from_translation(Vec3(0.0, 1.0, 0.0)))
    if key == KeyCode.Down:
        mesh1.apply_transformation(Mat4.from_translation(Vec3(0.0, -1.0, 0.0)))
    if key == KeyCode.Left:
        mesh1.apply_transformation(Mat4.from_translation(Vec3(-1.0, 0.0, 0.0)))
    if key == KeyCode.Right:
        mesh1.apply_transformation(Mat4.from_translation(Vec3(1.0, 0.0, 0.0)))
    if key == KeyCode.F:
        mesh1.apply_transformation(Mat4.from_translation(Vec3(0.0, 0.0, -1.0)))
    if key == KeyCode.B:
        mesh1.apply_transformation(Mat4.from_translation(Vec3(0.0, 0.0, 1.0)))

    if key == KeyCode.S:
        mesh1.apply_transformation(Mat4.from_scale(Vec3(0.5, 0.5, 0.5)))
    if key == KeyCode.R:
        mesh1.apply_transformation(Mat4.from_axis_angle(Vec3.unit_y(), 45.0, True))
    if key == KeyCode.I:
        mesh1.reset_transformation()


@window.event
def on_update(dt):
    mesh1.apply_transformation(Mat4.from_axis_angle(Vec3.unit_y(), 45.0 * dt, True))


app.init(window)
app.run()
