#VERSION

layout(location = 0) in vec3 vPos;
layout(location = 1) in vec3 vNormal;
uniform mat4 viewport_transform;
uniform mat4 world_transform;

out vec3 normal;

void main() {
    gl_Position = viewport_transform * world_transform * vec4(vPos, 1.0);
    normal = mat3(world_transform) * vNormal;
}
