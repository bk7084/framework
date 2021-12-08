from .entity import Entity


class Building(Entity):
    def __init__(self, name=None):
        super().__init__(name)
        self._meshes = []

    def append(self, mesh):
        self._meshes.append(mesh)

    @property
    def meshes(self):
        return self._meshes

    def draw(self, shader=None):
        for m in self._meshes:
            m.draw(shader)
