import inspect
import logging


class EventDispatcher:
    """Generic event dispatcher that listens and dispatches events.

    For each type of event, there can only be multiple handlers attached to it.

    @dispatcher.event
    def event_name():
        pass

    @dispatcher.event('event_name')
    def random_name():
        pass
    """
    _event_types = []

    def __init__(self):
        self._event_listeners = {}

    @classmethod
    def register_event_type(cls, name):
        """Register a certain type of event within the dispatcher.

        Args:
            name (str): Name of the event to register.

        Returns:
            None
        """
        cls._event_types.append(name)

    def attach_listener(self, event, listener):
        """Attach an event listener for an event type."""
        if event not in self._event_types:
            print(f'Warn: {event} not yet registered!')
        else:
            if event not in self._event_listeners:
                self._event_listeners[event] = []
            self._event_listeners[event].append(listener)

    def attach_listeners(self, obj):
        if not isinstance(obj, (list, tuple)):
            for event in self._event_types:
                if hasattr(obj, event):
                    self.attach_listener(event, getattr(obj, event))

    def detach_listener(self, event, listener):
        """Remove event listener."""
        if listener in self._event_listeners[event]:
            self._event_listeners[event].remove(listener)

    def detach_listeners(self, obj):
        if not isinstance(obj, (list, tuple)):
            for event in self._event_types:
                if hasattr(obj, event):
                    self.detach_listener(event, getattr(obj, event))

    def dispatch(self, event, *args, **kwargs):
        """Dispatch an event to attached handlers."""
        logging_enabled = kwargs.get('logging_enabled', False)
        reversed_exec_order = kwargs.get('reversed_exec_order', False)
        # Search in instance
        if hasattr(self, event):
            try:
                getattr(self, event)(*args)
            except TypeError:
                self._raise_dispatch_exception(event, args, getattr(self, event))

        # Search in listeners
        listeners = self._event_listeners.get(event, None)
        if listeners:
            try:
                if reversed_exec_order:
                    for listener in listeners:
                        if logging_enabled:
                            logging.info(f'dispatch <{event}> to <{listener}>')
                        listener(*args)
                else:
                    for listener in listeners[::-1]:
                        if logging_enabled:
                            logging.info(f'dispatch <{event}> to <{listener}>')
                        listener(*args)
            except TypeError:
                self._raise_dispatch_exception(event, args, getattr(self, event))

    @staticmethod
    def _raise_dispatch_exception(event, args, listener):
        n_args = len(args)
        spec = inspect.getfullargspec(listener)
        n_listener_args = len(spec.args)

        if inspect.ismethod(listener) and listener.__self__:
            n_listener_args -= 1

        # Allow *args varargs to over specify arguments
        if spec.varargs:
            n_listener_args = max(n_listener_args, n_args)

        # Allow default values to over specify arguments
        if n_listener_args > n_args >= n_listener_args - len(spec.defaults) and spec.defaults:
            n_listener_args = n_args

        if n_listener_args != n_args:
            if inspect.isfunction(listener) or inspect.ismethod(listener):
                desc = f'{listener.__name__} at {listener.__code__.co_filename}:{listener.__code__.co_firstlineno}'
            else:
                desc = repr(listener)

            raise TypeError(f'{event} event was dispatched with {n_args} arguments, but '
                            f'listener {desc} has an incompatible function signature')
        else:
            raise

    def event(self, *args):
        """Function decorator for an event listener/handler.
        """
        if len(args) == 0:  # @window.event()
            def decorator(fn):
                fn_name = fn.__name__
                self.attach_listener(fn_name, fn)
                return fn

            return decorator

        elif inspect.isroutine(args[0]):  # @window.event
            fn = args[0]
            fn_name = fn.__name__
            self.attach_listener(fn_name, fn)
            return args[0]

        elif type(args[0]) in (str,):  # @window.event('on_resize')
            fn_name = args[0]

            def decorator(fn):
                self.attach_listener(fn_name, fn)

            return decorator
