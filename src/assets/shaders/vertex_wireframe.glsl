#version 300 es
layout(location = 0) in vec3 vPos;

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;

void main()
{
    // Simple vertex transformation for wireframe rendering
    gl_Position = viewport_txfm * world_txfm * vec4(vPos, 1.0);
}
