__all__ = ['App']

import inspect
from bk7084.bk7084rs import AppState
from bk7084.bk7084rs import run_main_loop


class App(AppState):
    """The main application class."""
    def __new__(cls, name, width, height, resizable, fullscreen):
        return super().__new__(cls, name, width, height, resizable, fullscreen)

    def event(self, *args):
        """Decorator for attaching event handlers to the window."""
        if len(args) == 0:  # @window.event()
            def decorator(fn):
                self.attach_event_handler(fn.__name__, fn)
                return fn
            return decorator
        elif inspect.isroutine(args[0]):  # @window.event
            fn = args[0]
            self.attach_event_handler(fn.__name__, fn)
            return args[0]
        elif type(args[0]) in (str,):  # @window.event('on_resize')
            def decorator(fn):
                self.attach_event_handler(args[0], fn)
            return decorator

    def run(self):
        """Starts the main loop."""
        run_main_loop(self)