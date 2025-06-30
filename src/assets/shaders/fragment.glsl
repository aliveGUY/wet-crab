#VERSION
in vec3 norm;
in float joint_color;
out vec4 fragment;

void main()
{
    vec3 sun_dir = normalize(vec3(0.0, -1.0, -1.0));
    float diffuse = max(dot(norm, -sun_dir), 0.0);
    float ambient = 0.1;
    fragment = vec4((ambient + diffuse) * vec3(1.0, 1.0, 1.0), 1.0);
}
