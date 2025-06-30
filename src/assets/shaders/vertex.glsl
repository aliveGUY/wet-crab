#VERSION
layout(location = 0) in vec3 vNorm;
layout(location = 1) in vec3 vPos;
layout(location = 2) in uvec4 vJoints;
layout(location = 3) in vec4 vWeights;

layout(location = 0) uniform mat4 world_txfm;
layout(location = 1) uniform mat4 viewport_txfm;
layout(location = 2) uniform uint preview_joint = 2;
layout(location = 20) uniform mat4 inverse_bone_matrix[20];
layout(location = 40) uniform mat4 bone_matrix[20];

out vec3 norm;
out float joint_color;

void main()
{
    gl_Position = vec4(0.0);
    joint_color = 0.0;
    for (int i = 0; i < 4; ++i) { 
        gl_Position += vWeights[i] * (viewport_txfm * world_txfm * bone_matrix[vJoints[i]] * inverse_bone_matrix[vJoints[i]] * vec4(vPos, 1.0));
    }
    norm = mat3(world_txfm) * vNorm;
}
