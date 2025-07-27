#version 300 es
layout(location = 0) in vec3 vNorm;
layout(location = 1) in vec3 vPos;
layout(location = 4) in vec2 vTexCoord;

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;
uniform float outline_scale;

void main()
{
    // Scale the vertex position for outline effect
    vec3 scaled_pos = vPos * outline_scale;
    
    // Transform the scaled position
    gl_Position = viewport_txfm * world_txfm * vec4(scaled_pos, 1.0);
}
