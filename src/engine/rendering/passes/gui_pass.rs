use glow::HasContext;
use crate::index::engine::gui::UIManager;
use super::super::render_pass_manager::{RenderPass, OpenGLState};

/// Rendering pass for GUI elements (egui windows, buttons, etc.)
pub struct GuiPass {
    /// The UI manager that handles GUI rendering
    ui_manager: UIManager,
    /// Saved OpenGL state before this pass
    saved_state: Option<OpenGLState>,
}

impl GuiPass {
    /// Create a new GUI rendering pass
    pub fn new(ui_manager: UIManager) -> Self {
        Self {
            ui_manager,
            saved_state: None,
        }
    }
    
    /// Get a mutable reference to the UI manager for external updates
    pub fn ui_manager_mut(&mut self) -> &mut UIManager {
        &mut self.ui_manager
    }
    
    /// Get a reference to the UI manager
    pub fn ui_manager(&self) -> &UIManager {
        &self.ui_manager
    }
}

impl RenderPass for GuiPass {
    fn setup(&mut self, gl: &glow::Context, _width: u32, _height: u32) {
        // Save current OpenGL state before GUI rendering
        self.saved_state = Some(OpenGLState::capture(gl));
        
        // Note: egui_glow::Painter will handle its own OpenGL state setup
        // We don't need to manually configure OpenGL state here as egui
        // will set up blending, disable depth testing, etc. as needed
    }
    
    fn render(&mut self, gl: &glow::Context, width: u32, height: u32) {
        // Let the UI manager handle GUI rendering
        // The UIManager will internally use egui_glow::Painter which
        // will properly configure OpenGL state for 2D GUI rendering
        self.ui_manager.update(gl, width, height);
    }
    
    fn cleanup(&mut self, gl: &glow::Context) {
        // Restore the OpenGL state that was active before GUI rendering
        if let Some(ref saved_state) = self.saved_state {
            saved_state.restore(gl);
        }
        
        // Clear the saved state
        self.saved_state = None;
        
        // Additional cleanup to ensure GUI doesn't affect subsequent rendering
        unsafe {
            // Ensure no GUI-specific state is left active
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, None);
            
            // Reset to a known state for 3D rendering
            // (This will be overridden by the next pass's setup, but ensures consistency)
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
        }
    }
    
    fn name(&self) -> &'static str {
        "GUI"
    }
}
