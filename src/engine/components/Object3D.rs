// Import components
mod transform_mod {
    include!("Tranform.rs");
}
mod mesh_mod {
    include!("Mesh.rs");
}
mod material_mod {
    include!("Material.rs");
}
mod skeleton_mod {
    include!("Skeleton.rs");
}
mod animation_mod {
    include!("AnimationState.rs");
}
mod animator_mod {
    include!("Animator.rs");
}

pub use transform_mod::Transform;
pub use mesh_mod::Mesh;
pub use material_mod::Material;
pub use skeleton_mod::*;
pub use animation_mod::*;
pub use animator_mod::Animator;

use glow::HasContext;
use crate::index::math::*;

#[derive(Clone)]
pub struct Object3D {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Option<Material>,
    pub skeleton: Option<Skeleton>,
    pub animation_channels: Vec<AnimationChannel>,
    animator: Animator,
}

impl Object3D {
    pub fn new() -> Self {
        Self {
            transform: Transform::new(),
            mesh: Mesh::new(),
            material: None,
            skeleton: None,
            animation_channels: Vec::new(),
            animator: Animator::new(),
        }
    }

    pub fn with_mesh(mesh: Mesh) -> Self {
        Self {
            transform: Transform::new(),
            mesh,
            material: None,
            skeleton: None,
            animation_channels: Vec::new(),
            animator: Animator::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_transform_and_mesh(transform: Transform, mesh: Mesh) -> Self {
        Self {
            transform,
            mesh,
            material: None,
            skeleton: None,
            animation_channels: Vec::new(),
            animator: Animator::new(),
        }
    }

    pub fn set_material(&mut self, material: Material) {
        self.material = Some(material);
    }

    #[allow(dead_code)]
    pub fn has_material(&self) -> bool {
        self.material.is_some()
    }

    pub fn set_skeleton(&mut self, skeleton: Skeleton) {
        self.skeleton = Some(skeleton);
    }

    pub fn set_animation_channels(&mut self, channels: Vec<AnimationChannel>) {
        self.animation_channels = channels;
    }

    #[allow(dead_code)]
    pub fn add_animation_channel(&mut self, channel: AnimationChannel) {
        self.animation_channels.push(channel);
    }

    #[allow(dead_code)]
    pub fn has_skeleton(&self) -> bool {
        self.skeleton.is_some()
    }

    #[allow(dead_code)]
    pub fn has_animations(&self) -> bool {
        !self.animation_channels.is_empty()
    }

    pub fn get_transform_matrix(&mut self) -> [f32; 16] {
        self.transform.get_matrix()
    }

    #[allow(dead_code)]
    pub fn is_renderable(&self) -> bool {
        self.mesh.is_valid()
    }

    pub fn set_animation_speed(&mut self, speed: f32) {
        self.animator.set_animation_speed(speed);
    }

    pub fn get_animation_speed(&self) -> f32 {
        self.animator.get_animation_speed()
    }

    pub fn render(&mut self, gl: &glow::Context, shader_program: &glow::Program) {
        // Update animation - need to handle borrowing carefully
        {
            let animation_channels = &self.animation_channels.clone();
            let skeleton = &mut self.skeleton;
            self.animator.update_with_data(animation_channels, skeleton);
        }

        // Bind material (texture)
        if let Some(material) = &self.material {
            material.bind(gl);
        }

        unsafe {
            // Get world transform matrix
            let world_txfm = self.get_transform_matrix();
            
            // Bind vertex array
            gl.bind_vertex_array(Some(self.mesh.vao));

            // Calculate bone matrices
            let mut bone_matrices = vec![mat4x4_identity(); 20];
            let mut inverse_bone_matrices = vec![mat4x4_identity(); 20];

            if let Some(skeleton) = &self.skeleton {
                for (i, &joint_id) in skeleton.joint_ids.iter().enumerate() {
                    if i >= 20 {
                        break;
                    }
                    inverse_bone_matrices[i] = skeleton.joint_inverse_mats[i];
                    bone_matrices[i] = node_world_txfm(&skeleton.nodes, joint_id as usize);
                }
            }

            // Upload world transform uniform
            gl.uniform_matrix_4_f32_slice(
                Some(&gl.get_uniform_location(*shader_program, "world_txfm").unwrap()),
                true,
                &world_txfm
            );

            // Upload bone matrices
            let flat_inverse: Vec<f32> = inverse_bone_matrices
                .iter()
                .flatten()
                .copied()
                .collect();
            let flat_bones: Vec<f32> = bone_matrices.iter().flatten().copied().collect();

            if let Some(loc) = gl.get_uniform_location(*shader_program, "inverse_bone_matrix") {
                gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_inverse);
            }
            if let Some(loc) = gl.get_uniform_location(*shader_program, "bone_matrix") {
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

impl Default for Object3D {
    fn default() -> Self {
        Self::new()
    }
}
