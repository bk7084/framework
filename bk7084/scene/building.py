import abc
import uuid
from collections import namedtuple

from . import Mesh
from .entity import Entity
from ..math import Mat4


class Component(metaclass=abc.ABCMeta):
    def __init__(self):
        self._id = uuid.uuid1()
        self._transform = Mat4.identity()
        self._is_drawable = True

    @property
    @abc.abstractmethod
    def mesh(self) -> Mesh:
        raise NotImplementedError

    @property
    def transform(self):
        return self._transform

    @transform.setter
    def transform(self, value: Mat4):
        self._transform = value

    @property
    def id(self):
        return self._id

    def draw(self, matrices=None):
        matrix = Mat4.identity()
        if matrices is not None:
            for m in matrices:
                matrix = m * matrix
        self.mesh.draw(matrix)


class Building(Entity):
    def __init__(self, name=None):
        super().__init__(name)
        self._components = []  # stores the Component objects
        self._root_components = []  # store the Component objects index
        self._hierarchy = {}  # store the parent index of components: { comp_index: parent_index }
        self._is_drawable = True
        self._transform = Mat4.identity()

    def append(self, comp: Component, parent: Component = None):
        exists = False
        for c in self._components:
            if c.id == comp.id:
                exists = True
                break

        if exists:
            raise ValueError('Component already exists.')

        self._components.append(comp)
        index = len(self._components) - 1

        parent_index = -1

        if parent is not None:
            try:
                parent_index = self._components.index(parent)
            except ValueError:
                raise ValueError('Parent does not exist.')

        self._hierarchy[index] = parent_index

        if parent_index == -1:
            self._root_components.append(index)

    @property
    def components(self):
        return self._components

    @property
    def transform(self):
        return self._transform

    @transform.setter
    def transform(self, value: Mat4):
        self._transform = value

    def _parent_list(self, comp, parent_list):
        if comp in self._root_components:
            return parent_list
        else:
            if comp in self._hierarchy:
                parent_list.append(self._hierarchy[comp])
                return self._parent_list(self._hierarchy[comp], parent_list)
            else:
                return []

    def draw(self, shader=None):
        for idx, comp in enumerate(self._components):
            parents = self._parent_list(idx, [idx])
            matrices = [self._components[p].transform for p in parents] + [self.transform]
            comp.draw(matrices)
