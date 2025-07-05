use glow::HasContext;

mod math {
    include!("engine/utils/math.rs");
}
use math::*;

mod object3d {
    include!("engine/components/Object3D.rs");
}
use object3d::*;

mod gltf_loader_utils {
    use super::*;
    include!("engine/utils/GLTFLoaderUtils.rs");
}

mod assets_manager {
    use super::*;
    include!("engine/managers/AssetsManager.rs");
}

use assets_manager::{ initialize, getObject3DCopy, Assets };

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn apply_animation(time_since_start: f32, object3d: &mut Object3D) {
    for channel in &object3d.animation_channels {
        if channel.times.is_empty() {
            continue;
        }

        let rel_time_since_start = time_since_start % channel.times[channel.num_timesteps - 1];

        let mut last_timestep = 0;
        for (i, &time) in channel.times.iter().enumerate().rev() {
            if rel_time_since_start >= time {
                last_timestep = i;
                break;
            }
        }

        let next_timestep = if last_timestep + 1 < channel.num_timesteps {
            last_timestep + 1
        } else {
            last_timestep
        };

        let components = channel.components();
        let last_data = &channel.data[last_timestep * components..(last_timestep + 1) * components];
        let next_data = &channel.data[next_timestep * components..(next_timestep + 1) * components];

        let last_time = channel.times[last_timestep];
        let next_time = channel.times[next_timestep];
        let t = if next_time != last_time {
            (rel_time_since_start - last_time) / (next_time - last_time)
        } else {
            0.0
        };

        let mut out = vec![0.0; components];
        for i in 0..components {
            out[i] = lerp(last_data[i], next_data[i], t);
        }

        if let Some(skeleton) = &mut object3d.skeleton {
            if let Some(node) = skeleton.nodes.get_mut(channel.target as usize) {
                match channel.animation_type {
                    AnimationType::Translation => {
                        node.translation[0] = out[0];
                        node.translation[1] = out[1];
                        node.translation[2] = out[2];
                    }
                    AnimationType::Rotation => {
                        node.rotation[0] = out[0];
                        node.rotation[1] = out[1];
                        node.rotation[2] = out[2];
                        node.rotation[3] = out[3];
                    }
                    AnimationType::Scale => {
                        node.scale[0] = out[0];
                        node.scale[1] = out[1];
                        node.scale[2] = out[2];
                    }
                }
            }
        }
    }
}

// === WORLD TRANSFORM CALCULATION ===

fn node_world_txfm(nodes: &[Node], idx: usize) -> Mat4x4 {
    let node = &nodes[idx];

    let mut node_txfm = mat4x4_scale(node.scale[0], node.scale[1], node.scale[2]);
    node_txfm = mat4x4_mul(mat4x4_from_quat(node.rotation), node_txfm);
    node_txfm = mat4x4_mul(
        mat4x4_translate(node.translation[0], node.translation[1], node.translation[2]),
        node_txfm
    );

    if node.parent != u32::MAX {
        node_txfm = mat4x4_mul(node_world_txfm(nodes, node.parent as usize), node_txfm);
    }

    node_txfm
}

// === PLATFORM-AGNOSTIC SHADER COMPILATION ===
// Platform-specific shader version handling is moved to build folders

fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: String
) -> Result<glow::Shader, String> {
    unsafe {
        let shader = gl.create_shader(shader_type)?;
        gl.shader_source(shader, &source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            return Err(format!("Shader compile error: {}", log));
        }
        Ok(shader)
    }
}

// === MAIN PROGRAM ===

pub struct Program {
    gl: glow::Context,
    shader_program: glow::Program,
    object3d1: Object3D, // Y-axis rotation character
    object3d2: Object3D, // X-axis rotation character
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        // Initialize assets with GL context
        if let Err(e) = initialize(&gl) {
            eprintln!("⚠️  Failed to initialize assets with GL context: {}", e);
        }

        let object3d1 = getObject3DCopy(Assets::TestingDoll);
        let object3d2 = getObject3DCopy(Assets::TestingDoll);

        unsafe {
            // Platform-specific shader source preparation is handled by platform code
            let vs_src = get_vertex_shader_source();
            let fs_src = get_fragment_shader_source();

            let vs = compile_shader(&gl, glow::VERTEX_SHADER, vs_src)?;
            let fs = compile_shader(&gl, glow::FRAGMENT_SHADER, fs_src)?;

            let program = gl.create_program()?;
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                return Err(format!("Program link error: {}", log));
            }

            gl.delete_shader(vs);
            gl.delete_shader(fs);
            gl.use_program(Some(program));
            gl.enable(glow::DEPTH_TEST);

            println!("✅ Program initialized successfully with assets from singleton manager");

