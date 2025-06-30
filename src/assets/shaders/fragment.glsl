#VERSION

in vec3 normal;
in float joint_color;
out vec4 fragColor;

void main() {
    vec3 sun_dir = vec3(0.0, -1.0, 0.0);
    float diffuse = max(dot(normal, -sun_dir), 0.0);
    float ambient = 0.4;
    // fragColor = vec4((ambient + diffuse) * vec3(1.0, 1.0, 1.0), 1.0);
    fragColor = vec4(vec3(joint_color), 1.0);
}
