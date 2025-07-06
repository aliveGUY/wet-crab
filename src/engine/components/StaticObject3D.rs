// Import shared components
use crate::index::shared_components::{Transform, Mesh, Material};
use glow::HasContext;

#[derive(Clone)]
pub struct StaticObject3D {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Material,  // Required, no Option
}

impl StaticObject3D {
    pub fn new(transform: Transform, mesh: Mesh, material: Material) -> Self {
        Self {
            transform,
            mesh,
            material,
        }
    }

    pub fn render(&mut self, gl: &glow::Context) {
        // Use shader directly from material
        unsafe {
            gl.use_program(Some(self.material.shader_program));
        }

        // Bind material (texture)
        self.material.bind(gl);

        unsafe {
            // Get world transform matrix
            let world_txfm = self.transform.get_matrix();
            
            // Bind vertex array
            gl.bind_vertex_array(Some(self.mesh.vao));

            // Upload world transform uniform
            if let Some(loc) = gl.get_uniform_location(self.material.shader_program, "world_txfm") {
                gl.uniform_matrix_4_f32_slice(Some(&loc), true, world_txfm);
            }

            // Draw the mesh (simple static rendering)
            gl.draw_elements(
                glow::TRIANGLES,
                self.mesh.index_count as i32,
                glow::UNSIGNED_SHORT,
                0
            );
        }
    }
}
