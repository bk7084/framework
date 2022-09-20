"""Framework written in Python for BK7084 Computational Simulations"""
from .bk7084 import *

from . import app
from . import camera
from . import geometry
from . import graphics
from .app.window import *
from .app import ui
from . import math

__version__ = (0, 2, 0)


__doc__ = bk7084.__doc__

if hasattr(bk7084, "__all__"):
	__all__ = bk7084.__all__