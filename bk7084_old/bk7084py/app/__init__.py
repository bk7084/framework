import inspect
import logging
import os

from .event import EventDispatcher
from .window import Window
from bk7084rs import KeyCode, MouseButton

__all__ = [
    'current_time',
    'current_window',
    'gl_context_version',
    'init',
    'run'
]


class App:
    def __init__(self, title: str = "BK7084", width: int = 600, height: int = 600, resizable: bool = True,
                 fullscreen: bool = False):
        self._state = AppState(title, width, height, resizable, fullscreen)
        self._state.register_event_types([
            "on_cursor_enter",
            "on_cursor_leave",
            "on_cursor_move",
            "on_draw",
            "on_exit",
            "on_resize",
            "on_mouse_motion",
            "on_mouse_drag",
            "on_mouse_press",
            "on_mouse_release",
            "on_mouse_scroll",
            "on_key_press",
            "on_key_release",
            "on_update",
            "on_init",
        ])

    def init(self):
        # self.dispatch('on_init')
        pass

    def run(self):
        run_main_loop(self._state)

    def attach_event_handler(self, name, listener):
        self._state.attach_event_handler(name, listener)

    def on_resize(self, width, height):
        self._state.resize(width, height)
        self._state.dispatch_event('on_resize', width, height)

    @property
    def input(self):
        return self._state.input_state()

    def is_key_pressed(self, key: KeyCode) -> bool:
        return self._state.is_key_pressed(key)

    def is_key_released(self, key: KeyCode) -> bool:
        return self._state.is_key_released(key)

    def is_mouse_button_pressed(self, button: MouseButton) -> bool:
        return self._state.is_mouse_button_pressed(button)

    def is_mouse_button_released(self, button: MouseButton) -> bool:
        return self._state.is_mouse_button_released(button)

    def cursor_position(self) -> (float, float):
        return self._state.cursor_position()

    def cursor_delta(self) -> (float, float):
        return self._state.cursor_delta()

    def scroll_delta(self) -> float:
        return self._state.scroll_delta()

    def event(self, *args):
        """Decorator to register an event handler.

        Args:
            *args: Event name(s) to register.

        Returns:
            Callable: Decorator function.
        """
        if len(args) == 0:  # @app.event()
            def decorator(func):
                self._state.attach_event_handler(func.__name__, func)

            return decorator

        elif inspect.isroutine(args[0]):  # @app.event
            fn = args[0]
            fn_name = fn.__name__
            self._state.attach_event_handler(fn_name, fn)
            return args[0]

        elif type(args[0]) in (str,):  # @app.event('on_resize')
            def decorator(func):
                self._state.attach_event_handler(args[0], func)

            return decorator

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


def init(win: Window):
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
