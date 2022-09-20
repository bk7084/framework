import datetime
import enum
import logging
import math
import os
from collections.abc import Sequence

import numpy as np
import png

from .texture import TextureWrapMode
from .. import gl
from .util import GpuObject, BindSemanticObject


"""
An attachment can be a texture or a renderbuffer. Main difference between these two is that as for a texture,
you can access it from shader, but for renderbuffer you can only access its content using glReadPixels.

Renderbuffer Objects are OpenGL objects that contain images. They are created and used specifically with 
Framebuffer Objects. They are optimized for use as render targets, while Textures may not be, and are the
logical choice when you do not need to sample (i.e. in a post-pass shader) from the produced image. If 
you need to resample (such as when reading depth back in a second shader pass), use Textures instead.
Renderbuffer objects natively accommodate multisampling (MSAA).
"""


class AttachmentKind(enum.Enum):
    Color = 0
    Depth = 1
    Stencil = 2
    DepthStencil = 3


class Attachment(GpuObject, BindSemanticObject):
    """An attachment is an image (texture)."""
    def __init__(self, width, height, kind: AttachmentKind, internal_fmt, fmt, dtype, shader_accessible=True, **kwargs):
        """

        Args:
            width:
            height:
            kind:
            internal_fmt (OpenGL base internal formats, sized internal formats or compressed internal formats):
                Specifies the number of color components in the texture. Only needed if shader_accessible is true.
            fmt:
                Specifies the format of the pixel data. Accepted values: GL_RED, GL_RG, GL_RGB, GL_BGR, GL_RGBA,
                GL_BGRA, GL_RED_INTEGER, GL_RG_INTEGER, GL_RGB_INTEGER, GL_BGR_INTEGER, GL_RGBA_INTEGER,
                GL_BGRA_INTEGER, GL_STENCIL_INDEX, GL_DEPTH_COMPONENT, GL_DEPTH_STENCIL.
            dtype:
                Specifies the data type of the pixel data. Accepted values: GL_UNSIGNED_BYTE, GL_BYTE,
                GL_UNSIGNED_SHORT, GL_SHORT, GL_UNSIGNED_INT, GL_INT, GL_HALF_FLOAT, GL_FLOAT, GL_UNSIGNED_BYTE_3_3_2,
                GL_UNSIGNED_BYTE_2_3_3_REV, GL_UNSIGNED_SHORT_5_6_5, GL_UNSIGNED_SHORT_5_6_5_REV,
                GL_UNSIGNED_SHORT_4_4_4_4, GL_UNSIGNED_SHORT_4_4_4_4_REV, GL_UNSIGNED_SHORT_5_5_5_1,
                GL_UNSIGNED_SHORT_1_5_5_5_REV, GL_UNSIGNED_INT_8_8_8_8, GL_UNSIGNED_INT_8_8_8_8_REV,
                 GL_UNSIGNED_INT_10_10_10_2, and GL_UNSIGNED_INT_2_10_10_10_REV.
            shader_accessible:
        """
        target = gl.GL_TEXTURE_2D if shader_accessible else gl.GL_RENDERBUFFER
        super().__init__(target, -1)
        self._kind = kind
        self._target = target
        self._is_shader_accessible = shader_accessible
        self._width = width
        self._height = height
        self._internal_format = internal_fmt
        self._format = fmt
        self._dtype = dtype
        self._wrap_mode = kwargs.get('wrap_mode', TextureWrapMode.ClampBorder)
        self._border_color = kwargs.get('border_color', (1.0, 1.0, 1.0))
        self._is_dirty = True

        self._create()

    def _create(self):
        logging.info(f'Framebuffer attachment creation <{"texture" if self._is_shader_accessible else "renderbuffer"}>')
        if self._is_shader_accessible:
            self._id = gl.glGenTextures(1)
        else:
            self._id = gl.glGenRenderbuffers(1)

    def _delete(self):
        if self.is_valid():
            if self._is_shader_accessible:
                gl.glDeleteTextures(1, [self._id])
            else:
                gl.glDeleteRenderbuffers(1, [self._id])

    def _activate(self):
        self.bind()

    def _deactivate(self):
        self.unbind()

    @property
    def width(self):
        return self._width

    @property
    def height(self):
        return self._height

    @property
    def kind(self) -> AttachmentKind:
        return self._kind

    @property
    def is_shader_accessible(self):
        return self._is_shader_accessible

    @property
    def target(self):
        return self._target

    def resize(self, width, height):
        if width != self._width or height != self._height:
            self._is_dirty = True
            self._width = width
            self._height = height

    def bind(self):
        """Activate the attachment."""
        if self.is_valid():
            if self._is_shader_accessible:
                gl.glBindTexture(self._target, self._id)
            else:
                gl.glBindRenderbuffer(self._target, self._id)

            if self._is_dirty:
                self._resize()
                self._is_dirty = False

    def unbind(self):
        """Deactivate the attachment."""
        if self._is_shader_accessible:
            gl.glBindTexture(self._target, 0)
        else:
            gl.glBindRenderbuffer(self._target, 0)

    def _resize(self):
        if self._is_shader_accessible:
            gl.glTexImage2D(self._target, 0, self._internal_format, self._width, self._height, 0, self._format,
                            self._dtype, None)
            gl.glTexParameteri(self._target, gl.GL_TEXTURE_MIN_FILTER, gl.GL_LINEAR)
            gl.glTexParameteri(self._target, gl.GL_TEXTURE_MAG_FILTER, gl.GL_LINEAR)
            gl.glTexParameteri(self._target, gl.GL_TEXTURE_WRAP_S, self._wrap_mode.value)
            gl.glTexParameteri(self._target, gl.GL_TEXTURE_WRAP_T, self._wrap_mode.value)
            gl.glTexParameterfv(self._target, gl.GL_TEXTURE_BORDER_COLOR, self._border_color)
        else:
            gl.glRenderbufferStorage(self._target, self._format, self._width, self._height)


