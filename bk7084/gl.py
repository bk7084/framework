# MacOS fix
try:
    import OpenGL

    try:
        import OpenGL.GL  # this fails in <=2020 versions of Python on OS X 11.x

    except ImportError:
        print('Patching OpenGL for macOS')
        from ctypes import util

        orig_util_find_library = util.find_library


        def new_util_find_library(name):
            res = orig_util_find_library(name)
            if res:
                return res
            return '/System/Library/Frameworks/' + name + '.framework/' + name


        util.find_library = new_util_find_library
        import OpenGL.GL

except ImportError:
    print('OpenGL import error')
    pass


from OpenGL.GL import *

from OpenGL.GL.shaders import compileShader


def current_context_version():
    return tuple(map(int, glGetString(GL_VERSION).decode('utf-8')[:3].split('.')))
