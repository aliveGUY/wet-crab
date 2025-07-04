use glow::HasContext;
use web_time::Instant;
use gltf::buffer::Data;

mod math {
    include!("engine/utils/math.rs");
}
use math::*;

// Import Object3D and components
mod object3d {
    include!("engine/components/Object3D.rs");
}
use object3d::*;

// === DATA STRUCTURES ===

struct Model {
    object3d: Object3D,
    gl_vao: glow::VertexArray, // Keep OpenGL-specific data separate
}

// === GLTF LOADING ===

fn extract_buffer_data<T: bytemuck::Pod>(
    buffers: &[Data],
    accessor: &gltf::Accessor
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let view = accessor.view().ok_or("Missing buffer view")?;
    let buffer = &buffers[view.buffer().index()];
    let start = view.offset() + accessor.offset();
    let end = start + accessor.count() * accessor.size();

    if end > buffer.len() {
        return Err("Buffer overflow".into());
    }

    let slice = &buffer[start..end];
    let typed_slice = bytemuck::cast_slice(slice);
    Ok(typed_slice.to_vec())
}

fn load_model(gl: &glow::Context) -> Result<Model, Box<dyn std::error::Error>> {
    println!("🔄 Loading embedded GLTF data...");

    let gltf = gltf::Gltf::from_slice(include_str!("assets/meshes/guy.gltf").as_bytes())?;
    let buffers = vec![gltf::buffer::Data(include_bytes!("assets/meshes/guy.bin").to_vec())];

    let primitive = gltf
        .meshes()
        .next()
        .ok_or("No mesh found")?
        .primitives()
        .next()
        .ok_or("No primitive found")?;

    macro_rules! extract {
        ($sem:expr, $ty:ty) => {
            extract_buffer_data::<$ty>(&buffers, &primitive.get(&$sem).ok_or(concat!("Missing ", stringify!($sem)))?)?
        };
    }

    let positions: Vec<f32> = extract!(gltf::Semantic::Positions, f32);
    let normals: Vec<f32> = extract!(gltf::Semantic::Normals, f32);
    let joints: Vec<u8> = extract!(gltf::Semantic::Joints(0), u8);
    let weights: Vec<f32> = extract!(gltf::Semantic::Weights(0), f32);
    let indices: Vec<u16> = extract_buffer_data(
        &buffers,
        &primitive.indices().ok_or("No indices")?
    )?;

    unsafe {
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));

        let setup_attrib = |loc, data: &[u8], size, ty, stride, int| {
            let buf = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(loc);
            if int {
                gl.vertex_attrib_pointer_i32(loc, size, ty, stride, 0);
            } else {
                gl.vertex_attrib_pointer_f32(loc, size, ty, false, stride, 0);
            }
        };

        setup_attrib(1, bytemuck::cast_slice(&positions), 3, glow::FLOAT, 12, false);
        setup_attrib(0, bytemuck::cast_slice(&normals), 3, glow::FLOAT, 12, false);
        setup_attrib(2, &joints, 4, glow::UNSIGNED_BYTE, 4, true);
        setup_attrib(3, bytemuck::cast_slice(&weights), 4, glow::FLOAT, 16, false);

        let ebo = gl.create_buffer()?;
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&indices),
            glow::STATIC_DRAW
        );

        gl.bind_vertex_array(None);

        let mut node_parents = vec![u32::MAX; gltf.nodes().len()];
        for node in gltf.nodes() {
            for child in node.children() {
                node_parents[child.index()] = node.index() as u32;
            }
        }

        let nodes = gltf
            .nodes()
            .map(|n| {
                let (t, r, s) = n.transform().decomposed();
                Node { translation: t, rotation: r, scale: s, parent: node_parents[n.index()] }
            })
            .collect::<Vec<_>>();

        let animation_channels: Vec<AnimationChannel> = gltf
            .animations()
            .next()
            .map(|anim| {
                anim.channels()
                    .filter_map(|chan| {
                        let anim_type = match chan.target().property() {
                            gltf::animation::Property::Translation => AnimationType::Translation,
                            gltf::animation::Property::Rotation => AnimationType::Rotation,
                            gltf::animation::Property::Scale => AnimationType::Scale,
                            _ => {
                                return None;
                            }
                        };

                        let times = extract_buffer_data::<f32>(
                            &buffers,
                            &chan.sampler().input()
                        ).ok()?;
                        let data = extract_buffer_data::<f32>(
                            &buffers,
                            &chan.sampler().output()
                        ).ok()?;

                        Some(AnimationChannel {
                            target: chan.target().node().index() as u32,
                            animation_type: anim_type,
                            num_timesteps: times.len(),
                            times,
                            data,
                        })
                    })
                    .collect::<Vec<AnimationChannel>>()
            })
            .unwrap_or_default();

        let (joint_ids, joint_inverse_mats) = if let Some(skin) = gltf.skins().next() {
            let ids = skin
                .joints()
                .map(|j| j.index() as u32)
                .collect();
            let mut inv_mats = Vec::new();
            if let Some(ibm) = skin.inverse_bind_matrices() {
                let data: Vec<f32> = extract_buffer_data(&buffers, &ibm)?;
                inv_mats = data
                    .chunks(16)
                    .map(|m| {
                        let mut mat = [0.0; 16];
                        mat.copy_from_slice(m);
                        mat4x4_transpose(mat)
                    })
                    .collect();
            }
            (ids, inv_mats)
        } else {
            (Vec::new(), Vec::new())
        };

        // Create Object3D components
        let mesh_component = Mesh::with_buffers(
            0, // For WebGL, we'll use 0 as placeholder since VAO is handled separately
            0, // We don't track individual buffers in this simple case
            0,
            indices.len(),
            positions.len() / 3
        );

        let skeleton = if !nodes.is_empty() {
            Some(Skeleton {
                nodes,
                joint_ids,
                joint_inverse_mats,
            })
        } else {
            None
        };

        let mut object3d = Object3D::with_mesh(mesh_component);
        
        if let Some(skel) = skeleton {
            object3d.set_skeleton(skel);
        }
        
        object3d.set_animation_channels(animation_channels);

        println!(
            "✅ Model loaded: {} nodes, {} animations, {} joints",
            object3d.skeleton.as_ref().map_or(0, |s| s.nodes.len()),
            object3d.animation_channels.len(),
            object3d.skeleton.as_ref().map_or(0, |s| s.joint_ids.len())
        );

        Ok(Model {
            object3d,
            gl_vao: vao,
        })
    }
}

