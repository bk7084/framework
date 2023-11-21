from .input import *
from .app import *
from .camera import *
from .mesh import *
from . import math
from . import geometry
from bk7084.bkfw import ConcatOrder, Alignment


def res_path(path: str) -> str:
    """Returns the absolute path of a resource file.

    Args:
        path (str): Relative path of the resource file.

    Returns:
        str: Absolute path of the resource file.
    """
    import os.path as osp
    import traceback
    stack = traceback.extract_stack()
    dirname = osp.dirname(osp.abspath(stack[-2].filename))
    return osp.abspath(osp.join(dirname, path))
