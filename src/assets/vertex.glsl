#VERSION

layout(location = 0) in vec2 vPos;
layout(location = 1) in vec3 vCol;

uniform mat3 transform;
uniform mat4 projection;

out vec3 color;

void main() {
    gl_Position = projection * vec4((transform * vec3(vPos, 1.0)).xy, 0.0, 1.0);
    color = vCol;
}
