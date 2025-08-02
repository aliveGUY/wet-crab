#version 300 es
precision mediump float;
out vec4 fragment;

void main()
{
    // Output solid green color for wireframe
    fragment = vec4(0.0, 1.0, 0.0, 1.0);
}
