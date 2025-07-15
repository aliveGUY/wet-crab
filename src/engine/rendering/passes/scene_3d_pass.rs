use glow::HasContext;
use crate::index::game::systems::renderSystem::RenderSystem;
use super::super::render_pass_manager::{RenderPass, OpenGLState};

/// Rendering pass for 3D scene objects (meshes, models, etc.)
pub struct Scene3DPass {
    /// Saved OpenGL state before this pass
    saved_state: Option<OpenGLState>,
    /// Clear color for the 3D scene
    clear_color: [f32; 4],
}

impl Scene3DPass {
    /// Create a new 3D scene rendering pass
    pub fn new() -> Self {
        Self {
            saved_state: None,
            clear_color: [0.1, 0.1, 0.1, 1.0], // Dark gray background
        }
    }
    
    /// Set the clear color for the 3D scene
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = [r, g, b, a];
    }
}

impl RenderPass for Scene3DPass {
    fn setup(&mut self, gl: &glow::Context, width: u32, height: u32) {
        // Save current OpenGL state
        self.saved_state = Some(OpenGLState::capture(gl));
        
        unsafe {
            // Set up 3D rendering state
            gl.viewport(0, 0, width as i32, height as i32);
            
            // Enable depth testing for 3D rendering
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            
            // Disable blending for opaque 3D objects
            gl.disable(glow::BLEND);
            
            // Enable face culling for performance
            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
            gl.front_face(glow::CCW);
            
            // Clear the screen with the 3D scene background color
            gl.clear_color(
                self.clear_color[0],
                self.clear_color[1], 
                self.clear_color[2],
                self.clear_color[3]
            );
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }
    
    fn render(&mut self, gl: &glow::Context, width: u32, height: u32) {
        // Call the existing RenderSystem to render all 3D objects
        RenderSystem::update(gl, width, height);
        
        unsafe {
            // Ensure no VAO is bound after 3D rendering
            gl.bind_vertex_array(None);
            
            // Ensure no shader program is active
            gl.use_program(None);
        }
    }
    
    fn cleanup(&mut self, gl: &glow::Context) {
        // Restore the previous OpenGL state if we saved it
        if let Some(ref saved_state) = self.saved_state {
            saved_state.restore(gl);
        }
        
        // Clear the saved state
        self.saved_state = None;
    }
    
    fn name(&self) -> &'static str {
        "Scene3D"
    }
}

impl Default for Scene3DPass {
    fn default() -> Self {
        Self::new()
    }
}
