import os

__all__ = [
    'default_resolver',
    'PathResolver',
]


class PathResolver:
    def __init__(self, search_paths=None):
        self._search_paths = [
            os.path.abspath(os.path.dirname(__file__)),
            os.path.curdir,
        ]

        if search_paths is not None:
            for path in search_paths:
                if os.path.exists(path):
                    self._search_paths.append(path)

    def resolve(self, filepath):
        for base_path in self._search_paths:
            path = os.path.abspath(os.path.join(base_path, filepath))
            if os.path.exists(path):
                return path


default_resolver = PathResolver()
