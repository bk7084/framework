import os
import __main__


class PathResolver:
    def __init__(self, search_paths=None):
        _search_paths = [
            os.path.abspath(os.path.dirname(__file__)),
            os.path.abspath(os.path.curdir),
        ]
        if hasattr(__main__, '__file__'):
            _search_paths.insert(0, os.path.abspath(os.path.dirname(__main__.__file__)))

        if search_paths is not None:
            for path in search_paths:
                abs_path = os.path.abspath(path)
                if os.path.exists(abs_path):
                    _search_paths.append(abs_path)

        self._search_paths = list(set(_search_paths))

    def resolve(self, filepath: str):
        valid_path = None
        if os.path.isabs(filepath) and os.path.exists(filepath):
            valid_path = filepath
        else:
            for base_path in self._search_paths:
                path = os.path.join(base_path, filepath)
                if os.path.exists(path):
                    valid_path = path
                    break
        if valid_path is None:
            raise ValueError(f'Cannot resolve path: {filepath}')
        return valid_path


default_resolver = PathResolver()
