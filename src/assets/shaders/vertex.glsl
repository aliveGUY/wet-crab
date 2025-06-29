#VERSION

layout(location = 1) in vec3 vCol;
layout(location = 0) in vec3 vPos;
uniform mat4 transform;

out vec3 color;

void main() {
    gl_Position = transform * vec4(vPos, 1.0);
    color = vCol;
}