            Ok(Self {
                gl,
                shader_program: program,
                object3d1,
                object3d2,
            })
        }
    }

    pub fn render(&mut self, width: u32, height: u32, time_since_start: f32) -> Result<(), String> {
        // Apply animation to both characters
        apply_animation(time_since_start, &mut self.object3d1);
        apply_animation(time_since_start, &mut self.object3d2);

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.use_program(Some(self.shader_program));

            let fov = (90.0_f32).to_radians();
            let aspect_ratio = (width as f32) / (height as f32);
            let viewport_txfm = mat4x4_perspective(fov, aspect_ratio, 0.1, 10.0);

            // Upload viewport transform (shared by both objects)
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.gl.get_uniform_location(self.shader_program, "viewport_txfm").unwrap()),
                true,
                &viewport_txfm
            );

            // Upload texture uniforms (shared by both objects)
            if
                let Some(loc) = self.gl.get_uniform_location(
                    self.shader_program,
                    "baseColorTexture"
                )
            {
                self.gl.uniform_1_i32(Some(&loc), 0); // Texture unit 0
            }
            if let Some(loc) = self.gl.get_uniform_location(self.shader_program, "hasTexture") {
                self.gl.uniform_1_i32(Some(&loc), 1); // Both objects have textures
            }

            let angle = time_since_start * 0.5;

            // === RENDER CHARACTER 1 (Y-axis rotation) ===
            {
                self.object3d1.transform.set_rotation_y(angle);
                self.object3d1.transform.set_translation(-2.0, -3.0, -5.0);

                let world_txfm = self.object3d1.get_transform_matrix();
                self.gl.bind_vertex_array(Some(self.object3d1.mesh.vao));

                // Calculate bone matrices for character 1
                let mut bone_matrices = vec![mat4x4_identity(); 20];
                let mut inverse_bone_matrices = vec![mat4x4_identity(); 20];

                if let Some(skeleton) = &self.object3d1.skeleton {
                    for (i, &joint_id) in skeleton.joint_ids.iter().enumerate() {
                        if i >= 20 {
                            break;
                        }
                        inverse_bone_matrices[i] = skeleton.joint_inverse_mats[i];
                        bone_matrices[i] = node_world_txfm(&skeleton.nodes, joint_id as usize);
                    }
                }

                // Bind texture for character 1
                if let Some(material) = &self.object3d1.material {
                    if let Some(texture) = material.base_color_texture {
                        self.gl.active_texture(glow::TEXTURE0);
                        self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                    }
                }

                // Upload uniforms for character 1
                self.gl.uniform_matrix_4_f32_slice(
                    Some(&self.gl.get_uniform_location(self.shader_program, "world_txfm").unwrap()),
                    true,
                    &world_txfm
                );

                let flat_inverse: Vec<f32> = inverse_bone_matrices
                    .iter()
                    .flatten()
                    .copied()
                    .collect();
                let flat_bones: Vec<f32> = bone_matrices.iter().flatten().copied().collect();

                if
                    let Some(loc) = self.gl.get_uniform_location(
                        self.shader_program,
                        "inverse_bone_matrix"
                    )
                {
                    self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_inverse);
                }
                if let Some(loc) = self.gl.get_uniform_location(self.shader_program, "bone_matrix") {
                    self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_bones);
                }

                // Render character 1
                self.gl.draw_elements(
                    glow::TRIANGLES,
                    self.object3d1.mesh.index_count as i32,
                    glow::UNSIGNED_SHORT,
                    0
                );
            }

            // === RENDER CHARACTER 2 (X-axis rotation) ===
            {
                self.object3d2.transform.set_rotation_x(angle);
                self.object3d2.transform.set_translation(2.0, -3.0, -5.0);

                let world_txfm = self.object3d2.get_transform_matrix();
                self.gl.bind_vertex_array(Some(self.object3d2.mesh.vao));

                // Calculate bone matrices for character 2
                let mut bone_matrices = vec![mat4x4_identity(); 20];
                let mut inverse_bone_matrices = vec![mat4x4_identity(); 20];

                if let Some(skeleton) = &self.object3d2.skeleton {
                    for (i, &joint_id) in skeleton.joint_ids.iter().enumerate() {
                        if i >= 20 {
                            break;
                        }
                        inverse_bone_matrices[i] = skeleton.joint_inverse_mats[i];
                        bone_matrices[i] = node_world_txfm(&skeleton.nodes, joint_id as usize);
                    }
                }

                // Bind texture for character 2
                if let Some(material) = &self.object3d2.material {
                    if let Some(texture) = material.base_color_texture {
                        self.gl.active_texture(glow::TEXTURE0);
                        self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                    }
                }

                // Upload uniforms for character 2
                self.gl.uniform_matrix_4_f32_slice(
                    Some(&self.gl.get_uniform_location(self.shader_program, "world_txfm").unwrap()),
                    true,
                    &world_txfm
                );

                let flat_inverse: Vec<f32> = inverse_bone_matrices
                    .iter()
                    .flatten()
                    .copied()
                    .collect();
                let flat_bones: Vec<f32> = bone_matrices.iter().flatten().copied().collect();

                if
                    let Some(loc) = self.gl.get_uniform_location(
                        self.shader_program,
                        "inverse_bone_matrix"
                    )
                {
                    self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_inverse);
                }
                if let Some(loc) = self.gl.get_uniform_location(self.shader_program, "bone_matrix") {
                    self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, &flat_bones);
                }

                // Render character 2
                self.gl.draw_elements(
                    glow::TRIANGLES,
                    self.object3d2.mesh.index_count as i32,
                    glow::UNSIGNED_SHORT,
                    0
                );
            }

            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }

    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_program(self.shader_program);
            self.gl.delete_vertex_array(self.object3d1.mesh.vao);
            self.gl.delete_vertex_array(self.object3d2.mesh.vao);
        }
    }
}

// Platform-specific functions to be implemented by platform code
extern "Rust" {
    fn get_vertex_shader_source() -> String;
    fn get_fragment_shader_source() -> String;
}
