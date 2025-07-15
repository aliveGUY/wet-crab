use std::sync::Arc;
use glow::HasContext;

/// Trait for individual rendering passes
pub trait RenderPass {
    /// Set up OpenGL state for this rendering pass
    fn setup(&mut self, gl: &glow::Context, width: u32, height: u32);
    
    /// Execute the rendering for this pass
    fn render(&mut self, gl: &glow::Context, width: u32, height: u32);
    
    /// Clean up and restore OpenGL state after this pass
    fn cleanup(&mut self, gl: &glow::Context);
    
    /// Get the name of this pass for debugging
    fn name(&self) -> &'static str;
}

/// Manages multiple rendering passes with proper state isolation
pub struct RenderPassManager {
    gl: Arc<glow::Context>,
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderPassManager {
    /// Create a new render pass manager
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            gl,
            passes: Vec::new(),
        }
    }
    
    /// Add a rendering pass to the manager
    pub fn add_pass(&mut self, pass: Box<dyn RenderPass>) {
        self.passes.push(pass);
    }
    
    /// Execute all rendering passes in order with proper state isolation
    pub fn execute_passes(&mut self, width: u32, height: u32) {
        for pass in &mut self.passes {
            // Set up the pass
            pass.setup(&self.gl, width, height);
            
            // Execute the pass
            pass.render(&self.gl, width, height);
            
            // Clean up the pass
            pass.cleanup(&self.gl);
        }
    }
    
    /// Get the number of registered passes
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }
    
    /// Get pass names for debugging
    pub fn get_pass_names(&self) -> Vec<&'static str> {
        self.passes.iter().map(|pass| pass.name()).collect()
    }
}

/// OpenGL state snapshot for restoration
#[derive(Debug, Clone)]
pub struct OpenGLState {
    pub viewport: [i32; 4],
    pub depth_test_enabled: bool,
    pub blend_enabled: bool,
    pub cull_face_enabled: bool,
    pub current_program: Option<glow::Program>,
    pub bound_vao: Option<glow::VertexArray>,
    pub active_texture: i32,
}

impl OpenGLState {
    /// Capture current OpenGL state
    pub fn capture(gl: &glow::Context) -> Self {
        unsafe {
            let mut viewport = [0i32; 4];
            gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);
            
            Self {
                viewport,
                depth_test_enabled: gl.is_enabled(glow::DEPTH_TEST),
                blend_enabled: gl.is_enabled(glow::BLEND),
                cull_face_enabled: gl.is_enabled(glow::CULL_FACE),
                current_program: None, // We'll skip program state for now
                bound_vao: None, // We'll skip VAO state for now
                active_texture: gl.get_parameter_i32(glow::ACTIVE_TEXTURE),
            }
        }
    }
    
    /// Restore this OpenGL state
    pub fn restore(&self, gl: &glow::Context) {
        unsafe {
            // Restore viewport
            gl.viewport(self.viewport[0], self.viewport[1], self.viewport[2], self.viewport[3]);
            
            // Restore depth test
            if self.depth_test_enabled {
                gl.enable(glow::DEPTH_TEST);
            } else {
                gl.disable(glow::DEPTH_TEST);
            }
            
            // Restore blending
            if self.blend_enabled {
                gl.enable(glow::BLEND);
            } else {
                gl.disable(glow::BLEND);
            }
            
            // Restore face culling
            if self.cull_face_enabled {
                gl.enable(glow::CULL_FACE);
            } else {
                gl.disable(glow::CULL_FACE);
            }
            
            // Restore shader program
            gl.use_program(self.current_program);
            
            // Restore VAO
            gl.bind_vertex_array(self.bound_vao);
            
            // Restore active texture
            gl.active_texture(self.active_texture as u32);
        }
    }
}
