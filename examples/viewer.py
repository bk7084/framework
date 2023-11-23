import os.path
import sys

import bk7084 as bk
from bk7084.math import *


def read_from_file(filepath):
    if not os.path.exists(filepath):
        print("File not found: %s" % filepath)
        return
    if os.path.isdir(filepath):
        for root, dirs, files in os.walk(filepath):
            for filename in files:
                if filename.endswith(".obj"):
                    model = app.add_mesh(bk.Mesh.load_from(os.path.join(root, filename)))
                    model.set_visible(True)
                    # model.set_transform(Mat4.from_scale(Vec3(0.1, 0.1, 0.1)))
    else:
        model = app.add_mesh(bk.Mesh.load_from(filepath))
        model.set_visible(True)
        # model.set_transform(Mat4.from_scale(Vec3(0.1, 0.1, 0.1)))


if __name__ == "__main__":
    win = bk.Window()
    win.set_title("BK7084 - Viewer")
    win.set_size(800, 600)
    win.set_resizable(True)

    app = bk.App()
    camera = app.create_camera(pos=Vec3(50, 5, 50), look_at=Vec3(0, 0, 0), fov_v=60.0, near=0.1, far=500.0)
    camera.set_as_main_camera()

    for filepath in sys.argv[1:]:
        read_from_file(filepath)

    @app.event
    def on_update(input, dt, t):
        pass


    app.run(win)
