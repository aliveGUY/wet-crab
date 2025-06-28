#version 300 es

// Vertex positions for triangle
const vec2 positions[3] = vec2[3](
    vec2( 0.0,  0.5),  // Top vertex
    vec2(-0.5, -0.5),  // Bottom left vertex
    vec2( 0.5, -0.5)   // Bottom right vertex
);

void main() {
    gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
}
