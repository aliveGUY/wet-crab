#VERSION
layout(location = 0) in vec3 vNorm;
layout(location = 1) in vec3 vPos;
layout(location = 4) in vec2 vTexCoord;

uniform mat4 world_txfm;
uniform mat4 viewport_txfm;

out vec3 norm;
out vec2 texCoord;

void main()
{
    // Simple vertex transformation without skeletal animation
    gl_Position = viewport_txfm * world_txfm * vec4(vPos, 1.0);
    
    // Transform normal with world matrix
    norm = normalize(mat3(world_txfm) * vNorm);
    texCoord = vTexCoord;
}
