#version 330 core
layout(location = 0) in vec3 aPos;

uniform mat4 viewport_txfm;
uniform mat4 world_txfm;

void main() {
    gl_Position = viewport_txfm * world_txfm * vec4(aPos, 1.0);
}
