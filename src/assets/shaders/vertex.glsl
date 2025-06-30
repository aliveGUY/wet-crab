#VERSION

layout(location = 0) in vec3 vPos;
layout(location = 1) in vec3 vNormal;
layout(location = 2) in uvec4 vJoints;
layout(location = 3) in vec4 vWeights;
uniform mat4 viewport_txfm;
uniform mat4 world_txfm;
uniform uint preview_joint;

out vec3 normal;
out float joint_color;

void main() {
    gl_Position = viewport_txfm * world_txfm * vec4(vPos, 1.0);
    joint_color = 0.0;

    for(int i = 0; i < 4; i++) {
        if(vJoints[i] == preview_joint && vWeights[i] > 0.0)
            joint_color = vWeights[i];
    }

    normal = mat3(world_txfm) * vNormal;
}
