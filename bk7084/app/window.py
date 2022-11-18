from __future__ import annotations

import atexit
import logging
import os
import platform
import sys
import time

import glfw
import imgui
import numpy as np
from imgui.integrations.glfw import GlfwRenderer

from . import event
from .input import KeyCode, MouseButton, KeyModifier
from .. import misc, gl
from ..camera import Camera
from ..math import Vec2

# temporary work around to have access of OpenGL context before the app is initialised
__current_window__: Window


# TODO: detailed description of event listener parameters

class Window(event.EventDispatcher):
    """A thin wrapper around GLFW window.

    Attributes:
        _width (int): width of the window
        _height (int): height of the window
        _native_window (GLFWwindow): inner GLFWwindow

    Events:
        on_cursor_enter()
        on_cursor_leave()
        on_draw(dt)
        on_resize(width, height)
        on_mouse_motion(x, y, dx, dy)
        on_mouse_drag(x, y, dx, dy, button)
        on_mouse_press(x, y, button)
        on_mouse_release(x, y, button)
        on_mouse_scroll(x, y, x_offset, y_offset)
        on_key_press(key, mods)
        on_key_release(key, mods)
        on_update(dt)
        on_init()
        on_idle()
        on_show()
        on_hide()
        on_close()
        on_gui()
    """

    def __init__(self, title, width=600, height=600, **kwargs):
        """ Initialization of the GLFW window.

        Window creation hints are passed through `**kwargs`.

        Args:
            width (int): width of the window
            height (int): height of the window
            title (str): title of the window
            **kwargs:
                clear_color (array-like):
                    Specifies the color used to clear OpenGL color buffer.

                resizable (bool):
                    Specifies whether the window is resizable by the user.

                decorated (bool):
                    Specifies whether the windowed mode window will have window decorations such as a border,
                    a close widget, etc.

                floating (bool):
                    Specifies whether the windowed mode window will be floating above other regular windows.

                maximised (bool):
                    Specifies whether the windowed mode window will be maximized when created.

                gl_version ((int, int)):
                    OpenGL context version in tuple (major, minor).

                gl_forward_compat:
                    Specifies whether the OpenGL context should be forward-compatible, i.e. one where
                    all functionality deprecated in the requested version of OpenGL is removed. This
                    must only be used if the requested OpenGL version is 3.0 or above.

                gl_profile (str):
                    Specifies which OpenGL profile to create the context for. Possible values are
                    ['core', 'compat', 'any']

                macos_retina (bool):
                    Specifies whether to use full resolution framebuffer on Retina displays.

                macos_gfx_switch (bool):
                    Specifies whether to in Automatic Graphics Switching, i.e. to allow the system
                    to choose the integrated GPU for the OpenGL context and move it between GPUs if
                    necessary or whether to force it to always run on the discrete GPU.
        """
        super(Window, self).__init__()
        self._cursor_pos = Vec2(0., 0.)
        self._mouse_btn = MouseButton.NONE
        self._width = width
        self._height = height
        self._title = title
        self._native_window = None
        self._clear_flags = gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT | gl.GL_STENCIL_BUFFER_BIT
        self._x = 0
        self._y = 0
        self._clear_color = kwargs.get('clear_color', misc.PaletteDefault.Background)
        self._camera = None
        self._frame_rate = 0

        atexit.register(self._delete)

        if not glfw.init():
            raise RuntimeError("Failed to initialize GLFW!")

        glfw.set_error_callback(self._on_glfw_error)

        # by default the window is resizable, decorated and not floating over other windows
        glfw.window_hint(glfw.RESIZABLE, kwargs.get('resizable', True))
        glfw.window_hint(glfw.DECORATED, kwargs.get('decorated', True))
        glfw.window_hint(glfw.FLOATING, kwargs.get('floating', False))

        # by default the OpenGL version is 3.3
        (gl_major, gl_minor) = kwargs.get('gl_version', (3, 3))
        glfw.window_hint(glfw.CONTEXT_VERSION_MAJOR, gl_major)
        glfw.window_hint(glfw.CONTEXT_VERSION_MINOR, gl_minor)

        # GLFW_OPENGL_FORWARD_COMPAT is activated on macos
        if gl_major >= 3 and platform.system() == 'Darwin':  # MacOs
            glfw.window_hint(glfw.OPENGL_FORWARD_COMPAT, True)

        profile = kwargs.get('profile', 'core')
        if profile not in ['core', 'compat', 'any']:
            raise RuntimeError(f'Unknown OpenGL profile: {profile}')
        # Context profiles are only defined for OpenGL version 3.2 and above
        # by default, OpenGL context is created with core profile
        if gl_major >= 3 and profile == 'core':
            glfw.window_hint(glfw.OPENGL_PROFILE, glfw.OPENGL_CORE_PROFILE)
        elif gl_major >= 3 and profile == 'compatibility':
            glfw.window_hint(glfw.OPENGL_PROFILE, glfw.OPENGL_COMPAT_PROFILE)
        else:
            glfw.window_hint(glfw.OPENGL_PROFILE, glfw.OPENGL_ANY_PROFILE)

        # by default use full resolution on Retina display
        glfw.window_hint(glfw.COCOA_RETINA_FRAMEBUFFER, kwargs.get('macos_retina', True))

        # by default graphics switching on macos is disabled
        glfw.window_hint(glfw.COCOA_GRAPHICS_SWITCHING, kwargs.get('macos_gfx_switch', False))

        self._native_window = glfw.create_window(width, height, title, None, None)

        if not self._native_window:
            print("Failed to create GLFW window", file=sys.stderr)
            glfw.terminate()
            sys.exit()

        glfw.make_context_current(self._native_window)

        # MacOs: Check framebuffer size and window size. On retina display, they may be different.
        w, h = glfw.get_framebuffer_size(self._native_window)
        if platform.system() == 'darwin' and (w != width or h != height):
            width, height = width // 2, height // 2
            glfw.set_window_size(self._native_window, width, height)

        self._mouse_btn = MouseButton.NONE
        self._cursor_pos = [0., 0.]

        # install callbacks
        glfw.set_framebuffer_size_callback(self._native_window, self._on_glfw_framebuffer_resize)
        glfw.set_cursor_enter_callback(self._native_window, self._on_glfw_cursor_enter)
        glfw.set_window_close_callback(self._native_window, self._on_glfw_window_close)
        glfw.set_key_callback(self._native_window, self._on_glfw_key)
        glfw.set_mouse_button_callback(self._native_window, self._on_glfw_mouse_button)
        glfw.set_cursor_pos_callback(self._native_window, self._on_glfw_mouse_motion)
        glfw.set_scroll_callback(self._native_window, self._on_glfw_scroll)
        glfw.set_char_callback(self._native_window, self._on_glfw_char)

        self._width, self._height = glfw.get_framebuffer_size(self._native_window)
        self._x, self._y = glfw.get_window_pos(self._native_window)

        self._old_width, self._old_height, self._old_pos = self._width, self._height, glfw.get_window_pos(
            self._native_window)
        self._is_fullscreen = False

        imgui.create_context()
        self._gui = GlfwRenderer(self._native_window, False) if gl_major >= 3 else None

        self._default_shader = None

        if self.current_context_version >= (3, 3):
            from ..assets.manager import default_asset_mgr
            self._default_shader = default_asset_mgr.get_or_create_pipeline('default_pipeline')
            logging.info("Default shader created.")

        self._start_time = time.time()
        self._previous_time = self._start_time
        self._current_time = self._start_time

        gl.glEnable(gl.GL_DEPTH_TEST)

        global __current_window__
        __current_window__ = self

    def shut_down(self):
        glfw.destroy_window(self._native_window)

    @staticmethod
    def _delete():
        logging.info("GLFW clean up.")
        glfw.terminate()

    def _on_glfw_framebuffer_resize(self, _window, width, height):
        self._width = width
        self._height = height
        self.dispatch('on_resize', width, height)

    def _on_glfw_cursor_enter(self, _window, entered):
        if self._gui and not imgui.get_io().want_capture_mouse:
            if entered:
                self.dispatch('on_cursor_enter')
            else:
                self.dispatch('on_cursor_leave')

    def _on_glfw_window_close(self, _window):
        self.close()

    def _on_glfw_key(self, window, key, scancode, action, mods):
        keycode = KeyCode.from_glfw_keycode(key)
        modifiers = KeyModifier.from_glfw_modifiers(mods)

        if self._gui:
            self._gui.keyboard_callback(window, key, scancode, action, mods)

        if self._gui and not imgui.get_io().want_capture_keyboard:
            if action in [glfw.PRESS, glfw.REPEAT]:
                self.dispatch('on_key_press', keycode, modifiers)
            else:
                self.dispatch('on_key_release', keycode, modifiers)

    def _on_glfw_char(self, window, char):
        if self._gui:
            self._gui.char_callback(window, char)

    def _on_glfw_mouse_button(self, window, button, action, mods):
        x, y = glfw.get_cursor_pos(window)
        button = MouseButton.from_glfw_mouse_btn_code(button)
        if action == glfw.RELEASE:
            self._mouse_btn = MouseButton.NONE
            self._cursor_pos = [x, y]
            if self._gui and not imgui.get_io().want_capture_mouse:
                self.dispatch('on_mouse_release', x, y, button)
        elif action == glfw.PRESS:
            self._mouse_btn = button
            self._cursor_pos = [x, y]
            if self._gui and not imgui.get_io().want_capture_mouse:
                self.dispatch('on_mouse_press', x, y, button)

    def _on_glfw_mouse_motion(self, _window, x, y):
        dx = x - self._cursor_pos[0]
        dy = y - self._cursor_pos[1]
        self._cursor_pos = [x, y]
        if self._gui and not imgui.get_io().want_capture_mouse:
            if self._mouse_btn != MouseButton.NONE:
                self.dispatch('on_mouse_drag', x, y, dx, dy, self._mouse_btn)
            else:
                self.dispatch('on_mouse_motion', x, y, dx, dy)

    def _on_glfw_scroll(self, window, x_offset, y_offset):
        x, y = glfw.get_cursor_pos(window)
        if self._gui and not imgui.get_io().want_capture_mouse:
            self.dispatch('on_mouse_scroll', x, y, x_offset, y_offset)

    def _on_glfw_error(self, error, desc):
        print(f'GLFW Error: {desc}', file=sys.stderr)

    def toggle_fullscreen(self):
        monitor = glfw.get_primary_monitor()
        mode = glfw.get_video_mode(monitor)
        if not self._is_fullscreen:
            self._old_width, self._old_height = self.width, self.height
            self._old_pos = glfw.get_window_pos(self._native_window)
            glfw.set_window_size(self._native_window, mode.size.width, mode.size.height)
            glfw.set_window_pos(self._native_window, 0, 0)
        else:
            glfw.set_window_size(self._native_window, self._old_width, self._old_height)
            glfw.set_window_pos(self._native_window, self._old_pos[0], self._old_pos[1])

        self._is_fullscreen = not self._is_fullscreen

    @property
    def title(self):
        return self._title

    @property
    def width(self):
        self._width, self._height = glfw.get_framebuffer_size(self._native_window)
        return self._width

    @width.setter
    def width(self, value):
        glfw.set_window_size(self._native_window, value, self._height)
        self._width, self._height = glfw.get_framebuffer_size(self._native_window)

    @property
    def height(self):
        self._width, self._height = glfw.get_framebuffer_size(self._native_window)
        return self._height

    @height.setter
    def height(self, value):
        glfw.set_window_size(self._native_window, self._width, self._height)
        self._width, self._height = glfw.get_framebuffer_size(self._native_window)

    @property
    def aspect_ratio(self):
        return self._width / self._height

    @property
    def size(self):
        self._width, self._height = glfw.get_framebuffer_size(self._native_window)
        return self._width, self._height

    @size.setter
    def size(self, sz):
        glfw.set_window_size(self._native_window, sz[0], sz[1])
        self._width, self._height = glfw.get_framebuffer_size(self._native_window)

    @property
    def position(self):
        self._x, self._y = glfw.get_window_pos(self._native_window)
        return Vec2(self._x, self._y)

    @position.setter
    def position(self, pos):
        glfw.set_window_pos(self._native_window, pos[0], pos[1])
        self._x, self._y = glfw.get_window_pos(self._native_window)

    @property
    def current_context_version(self):
        return glfw.get_window_attrib(self._native_window, glfw.CONTEXT_VERSION_MAJOR), \
               glfw.get_window_attrib(self._native_window, glfw.CONTEXT_VERSION_MINOR)

    @property
    def default_shader(self):
        return self._default_shader

    @default_shader.setter
    def default_shader(self, shader):
        self._default_shader = shader

    @property
    def delta_time(self):
        self._current_time = time.time()
        elapsed = self._current_time - self._previous_time
        self._previous_time = self._current_time
        return elapsed

    @property
    def elapsed_time(self):
        return time.time() - self._start_time

    def main_loop(self):
        """
        Start the main loop of the window.
        """
        self._previous_time = time.time()
        
        while not glfw.window_should_close(self._native_window):
            # update time
            dt = self.delta_time
            if dt != 0.0:
                self._frame_rate = int(1.0 / dt)

            # process inputs
            glfw.poll_events()

            if self._gui:
                self._gui.process_inputs()
                imgui.new_frame()
                imgui.set_next_window_position(self._width - 150, 30)
                imgui.set_next_window_size(120, 20)
                imgui.begin('Frame Rate', False,
                            imgui.WINDOW_NO_TITLE_BAR | imgui.WINDOW_NO_RESIZE | imgui.WINDOW_NO_MOVE)
                imgui.text(f'framerate: {self._frame_rate}')
                imgui.end()
                if 'on_gui' in self._event_listeners and len(self._event_listeners['on_gui']) > 0:
                    imgui.begin('Controls')
                    self.dispatch('on_gui', reversed_exec_order=True)
                    imgui.end()
                imgui.end_frame()
                imgui.render()

            # update
            self.dispatch('on_update', dt)

            # render
            gl.glClear(self._clear_flags)

            self.dispatch('on_draw', dt)

            self.dispatch('on_idle', dt)

            if self._gui:
                draw_data = imgui.get_draw_data()
                if draw_data:
                    self._gui.render(draw_data)

            glfw.swap_buffers(self._native_window)

        self.dispatch('on_close')

    def clear(self, color):
        color = color if color else self._clear_color
        gl.glClearColor(*color)
        gl.glClear(self._clear_flags)

    def swap_buffers(self):
        glfw.swap_buffers(self._native_window)

    def activate(self):
        glfw.make_context_current(self._native_window)

    def destroy(self):
        glfw.destroy_window(self._native_window)

    def close(self):
        glfw.set_window_should_close(self._native_window, True)

    def show(self):
        glfw.show_window(self._native_window)
        self.dispatch('on_show')

    def hide(self):
        glfw.hide_window(self._native_window)
        self.dispatch('on_hide')

    def on_init(self):
        gl.glClearColor(*self._clear_color)
        gl.glClear(gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT)

    def on_close(self):
        if self._gui:
            self._gui.shutdown()
        glfw.terminate()

    def on_resize(self, width, height):
        """Default resize handler."""
        self.activate()
        gl.glViewport(0, 0, width, height)
        gl.glClear(self._clear_flags)
        self.dispatch('on_draw', self.delta_time)
        self.swap_buffers()

    def on_key_press(self, key, mods):
        if key == KeyCode.Escape:
            self.close()
        elif key == KeyCode.F10:
            import png
            import datetime
            h, w = self.size
            framebuffer = np.zeros((h, w * 3), dtype=np.uint8)
            gl.glReadPixels(0, 0, w, h, gl.GL_RGB, gl.GL_UNSIGNED_BYTE, framebuffer)
            filename = f'{datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")}.png'
            filepath = os.path.join(os.getcwd(), filename)
            png.from_array(framebuffer[::-1], 'RGB').save(filepath)
            print(f'Screenshot saved to {filepath}')
        elif key == KeyCode.F11:
            self.toggle_fullscreen()
            self.dispatch('on_resize', self.width, self.height)

        return True

    def mouse_position(self):
        return glfw.get_cursor_pos(self._native_window)

    def create_camera(self, pos, look_at, up, fov_v=45.0, near=0.1, far=1000., degrees=True, zoom_enabled=False,
                      safe_rotations=True, zoom_speed=0.25):
        self._camera = Camera(pos, look_at, up, self._width / self._height, fov_v, near, far, degrees, zoom_enabled,
                              safe_rotations, zoom_speed)
        self.attach_listeners(self._camera)

    @property
    def camera(self):
        return self._camera