// === ANIMATION ===

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn apply_animation(time_since_start: f32, model: &mut Model) {
    for channel in &model.object3d.animation_channels {
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

        if let Some(skeleton) = &mut model.object3d.skeleton {
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

// === SHADER COMPILATION ===

fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    mut source: String
) -> Result<glow::Shader, String> {
    unsafe {
        let is_web = cfg!(target_arch = "wasm32");
        if is_web {
            source = source.replace("#VERSION", "#version 300 es\nprecision mediump float;");
        } else {
            source = source.replace("#VERSION", "#version 460 core");
        }

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
    model: Model,
    start_time: Instant,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        let model = load_model(&gl).map_err(|e| format!("Failed to load model: {}", e))?;

        unsafe {
            let vs_src = include_str!("assets/shaders/vertex.glsl");
            let fs_src = include_str!("assets/shaders/fragment.glsl");

            let vs = compile_shader(&gl, glow::VERTEX_SHADER, vs_src.to_string())?;
            let fs = compile_shader(&gl, glow::FRAGMENT_SHADER, fs_src.to_string())?;

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

            Ok(Self {
                gl,
                shader_program: program,
                model,
                start_time: Instant::now(),
            })
        }
    }

    pub fn render(&mut self, width: u32, height: u32, _delta_time: f32) -> Result<(), String> {
        let time_since_start = self.start_time.elapsed().as_secs_f32();

        // Apply animation
        apply_animation(time_since_start, &mut self.model);

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.use_program(Some(self.shader_program));
            self.gl.bind_vertex_array(Some(self.model.gl_vao));

            let angle = time_since_start * 0.5;
            let mut world_txfm = mat4x4_translate(0.0, 0.0, 0.0);
            world_txfm = mat4x4_mul(world_txfm, mat4x4_rot_y(angle));
            world_txfm = mat4x4_mul(mat4x4_translate(0.0, -3.0, -5.0), world_txfm);

            let viewport_txfm = mat4x4_perspective(0.1, 10.0);

            // Calculate bone matrices
            let mut bone_matrices = vec![mat4x4_identity(); 20];
            let mut inverse_bone_matrices = vec![mat4x4_identity(); 20];

            if let Some(skeleton) = &self.model.object3d.skeleton {
                for (i, &joint_id) in skeleton.joint_ids.iter().enumerate() {
                    if i >= 20 {
                        break;
                    }
                    inverse_bone_matrices[i] = skeleton.joint_inverse_mats[i];
                    bone_matrices[i] = node_world_txfm(&skeleton.nodes, joint_id as usize);
                }
            }

            // Upload uniforms
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.gl.get_uniform_location(self.shader_program, "world_txfm").unwrap()),
                true,
                &world_txfm
            );
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.gl.get_uniform_location(self.shader_program, "viewport_txfm").unwrap()),
                true,
                &viewport_txfm
            );

            // Upload bone matrices
            let flat_inverse: Vec<f32> = inverse_bone_matrices.iter().flatten().copied().collect();
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

            // Render
            self.gl.draw_elements(
                glow::TRIANGLES,
                self.model.object3d.mesh.num_indices as i32,
                glow::UNSIGNED_SHORT,
                0
            );
            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }

    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_program(self.shader_program);
            self.gl.delete_vertex_array(self.model.gl_vao);
        }
    }
}
