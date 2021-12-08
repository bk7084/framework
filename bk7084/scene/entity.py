import abc
import uuid


class Entity(metaclass=abc.ABCMeta):
    def __init__(self, name=None):
        self._id = uuid.uuid1()
        self._name = str(name) if name else 'Unnamed{}'.format(self.__class__.__name__)
        self._is_visible = True
