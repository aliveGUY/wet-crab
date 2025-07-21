#version 300 es
layout(location = 0) in vec3 vNorm;
layout(location = 1) in vec3 vPos;
layout(location = 2) in uvec4 vJoints;
layout(location = 3) in vec4 vWeights;
layout(location = 4) in vec2 vTexCoord;

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;
uniform int preview_joint;
uniform mat4 inverse_bone_matrix[20];
uniform mat4 bone_matrix[20];

out vec3 norm;
out vec2 texCoord;
out float joint_color;

void main()
{
    // Transform vertex position with skeletal animation
    gl_Position = vec4(0.0);
    joint_color = 0.0;
    for (int i = 0; i < 4; ++i) { 
        gl_Position += vWeights[i] * (viewport_txfm * world_txfm * bone_matrix[vJoints[i]] * inverse_bone_matrix[vJoints[i]] * vec4(vPos, 1.0));
    }
    
    // Transform normals with skeletal animation (same bone matrices as vertices)
    vec3 transformed_normal = vec3(0.0);
    for (int i = 0; i < 4; ++i) {
        // Transform normal by bone matrix (3x3 part only)
        mat3 bone_normal_matrix = mat3(bone_matrix[vJoints[i]] * inverse_bone_matrix[vJoints[i]]);
        transformed_normal += vWeights[i] * (bone_normal_matrix * vNorm);
    }
    
    // Apply world transform to the animated normal
    norm = normalize(mat3(world_txfm) * transformed_normal);
    texCoord = vTexCoord;
}
