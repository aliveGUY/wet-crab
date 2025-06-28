#VERSION

layout(location = 0) in vec2 vPos;
layout(location = 1) in vec3 vCol;

uniform mat2 rot;

out vec3 color;

void main() {
    gl_Position = vec4(rot * vPos, 0.0, 1.0);
    color = vCol;
}
