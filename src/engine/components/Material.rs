use glow::HasContext;

#[derive(Debug)]
pub struct Material {
    pub base_color_texture: Option<glow::Texture>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub double_sided: bool,
}

impl Material {
    pub fn new() -> Self {
        Self {
            base_color_texture: None,
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            double_sided: false,
        }
    }

    pub fn with_texture(texture: glow::Texture) -> Self {
        Self {
            base_color_texture: Some(texture),
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            double_sided: false,
        }
    }

    pub fn has_texture(&self) -> bool {
        self.base_color_texture.is_some()
    }

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