Window.register_event_type('on_cursor_enter')
Window.register_event_type('on_cursor_leave')
Window.register_event_type('on_draw')
Window.register_event_type('on_resize')
Window.register_event_type('on_mouse_motion')
Window.register_event_type('on_mouse_drag')
Window.register_event_type('on_mouse_press')
Window.register_event_type('on_mouse_release')
Window.register_event_type('on_mouse_scroll')
Window.register_event_type('on_key_press')
Window.register_event_type('on_key_release')
Window.register_event_type('on_update')
Window.register_event_type('on_init')
Window.register_event_type('on_idle')
Window.register_event_type('on_show')
Window.register_event_type('on_hide')
Window.register_event_type('on_close')
Window.register_event_type('on_gui')


class WindowEventLogger:
    """Logs window events when triggered.
    """

    def __init__(self, log_file=None):
        """Creates a `WindowEventLogger` object.

        Args:
            log_file (file like object):
                Specifies the output of log messages. If not specified, stdout will be used.
        """
        self._output = log_file if log_file is not None else sys.stdout

    def on_key_press(self, key, mods):
        print(f'on_key_press({key}, mods={mods})', file=self._output)

    def on_key_release(self, key, mods):
        print(f'on_key_release({key}, mods={mods})', file=self._output)

    def on_mouse_motion(self, x, y, dx, dy):
        print(f'on_mouse_motion(x={x}, y={y}, dx={dx}, dy={dy})', file=self._output)

    def on_mouse_drag(self, x, y, dx, dy, btns, mods):
        print(f'on_mouse_drag(x={x}, y={y}, dx={dx}, dy={dy}, btns={btns}, mods={mods}', file=self._output)

    def on_mouse_press(self, x, y, btn, mods):
        print(f'on_mouse_press(x={x}, y={y}, btn={btn}, mods={mods}', file=self._output)

    def on_mouse_release(self, x, y, btn, mods):
        print(f'on_mouse_release(x={x}, y={y}, btn={btn}, mods={mods}', file=self._output)

    def on_mouse_scroll(self, x, y, dx, dy):
        print(f'on_mouse_scroll(x={x}, y={y}, dx={dx}, dy={dy})', file=self._output)

    def on_close(self):
        print(f'on_close()', file=self._output)

    def on_resize(self, w, h):
        print(f'on_resize(w={w}, h={h})', file=self._output)

    def on_draw(self):
        print(f'on_draw()', file=self._output)
