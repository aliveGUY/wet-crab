use glow::HasContext;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderType {
    Static,
    Animated,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub shader_type: ShaderType,
    pub base_color_texture: Option<glow::Texture>,
    #[allow(dead_code)]
    pub metallic_factor: f32,
    #[allow(dead_code)]
    pub roughness_factor: f32,
    #[allow(dead_code)]
    pub double_sided: bool,
}

impl Material {
    pub fn new() -> Self {
        Self {
            shader_type: ShaderType::Static, // Default to static
            base_color_texture: None,
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            double_sided: false,
        }
    }

    pub fn new_with_shader_type(shader_type: ShaderType) -> Self {
        Self {
            shader_type,
            base_color_texture: None,
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            double_sided: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_texture(texture: glow::Texture) -> Self {
        Self {
            shader_type: ShaderType::Static, // Default to static
            base_color_texture: Some(texture),
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            double_sided: false,
        }
    }

    pub fn with_texture_and_shader(texture: glow::Texture, shader_type: ShaderType) -> Self {
        Self {
            shader_type,
            base_color_texture: Some(texture),
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            double_sided: false,
        }
    }

    #[allow(dead_code)]
    pub fn has_texture(&self) -> bool {
        self.base_color_texture.is_some()
    }

    pub fn bind(&self, gl: &glow::Context) {
        if let Some(texture) = self.base_color_texture {
            unsafe {
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            }
        }
    }

    #[allow(dead_code)]
    pub fn cleanup(&self, gl: &glow::Context) {
        if let Some(texture) = self.base_color_texture {
            unsafe {
                gl.delete_texture(texture);
            }
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
