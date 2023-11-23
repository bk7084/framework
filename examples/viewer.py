import os.path
import sys

import bk7084 as bk
from bk7084.math import *

if __name__ == "__main__":
    filepath = sys.argv[1]

    if os.path.exists(filepath):
        win = bk.Window()
        win.set_title("BK7084 - 0x00: Window")
        win.set_size(800, 600)
        win.set_resizable(True)

        app = bk.App()
        camera = app.create_camera(pos=Vec3(5, 5, 5), look_at=Vec3(0, 0, 0), fov_v=60.0, near=1, far=10000.0)
        camera.set_as_main_camera()

        print(bk.res_path(filepath))

        mesh = bk.Mesh.load_from(filepath)
        model = app.add_mesh(mesh)
        model.set_visible(True)


        @app.event
        def on_update(input, dt, t):
            pass


        app.run(win)
