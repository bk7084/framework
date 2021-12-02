import logging
import os

from . import window

logging.basicConfig(level=os.environ.get('LOGLEVEL', 'ERROR'))

__window__ = None


def gl_context_version():
    return None if __window__ is None else __window__.current_context_version()


def current_window():
    if __window__ is not None:
        return __window__
    else:
        raise ValueError('Window has not been created.')


def current_time():
    return current_window().elapsed_time


def init(win: window.Window):
    global __window__
    """Initialize the main loop."""
    if not isinstance(win, window.Window):
        raise ValueError('Not a valid window object.')
    __window__ = win
    __window__.activate()
    __window__.dispatch('on_init')
    __window__.dispatch('on_resize', win.width, win.height)


def run():
    """Run the main loop."""
    if __window__:
        __window__.main_loop()
