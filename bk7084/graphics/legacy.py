from .. import gl


def draw(obj):
    from ..geometry.triangle import Triangle
    from ..geometry.ray import Ray

    if isinstance(obj, Triangle):
        gl.glColor4f(*obj.color)
        gl.glEnableClientState(gl.GL_VERTEX_ARRAY)
        gl.glVertexPointer(3, gl.GL_FLOAT, 0, obj.vertices)
        gl.glDrawArrays(gl.GL_TRIANGLES, 0, 3)
        gl.glDisableClientState(gl.GL_VERTEX_ARRAY)

    if isinstance(obj, Ray):
        gl.glColor4f(*obj.color)
        gl.glBegin(gl.GL_LINES)
        gl.glVertex3f(obj.origin.x, obj.origin.y, obj.origin.z)
        gl.glVertex3f(obj.origin.x + obj.direction.x, obj.origin.y + obj.direction.y, obj.origin.z + obj.direction.z)
        gl.glEnd()
