// Import shared components
use crate::index::shared_components::{Transform, Mesh, Material};

// Import animation-specific components
mod skeleton_mod {
    include!("Skeleton.rs");
}
mod animation_mod {
    include!("AnimationState.rs");
}
mod animator_mod {
    include!("Animator.rs");
}

pub use skeleton_mod::*;
pub use animation_mod::*;
pub use animator_mod::Animator;

use glow::HasContext;
use crate::index::math::*;

#[derive(Clone)]
pub struct AnimatedObject3D {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Material,  // Required, no Option
    pub skeleton: Skeleton,  // Required, no Option
    pub animation_channels: Vec<AnimationChannel>,  // Required
    animator: Animator,  // Required, private
}

impl AnimatedObject3D {
    pub fn new(
        transform: Transform, 
        mesh: Mesh, 
        material: Material, 
        skeleton: Skeleton, 
        animation_channels: Vec<AnimationChannel>
    ) -> Self {
        Self {
            transform,
            mesh,
            material,
            skeleton,
            animation_channels,
            animator: Animator::new(),
        }
    }

    pub fn render(&mut self, gl: &glow::Context) {
        // Use shader directly from material
        unsafe {
            gl.use_program(Some(self.material.shader_program));
        }

        // Update animation
        {
            let animation_channels = &self.animation_channels.clone();
            let skeleton = &mut self.skeleton;
            self.animator.update_with_data(animation_channels, skeleton);
        }

        // Bind material (texture)
        self.material.bind(gl);

        unsafe {
            // Get world transform matrix
            let world_txfm = self.transform.get_matrix();
            
            // Bind vertex array
            gl.bind_vertex_array(Some(self.mesh.vao));

            // Calculate bone matrices
            let mut bone_matrices = vec![mat4x4_identity(); 20];
            let mut inverse_bone_matrices = vec![mat4x4_identity(); 20];

            for (i, &joint_id) in self.skeleton.joint_ids.iter().enumerate() {
                if i >= 20 {
                    break;
                }
                inverse_bone_matrices[i] = self.skeleton.joint_inverse_mats[i];
                bone_matrices[i] = node_world_txfm(&self.skeleton.nodes, joint_id as usize);
            }

            // Upload world transform uniform
            if let Some(loc) = gl.get_uniform_location(self.material.shader_program, "world_txfm") {
                gl.uniform_matrix_4_f32_slice(Some(&loc), true, world_txfm);
            }

            // Upload bone matrices
            let flat_inverse: Vec<f32> = inverse_bone_matrices
                .iter()
                .flatten()
                .copied()
                .collect();
            let flat_bones: Vec<f32> = bone_matrices.iter().flatten().copied().collect();

            if let Some(loc) = gl.get_uniform_location(self.material.shader_program, "inverse_bone_matrix") {
                gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_inverse);
            }
            if let Some(loc) = gl.get_uniform_location(self.material.shader_program, "bone_matrix") {
                gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_bones);
            }

            // Draw the mesh
            gl.draw_elements(
                glow::TRIANGLES,
                self.mesh.index_count as i32,
                glow::UNSIGNED_SHORT,
                0
            );
        }
    }
}
