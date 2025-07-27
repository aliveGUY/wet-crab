#version 300 es
precision mediump float;

uniform vec3 outline_color;
out vec4 fragment;

void main()
{
    // Simple solid color output for outline
    fragment = vec4(outline_color, 1.0);
}
