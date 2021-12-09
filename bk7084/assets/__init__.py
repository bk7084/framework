import os
import __main__

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
        if hasattr(__main__, '__file__'):
            self._search_paths.insert(0, os.path.abspath(os.path.dirname(__main__.__file__)))

        if search_paths is not None:
            for path in search_paths:
                if os.path.exists(path):
                    self._search_paths.append(path)

    def resolve(self, filepath: str):
        valid_path = None
        for base_path in self._search_paths:
            path = os.path.abspath(os.path.join(base_path, filepath))
            if os.path.exists(path):
                valid_path = path
                break
        if valid_path is None:
            raise ValueError(f'Cannot resolve path: {filepath}')
        return valid_path


default_resolver = PathResolver()
