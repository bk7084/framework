from .entity import Entity


class Building(Entity):
    def __init__(self, name=None):
        super().__init__(name)
        self._meshes = []

    def add_components(self, *components):
        pass
