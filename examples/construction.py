from pprint import pprint

from bk7084 import Window, app, Camera
from bk7084.app.window.input import KeyCode
from bk7084.math import Vec3, Mat4
from bk7084.misc import PaletteDefault as Palette
from bk7084.graphics import draw, PointLight

# Setup window and add camera
from bk7084.scene import Mesh, Building, Component
from bk7084.scene.mesh import SubMesh

window = Window("BK7084: Construction", width=1024, height=1024)
window.create_camera(Vec3(4, 2.0, 4.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0)


class Wall(Component):
    def __init__(self, w, h, texture1, texture2):
        super().__init__()
        self._mesh = Mesh(
            vertices=[[-w / 2.0, -h / 2.0, 0.0], [w / 2.0, -h / 2.0, 0.0], [w / 2.0, h / 2.0, 0.0], [-w / 2.0, h / 2.0, 0.0],
                      # [-w, -h, 0.0], [w, -h, 0.0], [w, h, 0.0], [-w, h, 0.0]
                      ],
            colors=[Palette.BlueA.as_color()],
            normals=[[0.0, 0.0, 1.0]],
            uvs=[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.25, 0.25], [0.75, 0.25], [0.75, 0.75], [0.25, 0.75]],
            triangles=[
                # [(0, 1, 5, 4), (4, 5, 1, 0), (0, 0, 0, 0)],
                # [(1, 2, 6, 5), (5, 6, 2, 1), (0, 0, 0, 0)],
                # [(2, 3, 7, 6), (6, 7, 3, 2), (0, 0, 0, 0)],
                # [(3, 0, 4, 7), (7, 4, 0, 3), (0, 0, 0, 0)],
                [(0, 1, 2, 3), (0, 1, 2, 3), (0, 0, 0, 0)]])

        # self._mesh.update_sub_mesh(0, SubMesh(name='body', triangles=[0, 1, 2, 3]), texture=texture1)
        # self._mesh.append_sub_mesh(SubMesh(name='window', triangles=[4]), texture=texture2)
        self._mesh.texture_enabled = True
        # self._mesh.apply_transformation(Mat4.from_rotation_y(45, True))

    @property
    def mesh(self) -> Mesh:
        return self._mesh


wall = Wall(1.0, 1.0, texture1='textures/checker_small.png', texture2='textures/checker_color.png')
# wall2 = Wall(1.0, 1.0, texture1='textures/checker_huge.png', texture2='textures/checker_large.png')
# wall2.transform = Mat4.from_translation(Vec3(1.0, 0.0, 0.0))
# wall3 = Wall(1.0, 1.0, texture1='textures/checker_small.png', texture2='textures/checker_color.png')
# wall3.transform = Mat4.from_rotation_y(90, True) * Mat4.from_translation(Vec3(1.4, 0.0, 0.0))

building = Building()
building.append(wall)
# building.append(wall2, wall)
# building.append(wall3, wall)


@window.event
def on_draw(dt):
    building.draw()


@window.event
def on_key_press(key, mods):
    pass


@window.event
def on_update(dt):
    wall.transform *= Mat4.from_rotation_y(45.0 * dt, True)
    # wall2.mesh.transformation *= Mat4.from_rotation_z(45.0 * dt, True)


app.init(window)
app.run()
