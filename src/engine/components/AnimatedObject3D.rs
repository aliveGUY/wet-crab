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
            // Convert to the types expected by the animator
            let animation_channels: Vec<crate::index::animated_object3d::AnimationChannel> = 
                self.animation_channels.iter().map(|ch| crate::index::animated_object3d::AnimationChannel {
                    target: ch.target,
                    animation_type: match ch.animation_type {
                        AnimationType::Translation => crate::index::animated_object3d::AnimationType::Translation,
                        AnimationType::Rotation => crate::index::animated_object3d::AnimationType::Rotation,
                        AnimationType::Scale => crate::index::animated_object3d::AnimationType::Scale,
                    },
                    num_timesteps: ch.num_timesteps,
                    times: ch.times.clone(),
                    data: ch.data.clone(),
                }).collect();
            
            // Convert skeleton to the expected type
            let mut skeleton_converted = crate::index::animated_object3d::Skeleton {
                nodes: self.skeleton.nodes.iter().map(|n| crate::index::animated_object3d::Node {
                    translation: n.translation,
                    rotation: n.rotation,
                    scale: n.scale,
                    parent: n.parent,
                }).collect(),
                joint_ids: self.skeleton.joint_ids.clone(),
                joint_inverse_mats: self.skeleton.joint_inverse_mats.clone(),
            };
            
            self.animator.update_with_data(&animation_channels[..], &mut skeleton_converted);
            
            // Copy back the updated nodes
            for (i, node) in skeleton_converted.nodes.iter().enumerate() {
                if i < self.skeleton.nodes.len() {
                    self.skeleton.nodes[i].translation = node.translation;
                    self.skeleton.nodes[i].rotation = node.rotation;
                    self.skeleton.nodes[i].scale = node.scale;
                }
            }
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
                // Convert nodes to the expected type for the math function
                let nodes_converted: Vec<crate::index::animated_object3d::Node> = 
                    self.skeleton.nodes.iter().map(|n| crate::index::animated_object3d::Node {
                        translation: n.translation,
                        rotation: n.rotation,
                        scale: n.scale,
                        parent: n.parent,
                    }).collect();
                bone_matrices[i] = node_world_txfm(&nodes_converted[..], joint_id as usize);
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
