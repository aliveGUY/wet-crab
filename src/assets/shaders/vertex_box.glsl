#version 300 es

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;
uniform vec3 half_extents;

void main()
{
    // Generate 12 edges of a wireframe box using GL_LINES
    // Each edge needs 2 vertices, so 24 vertices total
    int edge_id = gl_VertexID / 2;
    int vertex_in_edge = gl_VertexID % 2;
    
    vec3 pos = vec3(0.0);
    
    // Define the 12 edges of a box
    if (edge_id == 0) {
        // Bottom face edges (4 edges)
        pos = vertex_in_edge == 0 ? vec3(-half_extents.x, -half_extents.y, -half_extents.z) : vec3(half_extents.x, -half_extents.y, -half_extents.z);
    } else if (edge_id == 1) {
        pos = vertex_in_edge == 0 ? vec3(half_extents.x, -half_extents.y, -half_extents.z) : vec3(half_extents.x, -half_extents.y, half_extents.z);
    } else if (edge_id == 2) {
        pos = vertex_in_edge == 0 ? vec3(half_extents.x, -half_extents.y, half_extents.z) : vec3(-half_extents.x, -half_extents.y, half_extents.z);
    } else if (edge_id == 3) {
        pos = vertex_in_edge == 0 ? vec3(-half_extents.x, -half_extents.y, half_extents.z) : vec3(-half_extents.x, -half_extents.y, -half_extents.z);
    } else if (edge_id == 4) {
        // Top face edges (4 edges)
        pos = vertex_in_edge == 0 ? vec3(-half_extents.x, half_extents.y, -half_extents.z) : vec3(half_extents.x, half_extents.y, -half_extents.z);
    } else if (edge_id == 5) {
        pos = vertex_in_edge == 0 ? vec3(half_extents.x, half_extents.y, -half_extents.z) : vec3(half_extents.x, half_extents.y, half_extents.z);
    } else if (edge_id == 6) {
        pos = vertex_in_edge == 0 ? vec3(half_extents.x, half_extents.y, half_extents.z) : vec3(-half_extents.x, half_extents.y, half_extents.z);
    } else if (edge_id == 7) {
        pos = vertex_in_edge == 0 ? vec3(-half_extents.x, half_extents.y, half_extents.z) : vec3(-half_extents.x, half_extents.y, -half_extents.z);
    } else if (edge_id == 8) {
        // Vertical edges (4 edges)
        pos = vertex_in_edge == 0 ? vec3(-half_extents.x, -half_extents.y, -half_extents.z) : vec3(-half_extents.x, half_extents.y, -half_extents.z);
    } else if (edge_id == 9) {
        pos = vertex_in_edge == 0 ? vec3(half_extents.x, -half_extents.y, -half_extents.z) : vec3(half_extents.x, half_extents.y, -half_extents.z);
    } else if (edge_id == 10) {
        pos = vertex_in_edge == 0 ? vec3(half_extents.x, -half_extents.y, half_extents.z) : vec3(half_extents.x, half_extents.y, half_extents.z);
    } else if (edge_id == 11) {
        pos = vertex_in_edge == 0 ? vec3(-half_extents.x, -half_extents.y, half_extents.z) : vec3(-half_extents.x, half_extents.y, half_extents.z);
    }
    
    gl_Position = viewport_txfm * world_txfm * vec4(pos, 1.0);
}
