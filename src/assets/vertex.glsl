#VERSION

layout(location = 0) in vec2 vPos;
layout(location = 1) in vec3 vCol;

uniform mat2 rot;
uniform mat4 projection;

out vec3 color;

void main() {
    vec2 rotated_pos = rot * vPos;
    gl_Position = projection * vec4(rotated_pos, 0.0, 1.0);
    color = vCol;
}
