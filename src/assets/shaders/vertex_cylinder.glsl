#version 300 es

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;
uniform float radius;
uniform float height;

const float PI = 3.14159265359;

void main()
{
    // Generate cylinder wireframe using GL_LINES: top circle + bottom circle + vertical edges
    // 2 circles with 32 segments each = 64 lines = 128 vertices
    // 4 vertical lines = 4 lines = 8 vertices
    // Total: 136 vertices (68 lines)
    
    float half_height = height * 0.5;
    vec3 pos = vec3(0.0);
    
    if (gl_VertexID < 128) {
        // Top and bottom circles (64 lines = 128 vertices)
        int line_id = gl_VertexID / 2;
        int vertex_in_line = gl_VertexID % 2;
        
        if (line_id < 32) {
            // Top circle
            float t1 = float(line_id) / 32.0 * 2.0 * PI;
            float t2 = float(line_id + 1) / 32.0 * 2.0 * PI;
            float t = vertex_in_line == 0 ? t1 : t2;
            pos = vec3(
                radius * cos(t),
                half_height,
                radius * sin(t)
            );
        } else {
            // Bottom circle
            int circle_seg = line_id - 32;
            float t1 = float(circle_seg) / 32.0 * 2.0 * PI;
            float t2 = float(circle_seg + 1) / 32.0 * 2.0 * PI;
            float t = vertex_in_line == 0 ? t1 : t2;
            pos = vec3(
                radius * cos(t),
                -half_height,
                radius * sin(t)
            );
        }
    } else {
        // Vertical connecting lines (4 lines = 8 vertices)
        int line_id = (gl_VertexID - 128) / 2;
        int vertex_in_line = (gl_VertexID - 128) % 2;
        
        if (line_id == 0) {
            // Front vertical line
            pos = vec3(
                radius,
                vertex_in_line == 0 ? -half_height : half_height,
                0.0
            );
        } else if (line_id == 1) {
            // Back vertical line
            pos = vec3(
                -radius,
                vertex_in_line == 0 ? -half_height : half_height,
                0.0
            );
        } else if (line_id == 2) {
            // Right vertical line
            pos = vec3(
                0.0,
                vertex_in_line == 0 ? -half_height : half_height,
                radius
            );
        } else {
            // Left vertical line
            pos = vec3(
                0.0,
                vertex_in_line == 0 ? -half_height : half_height,
                -radius
            );
        }
    }
    
    gl_Position = viewport_txfm * world_txfm * vec4(pos, 1.0);
}