class ColorAttachment(Attachment):
    def __init__(self, width, height, internal_fmt=gl.GL_RGBA, fmt=gl.GL_RGBA,
                 dtype=gl.GL_UNSIGNED_BYTE, shader_accessible=True):
        super().__init__(width, height, AttachmentKind.Color, internal_fmt, fmt, dtype, shader_accessible)


class DepthAttachment(Attachment):
    def __init__(self, width, height, internal_fmt=gl.GL_DEPTH_COMPONENT, fmt=gl.GL_DEPTH_COMPONENT,
                 dtype=gl.GL_FLOAT, shader_accessible=False):
        super().__init__(width, height, AttachmentKind.Depth, internal_fmt, fmt, dtype, shader_accessible)


class StencilAttachment(Attachment):
    def __init__(self, width, height, internal_fmt=gl.GL_STENCIL_INDEX, fmt=gl.GL_STENCIL_INDEX,
                 dtype=gl.GL_UNSIGNED_INT, shader_accessible=False):
        super().__init__(width, height, AttachmentKind.Stencil, internal_fmt, fmt, dtype, shader_accessible)


class DepthStencilAttachment(Attachment):
    def __init__(self, width, height, internal_fmt=gl.GL_DEPTH24_STENCIL8, fmt=gl.GL_DEPTH24_STENCIL8,
                 dtype=gl.GL_UNSIGNED_INT_24_8, shader_accessible=False):
        super().__init__(width, height, AttachmentKind.Stencil, internal_fmt, fmt, dtype, shader_accessible)


_color_attachment = [
    gl.GL_COLOR_ATTACHMENT0,
    gl.GL_COLOR_ATTACHMENT1,
    gl.GL_COLOR_ATTACHMENT2,
    gl.GL_COLOR_ATTACHMENT3,
    gl.GL_COLOR_ATTACHMENT4,
    gl.GL_COLOR_ATTACHMENT5,
    gl.GL_COLOR_ATTACHMENT6,
    gl.GL_COLOR_ATTACHMENT7,
    gl.GL_COLOR_ATTACHMENT8,
    gl.GL_COLOR_ATTACHMENT9,
    gl.GL_COLOR_ATTACHMENT10,
    gl.GL_COLOR_ATTACHMENT11,
    gl.GL_COLOR_ATTACHMENT12,
    gl.GL_COLOR_ATTACHMENT13,
    gl.GL_COLOR_ATTACHMENT14,
    gl.GL_COLOR_ATTACHMENT15,
    gl.GL_COLOR_ATTACHMENT16,
    gl.GL_COLOR_ATTACHMENT17,
    gl.GL_COLOR_ATTACHMENT18,
    gl.GL_COLOR_ATTACHMENT19,
    gl.GL_COLOR_ATTACHMENT20,
    gl.GL_COLOR_ATTACHMENT21,
    gl.GL_COLOR_ATTACHMENT22,
    gl.GL_COLOR_ATTACHMENT23,
    gl.GL_COLOR_ATTACHMENT24,
    gl.GL_COLOR_ATTACHMENT25,
    gl.GL_COLOR_ATTACHMENT26,
    gl.GL_COLOR_ATTACHMENT27,
    gl.GL_COLOR_ATTACHMENT28,
    gl.GL_COLOR_ATTACHMENT29,
    gl.GL_COLOR_ATTACHMENT30,
    gl.GL_COLOR_ATTACHMENT31,
]


