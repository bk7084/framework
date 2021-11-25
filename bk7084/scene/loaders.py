import os.path
import pywavefront

from .mesh import Mesh


class WavefrontReader:
    def __init__(self, filepath):
        if not os.path.isfile(filepath):
            raise ValueError(f"File {filepath} does not exist.")
        self._filepath = os.path.abspath(filepath)

    def read(self) -> Mesh:
        mesh_content = pywavefront.Wavefront(self._filepath)



