from framework.bk7084 import Window, app
from framework.bk7084.math import Vec3, Mat4
from framework.bk7084.misc import PaletteDefault as Palette

# Setup window and add camera
from framework.bk7084.scene import Mesh, Building, Component, Scene
from framework.bk7084.scene.mesh import SubMesh

window = Window("BK7084: Construction", width=1024, height=1024)
# window.create_camera(Vec3(4, 2.0, 4.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, zoom_enabled=True)


class Wall(Component):
    def __init__(self, w, h):
        super().__init__()
        self._mesh = Mesh(
            name='wall',
            vertices=[[-w / 2.0, -h / 2.0, 0.0], [w / 2.0, -h / 2.0, 0.0], [w / 2.0, h / 2.0, 0.0], [-w / 2.0, h / 2.0, 0.0],
                      [-w, -h, 0.0], [w, -h, 0.0], [w, h, 0.0], [-w, h, 0.0]],
            colors=[Palette.BlueA.as_color()],
            normals=[[0.0, 0.0, 1.0]],
            uvs=[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.25, 0.25], [0.75, 0.25], [0.75, 0.75], [0.25, 0.75]],
            faces=[
                [(0, 1, 5, 4), (4, 5, 1, 0), (0, 0, 0, 0)],
                [(1, 2, 6, 5), (5, 6, 2, 1), (0, 0, 0, 0)],
                [(2, 3, 7, 6), (6, 7, 3, 2), (0, 0, 0, 0)],
                [(3, 0, 4, 7), (7, 4, 0, 3), (0, 0, 0, 0)],
                [(0, 1, 2, 3), (0, 1, 2, 3), (0, 0, 0, 0)]],
            vertex_shader='shaders/example.vert',
            pixel_shader='shaders/example.frag')

        self._mesh.update_sub_mesh(0, SubMesh(name='body', faces=[0, 1, 2, 3], enable_normal_map=True),
                                   texture='models/brick.jpg', normal_map='models/brick_normal_map.png')
        self._mesh.append_sub_mesh(SubMesh(name='window', faces=[4]), texture='models/window.jpg')
                                   # pixel_shader='shaders/example2.frag')
        self._mesh.texture_enabled = True
        # self._mesh.apply_transformation(Mat4.from_rotation_y(45, True))

    @property
    def mesh(self) -> Mesh:
        return self._mesh


wall = Wall(1.0, 1.0)
print(wall.mesh.sub_meshes)
print(wall.mesh.rendering_info)
# wall2 = Wall(1.0, 1.0, texture1='textures/checker_huge.png', texture2='textures/checker_large.png')
# wall2.transform = Mat4.from_translation(Vec3(1.0, 0.0, 0.0))
# wall3 = Wall(1.0, 1.0, texture1='textures/checker_small.png', texture2='textures/checker_color.png')
# wall3.transform = Mat4.from_rotation_y(90, True) * Mat4.from_translation(Vec3(1.4, 0.0, 0.0))

building = Building()
building.append(wall)
# building.append(wall2, wall)
# building.append(wall3, wall)

scene = Scene(window, [building])
scene.create_camera(Vec3(2, 1.0, 2.0), Vec3(0, 0, 0), Vec3.unit_y(), 60.0, zoom_enabled=True, safe_rotations=False)


@window.event
def on_draw(dt):
    # building.draw()
    # scene.draw()
    scene.draw_v2()


@window.event
def on_key_press(key, mods):
    pass


@window.event
def on_update(dt):
    pass
    # wall.transform *= Mat4.from_rotation_y(45.0 * dt, True)
    # wall2.mesh.transformation *= Mat4.from_rotation_z(45.0 * dt, True)


app.init(window)
app.run()
