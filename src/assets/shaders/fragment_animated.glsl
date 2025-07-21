#version 300 es
precision mediump float;
in vec3 norm;
in vec2 texCoord;
in float joint_color;
out vec4 fragment;

uniform sampler2D baseColorTexture;
uniform bool hasTexture;

void main()
{
    // Top-down directional light
    vec3 light_dir = normalize(vec3(0.0, -1.0, 0.0)); // Pure top-down light
    float diffuse = max(dot(norm, -light_dir), 0.0);
    float ambient = 0.2; // Slightly higher ambient for top-down lighting
    
    // Default orange/tan color for the character
    vec3 baseColor = vec3(0.8, 0.6, 0.4);
    if (hasTexture) {
        vec4 texColor = texture(baseColorTexture, texCoord);
        baseColor = texColor.rgb;
        
        // Preserve very dark colors (black regions like pupils, mouth)
        if (texColor.r < 0.1 && texColor.g < 0.1 && texColor.b < 0.1) {
            // For very dark pixels, use minimal lighting to preserve black colors
            fragment = vec4(texColor.rgb * (ambient + diffuse * 0.1), 1.0);
            return;
        }
    }
    
    // Apply dynamic lighting that responds to surface orientation
    float lighting = ambient + diffuse * 0.8;
    fragment = vec4(lighting * baseColor, 1.0);
}
