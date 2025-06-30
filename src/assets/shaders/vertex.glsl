#VERSION

layout(location = 0) in vec3 vPos;
layout(location = 1) in vec3 vNormal;
uniform mat4 viewport_txfm;
uniform mat4 world_txfm;

out vec3 normal;

void main() {
    gl_Position = viewport_txfm * world_txfm * vec4(vPos, 1.0);
    normal = mat3(world_txfm) * vNormal;
}
