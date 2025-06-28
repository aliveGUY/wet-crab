#version 300 es

uniform float u_aspect_ratio;

// Centered triangle vertices
const vec2 positions[3] = vec2[3](
    vec2( 0.0,  0.3),  // Top vertex (centered)
    vec2(-0.3, -0.3),  // Bottom left vertex
    vec2( 0.3, -0.3)   // Bottom right vertex
);

void main() {
    vec2 pos = positions[gl_VertexID];
    
    // Apply aspect ratio correction to maintain square proportions
    if (u_aspect_ratio > 1.0) {
        // Window is wider than tall - compress horizontally
        pos.x /= u_aspect_ratio;
    } else {
        // Window is taller than wide - compress vertically
        pos.y *= u_aspect_ratio;
    }
    
    gl_Position = vec4(pos, 0.0, 1.0);
}
