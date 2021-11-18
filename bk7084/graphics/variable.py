import abc
from dataclasses import dataclass

import numpy as np

from .. import gl

gl_type_info = {
    gl.GL_FLOAT: (1, gl.GL_FLOAT, np.float32),
    gl.GL_FLOAT_VEC2: (2, gl.GL_FLOAT, np.float32),
    gl.GL_FLOAT_VEC3: (3, gl.GL_FLOAT, np.float32),
    gl.GL_FLOAT_VEC4: (4, gl.GL_FLOAT, np.float32),
    gl.GL_INT: (1, gl.GL_INT, np.int32),
    gl.GL_INT_VEC2: (2, gl.GL_INT, np.int32),
    gl.GL_INT_VEC3: (3, gl.GL_INT, np.int32),
    gl.GL_INT_VEC4: (4, gl.GL_INT, np.int32),
    gl.GL_BOOL: (1, gl.GL_BOOL, np.bool),
    gl.GL_BOOL_VEC2: (2, gl.GL_BOOL, np.bool),
    gl.GL_BOOL_VEC3: (3, gl.GL_BOOL, np.bool),
    gl.GL_BOOL_VEC4: (4, gl.GL_BOOL, np.bool),
    gl.GL_FLOAT_MAT2: (4, gl.GL_FLOAT, np.float32),
    gl.GL_FLOAT_MAT3: (9, gl.GL_FLOAT, np.float32),
    gl.GL_FLOAT_MAT4: (16, gl.GL_FLOAT, np.float32),
    gl.GL_SAMPLER_1D: (1, gl.GL_UNSIGNED_INT, np.uint32),
    gl.GL_SAMPLER_2D: (1, gl.GL_UNSIGNED_INT, np.uint32),
    gl.GL_SAMPLER_CUBE: (1, gl.GL_UNSIGNED_INT, np.uint32)
}

# A lookup table to specify the value of a uniform variable for shader program.
gl_uniform = {
    gl.GL_FLOAT: gl.glUniform1fv,
    gl.GL_FLOAT_VEC2: gl.glUniform2fv,
    gl.GL_FLOAT_VEC3: gl.glUniform3fv,
    gl.GL_FLOAT_VEC4: gl.glUniform4fv,
    gl.GL_INT: gl.glUniform1iv,
    gl.GL_INT_VEC2: gl.glUniform2iv,
    gl.GL_INT_VEC3: gl.glUniform3iv,
    gl.GL_INT_VEC4: gl.glUniform4iv,
    gl.GL_BOOL: gl.glUniform1iv,
    gl.GL_BOOL_VEC2: gl.glUniform2iv,
    gl.GL_BOOL_VEC3: gl.glUniform3iv,
    gl.GL_BOOL_VEC4: gl.glUniform4iv,
    gl.GL_FLOAT_MAT2: gl.glUniformMatrix2fv,
    gl.GL_FLOAT_MAT3: gl.glUniformMatrix3fv,
    gl.GL_FLOAT_MAT4: gl.glUniformMatrix4fv,
    gl.GL_SAMPLER_1D: gl.glUniform1i,
    gl.GL_SAMPLER_2D: gl.glUniform1i,
    gl.GL_SAMPLER_CUBE: gl.glUniform1i
}

# gl_vertex_attrib = {
#     gl.GL_FLOAT: gl.glVertexAttrib1f,
#     gl.GL_FLOAT_VEC2: gl.glVertexAttrib2f,
#     gl.GL_FLOAT_VEC3: gl.glVertexAttrib3f,
#     gl.GL_FLOAT_VEC4: gl.glVertexAttrib3f,
#     gl.GL_INT: gl.glVertexAttribI1i,
#     gl.GL_INT_VEC2: gl.glVertexAttribI2i,
#     gl.GL_INT_VEC3: gl.glVertexAttribI3i,
#     gl.GL_INT_VEC4: gl.glVertexAttribI4i,
#     gl.GL_BOOL: gl.glVertexAttribI1ui,
#     gl.GL_BOOL_VEC2: gl.glVertexAttrib2fv,
#     gl.GL_BOOL_VEC3: gl.glUniform3iv,
#     gl.GL_BOOL_VEC4: gl.glUniform4iv,
#     gl.GL_FLOAT_MAT2: gl.glUniformMatrix2fv,
#     gl.GL_FLOAT_MAT3: gl.glUniformMatrix3fv,
#     gl.GL_FLOAT_MAT4: gl.glUniformMatrix4fv,
# }


@dataclass
class AbstractDataclass(abc.ABC):
    def __new__(cls, *args, **kwargs):
        if cls == AbstractDataclass or cls.__bases__[0] == AbstractDataclass:
            raise TypeError("Cannot instantiate an abstract class.")
        return super().__new__(cls)


@dataclass
class ShaderVariable(AbstractDataclass):
    """Interface for shader variables: attribute(in) and uniform"""
    program: gl.Constant
    name: str
    gtype: gl.Constant
    dtype: np.dtype
    size: int

    def __init__(self, program, name, gl_type: gl.Constant):
        if gl_type not in gl_type_info:
            raise TypeError("Unknown shader variable type")
        self.program = program
        self.name = name
        self.gtype = gl_type
        self.dtype = gl_type_info['gl_type']


class ShaderAttribute(ShaderVariable):
    def __init__(self, program, name, attr_type, size, location):
        super(ShaderAttribute, self).__init__(program, name)
        self.size = size
        self.location = location


class ShaderUniform(ShaderVariable):
    def __init__(self, program, name, uniform_type, gl_type, location, length, count, gl_setter, gl_getter):
        self.program = program
        self.name = name
        self.type = uniform_type
        self.location = location
        self.length = length
        self.count = count