class Framebuffer(GpuObject, BindSemanticObject):
    """
    A Framebuffer object is a collection of attachments (images).
    Attachment points:
      - color attachment
      - depth attachment
      - stencil attachment
      - depth stencil attachment
    """
    def __init__(self, width, height,
                 color: Sequence[ColorAttachment] = (),
                 depth: Sequence[DepthAttachment] = (),
                 stencil: Sequence[StencilAttachment] = (), **kwargs):
        """

        Args:
            width:
            height:
            color:
            depth:
            stencil:
            **kwargs:
                enable_color_attachment (bool) : Defaults to True.
                enable_depth_attachment (bool) : Defaults to True.
                enable_stencil_attachment (bool) : Defaults to False.
                color_shader_accessible (bool) : Defaults to True.
                depth_shader_accessible (bool) : Defaults to True.
                stencil_shader_accessible (bool) : Defaults to True.
        """
        super().__init__(gl.GL_FRAMEBUFFER, -1)
        self._target = gl.GL_FRAMEBUFFER
        self._width = width
        self._height = height

        self._color_enabled = kwargs.get('enabled_color_attachment', True)
        shader_accessible = kwargs.get('color_shader_accessible', True)
        if self._color_enabled:
            self._color = [ColorAttachment(width, height, shader_accessible=shader_accessible)] if len(color) == 0 else list(color)
        else:
            self._color = []

        self._depth_enabled = kwargs.get('enable_depth_attachment', True)
        shader_accessible = kwargs.get('depth_shader_accessible', False)
        if self._depth_enabled:
            self._depth = [DepthAttachment(width, height, shader_accessible=shader_accessible)] if len(depth) == 0 else list(depth)
        else:
            self._depth = []

        self._stencil_enabled = kwargs.get('enable_stencil_attachment', False)
        shader_accessible = kwargs.get('stencil_shader_accessible', False)
        if self._stencil_enabled:
            self._stencil = [StencilAttachment(width, height, shader_accessible=shader_accessible)] if len(stencil) == 0 else list(stencil)
        else:
            self._stencil = []

        self._is_activated = False

        # Creation of framebuffer object
        self._id = gl.glGenFramebuffers(1)

        # Bind the framebuffer to activate it
        gl.glBindFramebuffer(self._target, self._id)

        # Attach buffers
        if len(self._color) > 0:
            for i, color in enumerate(self._color):
                with color:  # bind to use
                    if color.is_shader_accessible:
                        gl.glFramebufferTexture2D(self._target, _color_attachment[i], color.target, color.handle, 0)
                    else:
                        gl.glFramebufferRenderbuffer(self._target, _color_attachment[i], color.target, color.handle)
        if len(self._depth) > 0:
            for i, depth in enumerate(self._depth):
                with depth:  # bind to use
                    if depth.is_shader_accessible:
                        gl.glFramebufferTexture2D(self._target, gl.GL_DEPTH_ATTACHMENT, depth.target, depth.handle, 0)
                    else:
                        gl.glFramebufferRenderbuffer(self._target, gl.GL_DEPTH_ATTACHMENT, depth.target, depth.handle)
        if len(self._stencil) > 0:
            for i, stencil in enumerate(self._stencil):
                with stencil:  # bind to use
                    if stencil.is_shader_accessible:
                        gl.glFramebufferTexture2D(self._target, gl.GL_STENCIL_ATTACHMENT, stencil.target, stencil.handle, 0)
                    else:
                        gl.glFramebufferRenderbuffer(self._target, gl.GL_STENCIL_ATTACHMENT, stencil.target, stencil.handle)

        if gl.glCheckFramebufferStatus(self._target) != gl.GL_FRAMEBUFFER_COMPLETE:
            raise ValueError('Framebuffer is not complete!')

        gl.glBindFramebuffer(self._target, 0)

    def _activate(self):
        self.bind()

    def _deactivate(self):
        self.unbind()

    def bind(self):
        if self.is_valid():
            gl.glBindFramebuffer(self._target, self._id)
            self._is_activated = True

    def unbind(self):
        if self.is_valid():
            gl.glBindFramebuffer(self._target, 0)
            self._is_activated = False

    def _delete(self):
        logging.info("GPU: delete framebuffer.")
        gl.glDeleteFramebuffers(1, [self._id])

    @property
    def color_attachments(self):
        return self._color

    @color_attachments.setter
    def color_attachments(self, buffers):
        pass

    @property
    def depth_attachments(self):
        return self._depth

    @depth_attachments.setter
    def depth_attachments(self, buffers):
        pass

    @property
    def stencil_attachments(self):
        return self._stencil

    @stencil_attachments.setter
    def stencil_attachments(self, buffers):
        pass

    @property
    def width(self):
        return self._width

    @property
    def height(self):
        return self._height

    def clear(self, color=(0.635, 0.824, 1.0, 1.0), color_bit=True, depth_bit=True, stencil_bit=True):
        gl.glClearColor(*color)
        if self._is_activated:
            flags = 0
            if color_bit:
                flags |= gl.GL_COLOR_BUFFER_BIT
            if depth_bit:
                flags |= gl.GL_DEPTH_BUFFER_BIT
            if stencil_bit:
                flags |= gl.GL_STENCIL_BUFFER_BIT

            gl.glClear(flags)

    def enable_depth_test(self):
        if self._is_activated:
            gl.glEnable(gl.GL_DEPTH_TEST)

    def enable_stencil_test(self):
        if self._is_activated:
            gl.glEnable(gl.GL_STENCIL_TEST)

    def disable_color_output(self):
        """
        Tells OpenGL that we are not going to render any color data.
        """
        if self._is_activated:
            gl.glDrawBuffer(gl.GL_NONE)
            gl.glReadBuffer(gl.GL_NONE)

    def enable_color_output(self, index):
        if self._is_activated:
            if index < len(self._color):
                gl.glDrawBuffer(_color_attachment[index])
                gl.glReadBuffer(_color_attachment[index])

    def resize(self, width, height):
        if self._color:
            for c in self._color:
                c.resize(width, height)
        if self._depth:
            for d in self._depth:
                d.resize(width, height)
        if self._stencil:
            for s in self._stencil:
                s.resize(width, height)

    def save_color_attachment(self, filename=None, save_as_image=True):
        if self._color_enabled:
            with self:
                data = np.zeros((self._height, self._width * 4), dtype=np.uint8)
                gl.glReadBuffer(gl.GL_COLOR_ATTACHMENT0)
                gl.glReadPixels(0, 0, self._width, self._height, gl.GL_RGBA, gl.GL_UNSIGNED_BYTE, data)
                if save_as_image:
                    filename = f'{datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")}_color_attach.png' if filename is None else filename
                    filepath = os.path.join(os.getcwd(), filename)
                    png.from_array(data[::-1], 'RGBA').save(filepath)
                    print(f'Color attachment saved to {filepath}')
                return data

    def save_depth_attachment(self, near, far, is_perspective=True, filename=None, save_as_image=True):
        if self._depth_enabled:
            with self:
                data = np.zeros((self._height, self._width), dtype=np.float32)
                gl.glReadPixels(0, 0, self._width, self._height, gl.GL_DEPTH_COMPONENT, gl.GL_FLOAT, data)
                data = np.vectorize(Framebuffer._depth_value_normalisation)(data, near, far, is_perspective).astype(np.uint8)
                if save_as_image:
                    filename = f'{datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")}_depth_attach.png' if filename is None else filename
                    filepath = os.path.join(os.getcwd(), filename)
                    png.from_array(data[::-1], 'L').save(filepath)
                    print(f'Depth attachment saved to {filepath}')
                return data

    @staticmethod
    def _depth_value_normalisation(d, near, far, is_perspective):
        if is_perspective:
            # from range [0, 1] to NDC in range [-1, 1]
            ndc = d * 2.0 - 1.0
            # inverse the transformation to retrieve linear depth value
            value = (2.0 * near * far) / (far + near - ndc * (far - near))
            # now the value is between near and far
            value /= far - near
            # remap to [0, 255]
            value = math.floor(value * 255.0)
        else:
            value = math.floor(d * 255.0)
        return value

    def save_stencil_attachment(self):
        pass
