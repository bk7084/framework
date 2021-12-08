from pprint import pprint

from bk7084 import Window, app, Camera
from bk7084.app.window.input import KeyCode
from bk7084.geometry import Triangle, Ray, Line, Box
from bk7084.math import Vec3, Mat3, Mat4
from bk7084.misc import PaletteDefault as Palette
from bk7084.graphics import draw, PointLight

# Setup window and add camera
from bk7084.scene import Mesh, Building, Scene

window = Window("BK7084: Construction", width=1024, height=1024)
window.create_camera(Vec3(4, 2.0, 4.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)

# t = Triangle(Vec3(0, 0, 0), Vec3(1, 1, 1), Vec3(1, 0, 1))

mesh = Mesh(vertices=[[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]],
            colors=[Palette.BrownB.as_color(), Palette.RedB.as_color(), Palette.GreenA.as_color(), Palette.YellowB.as_color()],
            normals=[[0.0, 0.0, 1.0]],
            uvs=[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            triangles=[[(0, 1, 2), (0, 1, 2), (0, 0, 0)],
                       [(2, 3, 0), (2, 3, 0), (0, 0, 0)]],
            texture='textures/spot_texture.png')

print(mesh.sub_meshes)

mesh.texture_enabled = True
mesh.alternate_texture_enabled = True

# mesh = Mesh('./models/cube.obj', texture='textures/spot_texture.png')
#mesh.texture_enabled = True
# mesh.alternate_texture_enabled = True

# camera = Camera(Vec3(2, 1.0, 2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)
#
# light = PointLight()
#
building = Building()
#
# building.add_components()
#
# scene = Scene([building], [camera], [light])


@window.event
def on_draw(dt):
    # scene.draw()
    draw(mesh)


@window.event
def on_key_press(key, mods):
    pass


@window.event
def on_update(dt):
    pass


app.init(window)
app.run()
