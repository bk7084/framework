import abc
from dataclasses import dataclass

import numpy as np

from .. import gl

glsl_types = {
    'float': gl.GL_FLOAT,
    'vec2': gl.GL_FLOAT_VEC2,
    'vec3': gl.GL_FLOAT_VEC3,
    'vec4': gl.GL_FLOAT_VEC4,
    'int': gl.GL_INT,
    'ivec2': gl.GL_INT_VEC2,
    'ivec3': gl.GL_INT_VEC3,
    'ivec4': gl.GL_INT_VEC4,
    'bool': gl.GL_BOOL,
    'bvec2': gl.GL_BOOL_VEC2,
    'bvec3': gl.GL_BOOL_VEC3,
    'bvec4': gl.GL_BOOL_VEC4,
    'mat2': gl.GL_FLOAT_MAT2,
    'mat3': gl.GL_FLOAT_MAT3,
    'mat4': gl.GL_FLOAT_MAT4,
    'sampler1D': gl.GL_SAMPLER_1D,
    'sampler2D': gl.GL_SAMPLER_2D,
    'samplerCube': gl.GL_SAMPLER_CUBE,
}

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
gl_set_uniform = {
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
    gl.GL_SAMPLER_1D: gl.glUniform1iv,
    gl.GL_SAMPLER_2D: gl.glUniform1iv,
    gl.GL_SAMPLER_CUBE: gl.glUniform1iv
}

gl_get_uniform = {
    gl.GL_FLOAT: gl.glUniform1fv,
    gl.GL_FLOAT_VEC2: gl.glGetnUniformfv,
    gl.GL_FLOAT_VEC3: gl.glGetnUniformfv,
    gl.GL_FLOAT_VEC4: gl.glGetnUniformfv,
    gl.GL_INT: gl.glGetnUniformiv,
    gl.GL_INT_VEC2: gl.glGetnUniformiv,
    gl.GL_INT_VEC3: gl.glGetnUniformiv,
    gl.GL_INT_VEC4: gl.glGetnUniformiv,
    gl.GL_BOOL: gl.glGetnUniformiv,
    gl.GL_BOOL_VEC2: gl.glGetnUniformiv,
    gl.GL_BOOL_VEC3: gl.glGetnUniformiv,
    gl.GL_BOOL_VEC4: gl.glGetnUniformiv,
    gl.GL_FLOAT_MAT2: gl.glGetnUniformfv,
    gl.GL_FLOAT_MAT3: gl.glGetnUniformfv,
    gl.GL_FLOAT_MAT4: gl.glGetnUniformfv,
    gl.GL_SAMPLER_1D: gl.glGetnUniformiv,
    gl.GL_SAMPLER_2D: gl.glGetnUniformiv,
    gl.GL_SAMPLER_CUBE: gl.glGetnUniformiv
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
