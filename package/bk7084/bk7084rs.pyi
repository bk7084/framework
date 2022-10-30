from typing import Callable, List

# from .bk7084rs import *

def run_main_loop(state: AppState):
    """Run the main loop of the application.
    """
    ...


class InputState:
    pass

class KeyCode:
    pass

class AppState:
    def __init__(self, title: str, width: int, height: int, resizable: bool = True, fullscreen: bool = False):
        """Create a new application state.
        Note: please register 'on_key_press' and 'on_key_release' callbacks in order to receive keyboard events.
        """
        ...

    def register_event_type(self, event_type: str):
        """
        Register an event type to be used with the event system.
        """
        ...

    def register_event_types(self, event_types: List[str]):
        """
        Register multiple event types to be used with the event system.
        """
        ...

    def attach_event_handler(self, event_type: str, handler: Callable):
        """
        Attach a handler to a given event type.
        """
        ...

    def detach_event_handler(self, event_type: str, handler: Callable):
        """
        Detach a handler from a given event type.
        """
        ...

    def dispatch_event(self, event_type: str, *args, **kwargs):
        """
        Dispatch an event of the given type.
        """
        ...

    def delta_time(self) -> float:
        """
        Get the time since the last frame in seconds.
        """
        ...

    def resize(self, width: int, height: int):
        """
        Resize the window.
        """
        ...

    def toggle_fullscreen(self):
        """
        Toggle fullscreen mode.
        """
        ...

    def input(self) -> InputState:
        """
        Get the current input state.
        """
        ...