__all__ = ['App', 'Window']

import inspect
from bk7084.bkfw import Window
from bk7084.bkfw import PyAppState
from bk7084.bkfw import run_main_loop


class App(PyAppState):
    """The main application class."""
    def __new__(cls):
        return super().__new__(cls)

    def event(self, *args):
        """Decorator for attaching event handlers to the window.

        Usage:

        @app.event
        def on_update(dt, input):
            pass

        @app.event('on_resize')
        def random_name(width, height):
            pass
        """
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

    def run(self, builder: Window):
        """Starts the main loop."""
        run_main_loop(self, builder)
