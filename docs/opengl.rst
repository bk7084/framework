OpenGL
======

Different ways of drawing:

Immediate mode:

glBegin()/glEnd()
glVertex3f(x, y, z)


Modern OpenGL:

https://stackoverflow.com/questions/8704801/glvertexattribpointer-clarification

1. Separate VBO per vertex attribute.
https://stackoverflow.com/questions/7223623/storing-different-vertex-attributes-in-different-vbos

2. All attributes in the same VBO.
3. Minimizing draw calls using VAO.

Vertex Array Object (VAO). The VAO contains information about the connections between the data in our buffers and the input vertex attributes.

- 000: only with vertex position
- 001: vertex position and color but not in the same buffer
- 002: vertex position and color, in the same buffer
