#version 300 es

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;
uniform float radius;
uniform float height;

const float PI = 3.14159265359;

void main()
{
    // Generate capsule wireframe with complete hemispheres using GL_LINES
    // 2 circles (32 segments each) = 64 lines = 128 vertices
    // 2 vertical lines = 2 lines = 4 vertices  
    // 8 hemisphere meridians (16 segments each) = 128 lines = 256 vertices
    // 4 hemisphere latitude circles (16 segments each) = 64 lines = 128 vertices
    // Total: 516 vertices (258 lines)
    
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
    } else if (gl_VertexID < 132) {
        // Vertical connecting lines (2 lines = 4 vertices)
        int line_id = (gl_VertexID - 128) / 2;
        int vertex_in_line = (gl_VertexID - 128) % 2;
        
        if (line_id == 0) {
            // Front vertical line
            pos = vec3(
                radius,
                vertex_in_line == 0 ? -half_height : half_height,
                0.0
            );
        } else {
            // Back vertical line
            pos = vec3(
                -radius,
                vertex_in_line == 0 ? -half_height : half_height,
                0.0
            );
        }
    } else if (gl_VertexID < 388) {
        // Hemisphere meridians (8 meridians × 16 segments = 128 lines = 256 vertices)
        int meridian_vertex_id = gl_VertexID - 132;
        int line_id = meridian_vertex_id / 2;
        int vertex_in_line = meridian_vertex_id % 2;
        int seg_id = line_id % 16;           // 0-15 segments within meridian
        int meridian_id = (line_id / 16);    // 0-7 meridians total
        int hemisphere = meridian_id / 4;    // 0 = top, 1 = bottom  
        int local_meridian = meridian_id % 4; // 0-3 meridians per hemisphere
        
        float angle = float(local_meridian) * PI * 0.5; // 0°, 90°, 180°, 270°
        float t1 = float(seg_id) / 16.0 * PI * 0.5;
        float t2 = float(seg_id + 1) / 16.0 * PI * 0.5;
        float t = vertex_in_line == 0 ? t1 : t2;
        
        if (hemisphere == 0) {
            // Top hemisphere meridians
            pos = vec3(
                radius * cos(t) * cos(angle),
                half_height + radius * sin(t),
                radius * cos(t) * sin(angle)
            );
        } else {
            // Bottom hemisphere meridians
            pos = vec3(
                radius * cos(t) * cos(angle),
                -half_height - radius * sin(t),
                radius * cos(t) * sin(angle)
            );
        }
    } else {
        // Hemisphere latitude circles (4 circles × 16 segments = 64 lines = 128 vertices)
        int lat_vertex_id = gl_VertexID - 388;
        int line_id = lat_vertex_id / 2;
        int vertex_in_line = lat_vertex_id % 2;
        int circle_id = line_id / 16;
        int seg_id = line_id % 16;
        
        float t1 = float(seg_id) / 16.0 * 2.0 * PI;
        float t2 = float(seg_id + 1) / 16.0 * 2.0 * PI;
        float t = vertex_in_line == 0 ? t1 : t2;
        
        if (circle_id < 2) {
            // Top hemisphere latitude circles
            float lat_angle = (float(circle_id) + 1.0) * PI * 0.25; // 45°, 90°
            float circle_radius = radius * cos(lat_angle);
            float circle_height = radius * sin(lat_angle);
            pos = vec3(
                circle_radius * cos(t),
                half_height + circle_height,
                circle_radius * sin(t)
            );
        } else {
            // Bottom hemisphere latitude circles
            int bottom_circle = circle_id - 2;
            float lat_angle = (float(bottom_circle) + 1.0) * PI * 0.25; // 45°, 90°
            float circle_radius = radius * cos(lat_angle);
            float circle_height = radius * sin(lat_angle);
            pos = vec3(
                circle_radius * cos(t),
                -half_height - circle_height,
                circle_radius * sin(t)
            );
        }
    }
    
    gl_Position = viewport_txfm * world_txfm * vec4(pos, 1.0);
}
