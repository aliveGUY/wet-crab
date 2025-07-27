#version 300 es
layout(location = 0) in vec3 vNorm;
layout(location = 1) in vec3 vPos;
layout(location = 2) in uvec4 vJoints;
layout(location = 3) in vec4 vWeights;
layout(location = 4) in vec2 vTexCoord;

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;
uniform float outline_scale;
uniform mat4 inverse_bone_matrix[20];
uniform mat4 bone_matrix[20];

void main()
{
    // Scale the vertex position for outline effect
    vec3 scaled_pos = vPos * outline_scale;
    
    // Transform vertex position with skeletal animation (using scaled position)
    gl_Position = vec4(0.0);
    for (int i = 0; i < 4; ++i) { 
        gl_Position += vWeights[i] * (viewport_txfm * world_txfm * bone_matrix[vJoints[i]] * inverse_bone_matrix[vJoints[i]] * vec4(scaled_pos, 1.0));
    }
}
