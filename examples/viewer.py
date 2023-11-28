import argparse
import os.path
import sys

import bk7084 as bk
from bk7084.math import *


def read_from_file(filepath, scale=1.0):
    if not os.path.exists(filepath):
        print("File not found: %s" % filepath)
        return
    transform = Mat4.from_scale(Vec3(scale, scale, scale))
    if os.path.isdir(filepath):
        for root, dirs, files in os.walk(filepath):
            for filename in files:
                if filename.endswith(".obj"):
                    model = app.add_mesh(bk.Mesh.load_from(os.path.join(root, filename)))
                    model.set_visible(True)
                    model.set_transform(transform)
    else:
        model = app.add_mesh(bk.Mesh.load_from(filepath))
        model.set_visible(True)
        model.set_transform(transform)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="*", help="Files to open")
    parser.add_argument("--far", type=int, default=200, help="Far plane")
    parser.add_argument("--cam-pos", type=float, nargs=3, default=[5, 5, 5], help="Camera position")
    parser.add_argument("--scale", type=float, default=1.0, help="Scale")

    args = parser.parse_args()

    win = bk.Window()
    win.set_title("BK7084 - Viewer")
    win.set_size(800, 600)
    win.set_resizable(True)

    app = bk.App()
    camera = app.create_camera(pos=Vec3(args.cam_pos), look_at=Vec3(0, 0, 0), fov_v=60.0, near=0.1, far=args.far)
    camera.set_as_main_camera()

    app.add_directional_light(Vec3(-1.0, -1.0, -1.0), bk.Color.WHITE)
    app.add_directional_light(Vec3(1.0, 1.0, 1.0), bk.Color.WHITE)
    # app.add_point_light(Vec3(2.5, 0, 2.5), bk.Color(1.0, 1.0, 1.0), show_light=True)

    for filepath in args.files:
        read_from_file(filepath, args.scale)

    @app.event
    def on_update(input, dt, t):
        pass


    app.run(win)
