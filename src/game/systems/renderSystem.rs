use glow::HasContext;

// Import types and functions from parent scope
use crate::index::Transform;
use crate::index::StaticObject3D;
use crate::index::AnimatedObject3D;
use crate::index::animated_object3d::AnimationType;
use crate::index::System;
use crate::index::mat4x4_perspective;
use crate::index::mat4x4_mul;
use crate::index::mat4x4_identity;
use crate::index::node_world_txfm;
use crate::index::{PLAYER_ENTITY_ID, Camera};

#[derive(Debug)]
pub struct RenderSystem;

impl RenderSystem {
    pub fn update(gl: &glow::Context, width: u32, height: u32) {
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
            gl.clear_color(0.1, 0.1, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        // Get player ID and camera in one scope to avoid lifetime issues
        let view_matrix = {
            let player_id_guard = PLAYER_ENTITY_ID.read().unwrap();
            let player_id = match player_id_guard.as_ref() {
                Some(id) => id,
                None => return,
            };

            // Get camera - early return if None  
            let camera = match get_query_by_id!(player_id, (Camera)) {
                Some(cam) => cam,
                None => return,
            };

            // Get view matrix while we have the camera reference
            camera.get_view_matrix()
        };
        let fov = (90.0_f32).to_radians();
        let aspect_ratio = (width as f32) / (height as f32);
        let projection_matrix = mat4x4_perspective(fov, aspect_ratio, 0.1, 10.0);
        let view_proj = mat4x4_mul(projection_matrix, view_matrix);

        Self::render_animated_objects(gl, &view_proj);
        Self::render_static_objects(gl, &view_proj);

        unsafe {
            gl.bind_vertex_array(None);
        }
    }

    fn render_animated_objects(gl: &glow::Context, view_proj: &[f32; 16]) {
        query!((Transform, AnimatedObject3D), |_id, transform, animated_object| {
            Self::setup_viewport_uniform(gl, view_proj, animated_object.material.shader_program);
            
            // Use shader directly from material
            unsafe {
                gl.use_program(Some(animated_object.material.shader_program));
            }

            // Update animation
            {
                // Convert to the types expected by the animator
                let animation_channels: Vec<crate::index::animated_object3d::AnimationChannel> =
                    animated_object.animation_channels
                        .iter()
                        .map(|ch| crate::index::animated_object3d::AnimationChannel {
                            target: ch.target,
                            animation_type: match ch.animation_type {
                                AnimationType::Translation =>
                                    crate::index::animated_object3d::AnimationType::Translation,
                                AnimationType::Rotation =>
                                    crate::index::animated_object3d::AnimationType::Rotation,
                                AnimationType::Scale =>
                                    crate::index::animated_object3d::AnimationType::Scale,
                            },
                            num_timesteps: ch.num_timesteps,
                            times: ch.times.clone(),
                            data: ch.data.clone(),
                        })
                        .collect();

                // Convert skeleton to the expected type
                let mut skeleton_converted = crate::index::animated_object3d::Skeleton {
                    nodes: animated_object.skeleton.nodes
                        .iter()
                        .map(|n| crate::index::animated_object3d::Node {
                            translation: n.translation,
                            rotation: n.rotation,
                            scale: n.scale,
                            parent: n.parent,
                        })
                        .collect(),
                    joint_ids: animated_object.skeleton.joint_ids.clone(),
                    joint_inverse_mats: animated_object.skeleton.joint_inverse_mats.clone(),
                };

                animated_object.animator.update_with_data(&animation_channels[..], &mut skeleton_converted);

                // Copy back the updated nodes
                for (i, node) in skeleton_converted.nodes.iter().enumerate() {
                    if i < animated_object.skeleton.nodes.len() {
                        animated_object.skeleton.nodes[i].translation = node.translation;
                        animated_object.skeleton.nodes[i].rotation = node.rotation;
                        animated_object.skeleton.nodes[i].scale = node.scale;
                    }
                }
            }

            // Bind material (texture)
            animated_object.material.bind(gl);

            unsafe {
                // Get world transform matrix
                let world_txfm = transform.get_matrix();

                // Bind vertex array
                gl.bind_vertex_array(Some(animated_object.mesh.vao));

                // Calculate bone matrices
                let mut bone_matrices = vec![mat4x4_identity(); 20];
                let mut inverse_bone_matrices = vec![mat4x4_identity(); 20];

                for (i, &joint_id) in animated_object.skeleton.joint_ids.iter().enumerate() {
                    if i >= 20 {
                        break;
                    }
                    inverse_bone_matrices[i] = animated_object.skeleton.joint_inverse_mats[i];
                    // Convert nodes to the expected type for the math function
                    let nodes_converted: Vec<crate::index::animated_object3d::Node> =
                        animated_object.skeleton.nodes
                            .iter()
                            .map(|n| crate::index::animated_object3d::Node {
                                translation: n.translation,
                                rotation: n.rotation,
                                scale: n.scale,
                                parent: n.parent,
                            })
                            .collect();
                    bone_matrices[i] = node_world_txfm(&nodes_converted[..], joint_id as usize);
                }

                // Upload world transform uniform
                if let Some(loc) = gl.get_uniform_location(
                    animated_object.material.shader_program,
                    "world_txfm"
                ) {
                    gl.uniform_matrix_4_f32_slice(Some(&loc), true, world_txfm);
                }

                // Upload bone matrices
                let flat_inverse: Vec<f32> = inverse_bone_matrices
                    .iter()
                    .flatten()
                    .copied()
                    .collect();
                let flat_bones: Vec<f32> = bone_matrices.iter().flatten().copied().collect();

                if let Some(loc) = gl.get_uniform_location(
                    animated_object.material.shader_program,
                    "inverse_bone_matrix"
                ) {
                    gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_inverse);
                }
                if let Some(loc) = gl.get_uniform_location(
                    animated_object.material.shader_program,
                    "bone_matrix"
                ) {
                    gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_bones);
                }

                // Draw the mesh
                gl.draw_elements(
                    glow::TRIANGLES,
                    animated_object.mesh.index_count as i32,
                    glow::UNSIGNED_SHORT,
                    0
                );
            }
        });
    }

    fn render_static_objects(gl: &glow::Context, view_proj: &[f32; 16]) {
        query!((Transform, StaticObject3D), |_id, transform, static_object| {
            Self::setup_viewport_uniform(gl, view_proj, static_object.material.shader_program);
            
            // Use shader directly from material
            unsafe {
                gl.use_program(Some(static_object.material.shader_program));
            }

            // Bind material (texture)
            static_object.material.bind(gl);

            unsafe {
                let world_txfm = transform.get_matrix();

                // Bind vertex array
                gl.bind_vertex_array(Some(static_object.mesh.vao));

                // Upload world transform uniform
                if let Some(loc) = gl.get_uniform_location(static_object.material.shader_program, "world_txfm") {
                    gl.uniform_matrix_4_f32_slice(Some(&loc), true, world_txfm);
                }

                // Draw the mesh (simple static rendering)
                gl.draw_elements(
                    glow::TRIANGLES,
                    static_object.mesh.index_count as i32,
                    glow::UNSIGNED_SHORT,
                    0
                );
            }
        });
    }

    fn setup_viewport_uniform(gl: &glow::Context, viewport_txfm: &[f32; 16], shader_program: glow::Program) {
        unsafe {
            gl.use_program(Some(shader_program));

            if let Some(loc) = gl.get_uniform_location(shader_program, "viewport_txfm") {
                gl.uniform_matrix_4_f32_slice(Some(&loc), true, viewport_txfm);
            }

            if let Some(loc) = gl.get_uniform_location(shader_program, "baseColorTexture") {
                gl.uniform_1_i32(Some(&loc), 0);
            }
            if let Some(loc) = gl.get_uniform_location(shader_program, "hasTexture") {
                gl.uniform_1_i32(Some(&loc), 1);
            }
        }
    }
}

impl System for RenderSystem {
    fn update(&self) {
        // This will be called by the system manager, but we need gl context
        // The actual rendering will be called from index.rs with proper parameters
        panic!("RenderSystem::update() should not be called directly. Use RenderSystem::update(gl, width, height) instead.");
    }
}
