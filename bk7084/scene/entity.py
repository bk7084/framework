import abc
import uuid


class Entity(metaclass=abc.ABCMeta):
    def __init__(self, name=None, cast_shadow=True):
        self._uuid = uuid.uuid1()
        self._name = str(name) if name else 'Unnamed{}'.format(self.__class__.__name__)
        self._is_drawable = True
        self._is_visible = True
        self._cast_shadow = cast_shadow

    @property
    def visible(self):
        return self._is_visible

    @visible.setter
    def visible(self, value):
        self._is_visible = value

    @property
    def drawable(self):
        return self._is_drawable

    @drawable.setter
    def drawable(self, value):
        self._is_drawable = value

    @property
    def cast_shadow(self):
        return self._cast_shadow

    @cast_shadow.setter
    def cast_shadow(self, value):
        self._cast_shadow = value

    @property
    def name(self):
        return self._name

    @name.setter
    def name(self, value):
        self._name = value

    @property
    def uuid(self):
        return self._uuid

    @property
    @abc.abstractmethod
    def meshes(self):
        return NotImplementedError

