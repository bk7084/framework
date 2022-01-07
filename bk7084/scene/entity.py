import abc
import uuid


class Entity(metaclass=abc.ABCMeta):
    def __init__(self, name=None):
        self._id = uuid.uuid1()
        self._name = str(name) if name else 'Unnamed{}'.format(self.__class__.__name__)
        self._is_visible = True
        self._is_drawable = False
        self._cast_shadow = False

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
