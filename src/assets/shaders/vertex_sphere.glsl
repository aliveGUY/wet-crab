#version 300 es

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;
uniform float radius;

const float PI = 3.14159265359;

void main()
{
    // Generate sphere wireframe using GL_LINES: 2 vertical meridians + 1 horizontal equator
    // 3 circles, each with 32 segments = 96 lines = 192 vertices
    int line_id = gl_VertexID / 2;
    int vertex_in_line = gl_VertexID % 2;
    int circle_id = line_id / 32;
    int seg_id = line_id % 32;
    
    float t1 = float(seg_id) / 32.0 * 2.0 * PI;
    float t2 = float(seg_id + 1) / 32.0 * 2.0 * PI;
    float t = vertex_in_line == 0 ? t1 : t2;
    vec3 pos = vec3(0.0);
    
    if (circle_id == 0) {
        // First vertical meridian (longitude 0)
        pos = vec3(
            radius * sin(t),
            radius * cos(t),
            0.0
        );
    } else if (circle_id == 1) {
        // Second vertical meridian (longitude 90 degrees)
        pos = vec3(
            0.0,
            radius * cos(t),
            radius * sin(t)
        );
    } else if (circle_id == 2) {
        // Horizontal equator
        pos = vec3(
            radius * cos(t),
            0.0,
            radius * sin(t)
        );
    }
    
    gl_Position = viewport_txfm * world_txfm * vec4(pos, 1.0);
}
