use glow::HasContext;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

// === MATH HELPERS (Row-major like the C implementation) ===

type Mat4x4 = [f32; 16];

fn mat4x4_identity() -> Mat4x4 {
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_translate(x: f32, y: f32, z: f32) -> Mat4x4 {
    [1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_rot_y(angle: f32) -> Mat4x4 {
    let c = angle.cos();
    let s = angle.sin();
    [c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_scale(x: f32, y: f32, z: f32) -> Mat4x4 {
    [x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_from_quat(quat: [f32; 4]) -> Mat4x4 {
    let [x, y, z, w] = quat;
    let x2 = x * x;
    let y2 = y * y;
    let z2 = z * z;
    let w2 = w * w;

    let xy = 2.0 * x * y;
    let xz = 2.0 * x * z;
    let xw = 2.0 * x * w;
    let yz = 2.0 * y * z;
    let yw = 2.0 * y * w;
    let zw = 2.0 * z * w;

    [
        w2 + x2 - y2 - z2,
        xy - zw,
        xz + yw,
        0.0,
        xy + zw,
        w2 - x2 + y2 - z2,
        yz - xw,
        0.0,
        xz - yw,
        yz + xw,
        w2 - x2 - y2 + z2,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ]
}

fn mat4x4_transpose(matrix: Mat4x4) -> Mat4x4 {
    let mut ret = [0.0; 16];
    for i in 0..16 {
        let row = i / 4;
        let col = i % 4;
        ret[col * 4 + row] = matrix[row * 4 + col];
    }
    ret
}

fn vec4_dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

fn mat4x4_row(mat: &Mat4x4, row: usize) -> [f32; 4] {
    let start_idx = row * 4;
    [mat[start_idx], mat[start_idx + 1], mat[start_idx + 2], mat[start_idx + 3]]
}

fn mat4x4_col(mat: &Mat4x4, col: usize) -> [f32; 4] {
    [mat[col], mat[4 + col], mat[8 + col], mat[12 + col]]
}

fn mat4x4_mul(a: Mat4x4, b: Mat4x4) -> Mat4x4 {
    let mut ret = [0.0; 16];
    for i in 0..16 {
        let row = i / 4;
        let col = i % 4;
        let a_row = mat4x4_row(&a, row);
        let b_col = mat4x4_col(&b, col);
        ret[i] = vec4_dot(a_row, b_col);
    }
    ret
}

fn mat4x4_perspective(n: f32, f: f32) -> Mat4x4 {
    let a = -f / (f - n);
    let b = (-f * n) / (f - n);
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, a, b, 0.0, 0.0, -1.0, 0.0]
}

// === DATA STRUCTURES ===

#[derive(Debug, Clone)]
struct Node {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
    parent: u32,
}

#[derive(Debug, Clone)]
enum AnimationType {
    Translation = 0,
    Rotation = 1,
    Scale = 2,
}

#[derive(Debug, Clone)]
struct AnimationChannel {
    target: u32,
    animation_type: AnimationType,
    num_timesteps: usize,
    times: Vec<f32>,
    data: Vec<f32>,
}

impl AnimationChannel {
    fn components(&self) -> usize {
        match self.animation_type {
            AnimationType::Translation | AnimationType::Scale => 3,
            AnimationType::Rotation => 4,
        }
    }
}

struct Model {
    vao: glow::VertexArray,
    num_indices: usize,
    nodes: Vec<Node>,
    animation_channels: Vec<AnimationChannel>,
    joint_ids: Vec<u32>,
    joint_inverse_mats: Vec<Mat4x4>,
}

// === FILE LOADING ===

fn load_file(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn load_model(gl: &glow::Context) -> Result<Model, Box<dyn std::error::Error>> {
    println!("🔄 Loading binary mesh data...");

    // Load mesh data
    let positions_data = load_file("src/assets/meshes/positions.bin")?;
    let normals_data = load_file("src/assets/meshes/normals.bin")?;
    let indices_data = load_file("src/assets/meshes/indices.bin")?;
    let vert_joints_data = load_file("src/assets/meshes/vert_joints.bin")?;
    let vert_weights_data = load_file("src/assets/meshes/vert_weights.bin")?;
    let nodes_data = load_file("src/assets/meshes/nodes.bin")?;
    let joint_info_data = load_file("src/assets/meshes/joint_info.bin")?;

    unsafe {
        // Create VAO
        let vertex_array = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vertex_array));

        // Position buffer (location 1)
        let position_buffer = gl.create_buffer()?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(position_buffer));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &positions_data, glow::STATIC_DRAW);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 12, 0);

        // Normal buffer (location 0)
        let normals_buffer = gl.create_buffer()?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(normals_buffer));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &normals_data, glow::STATIC_DRAW);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);

        // Joints buffer (location 2)
        let joints_buffer = gl.create_buffer()?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(joints_buffer));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vert_joints_data, glow::STATIC_DRAW);
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_i32(2, 4, glow::UNSIGNED_BYTE, 4, 0);

        // Weights buffer (location 3)
        let weights_buffer = gl.create_buffer()?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(weights_buffer));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vert_weights_data, glow::STATIC_DRAW);
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_f32(3, 4, glow::FLOAT, false, 16, 0);

        // Index buffer
        let ebo = gl.create_buffer()?;
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &indices_data, glow::STATIC_DRAW);

        gl.bind_vertex_array(None);

        // Parse nodes
        println!("🔄 Loading node hierarchy...");
        let num_nodes = nodes_data.len() / 44; // 11 * 4 bytes per node
        let mut nodes = Vec::new();

        for i in 0..num_nodes {
            let offset = i * 44;
            let translation = [
                f32::from_le_bytes([
                    nodes_data[offset],
                    nodes_data[offset + 1],
                    nodes_data[offset + 2],
                    nodes_data[offset + 3],
                ]),
                f32::from_le_bytes([
                    nodes_data[offset + 4],
                    nodes_data[offset + 5],
                    nodes_data[offset + 6],
                    nodes_data[offset + 7],
                ]),
                f32::from_le_bytes([
                    nodes_data[offset + 8],
                    nodes_data[offset + 9],
                    nodes_data[offset + 10],
                    nodes_data[offset + 11],
                ]),
            ];
            let rotation = [
                f32::from_le_bytes([
                    nodes_data[offset + 12],
                    nodes_data[offset + 13],
                    nodes_data[offset + 14],
                    nodes_data[offset + 15],
                ]),
                f32::from_le_bytes([
                    nodes_data[offset + 16],
                    nodes_data[offset + 17],
                    nodes_data[offset + 18],
                    nodes_data[offset + 19],
                ]),
                f32::from_le_bytes([
                    nodes_data[offset + 20],
                    nodes_data[offset + 21],
                    nodes_data[offset + 22],
                    nodes_data[offset + 23],
                ]),
                f32::from_le_bytes([
                    nodes_data[offset + 24],
                    nodes_data[offset + 25],
                    nodes_data[offset + 26],
                    nodes_data[offset + 27],
                ]),
            ];
            let scale = [
                f32::from_le_bytes([
                    nodes_data[offset + 28],
                    nodes_data[offset + 29],
                    nodes_data[offset + 30],
                    nodes_data[offset + 31],
                ]),
                f32::from_le_bytes([
                    nodes_data[offset + 32],
                    nodes_data[offset + 33],
                    nodes_data[offset + 34],
                    nodes_data[offset + 35],
                ]),
                f32::from_le_bytes([
                    nodes_data[offset + 36],
                    nodes_data[offset + 37],
                    nodes_data[offset + 38],
                    nodes_data[offset + 39],
                ]),
            ];
            let parent = u32::from_le_bytes([
                nodes_data[offset + 40],
                nodes_data[offset + 41],
                nodes_data[offset + 42],
                nodes_data[offset + 43],
            ]);

            nodes.push(Node { translation, rotation, scale, parent });
        }

        // Load animations
        println!("🔄 Loading animation data...");
        let mut animation_channels = Vec::new();
        for i in 0..30 {
            // We know there are 30 animation files
            let path = format!("src/assets/meshes/animations_{}.bin", i);
            if let Ok(animation_data) = load_file(&path) {
                let target_node = u32::from_le_bytes([
                    animation_data[0],
                    animation_data[1],
                    animation_data[2],
                    animation_data[3],
                ]);
                let target_path = u32::from_le_bytes([
                    animation_data[4],
                    animation_data[5],
                    animation_data[6],
                    animation_data[7],
                ]);
                let num_timesteps = u32::from_le_bytes([
                    animation_data[8],
                    animation_data[9],
                    animation_data[10],
                    animation_data[11],
                ]) as usize;

                let animation_type = match target_path {
                    0 => AnimationType::Translation,
                    1 => AnimationType::Rotation,
                    2 => AnimationType::Scale,
                    _ => {
                        continue;
                    }
                };

                let mut cursor = 12;
                let mut times = Vec::new();
                for _ in 0..num_timesteps {
                    let time = f32::from_le_bytes([
                        animation_data[cursor],
                        animation_data[cursor + 1],
                        animation_data[cursor + 2],
                        animation_data[cursor + 3],
                    ]);
                    times.push(time);
                    cursor += 4;
                }

                let components = match animation_type {
                    AnimationType::Translation | AnimationType::Scale => 3,
                    AnimationType::Rotation => 4,
                };

                let mut data = Vec::new();
                for _ in 0..num_timesteps {
                    for _ in 0..components {
                        let value = f32::from_le_bytes([
                            animation_data[cursor],
                            animation_data[cursor + 1],
                            animation_data[cursor + 2],
                            animation_data[cursor + 3],
                        ]);
                        data.push(value);
                        cursor += 4;
                    }
                }

                animation_channels.push(AnimationChannel {
                    target: target_node,
                    animation_type,
                    num_timesteps,
                    times,
                    data,
                });
            }
        }

        // Load joint info
        println!("🔄 Loading joint and skinning data...");
        let num_joints = u32::from_le_bytes([
            joint_info_data[0],
            joint_info_data[1],
            joint_info_data[2],
            joint_info_data[3],
        ]) as usize;
        let mut joint_ids = Vec::new();
        let mut cursor = 4;

        for _ in 0..num_joints {
            let joint_id = u32::from_le_bytes([
                joint_info_data[cursor],
                joint_info_data[cursor + 1],
                joint_info_data[cursor + 2],
                joint_info_data[cursor + 3],
            ]);
            joint_ids.push(joint_id);
            cursor += 4;
        }

        let mut joint_inverse_mats = Vec::new();
        for _ in 0..num_joints {
            let mut matrix = [0.0f32; 16];
            for j in 0..16 {
                matrix[j] = f32::from_le_bytes([
                    joint_info_data[cursor],
                    joint_info_data[cursor + 1],
                    joint_info_data[cursor + 2],
                    joint_info_data[cursor + 3],
                ]);
                cursor += 4;
            }
            // Transpose like the C implementation
            joint_inverse_mats.push(mat4x4_transpose(matrix));
        }

        println!(
            "✅ Model loaded: {} nodes, {} animations, {} joints",
            nodes.len(),
            animation_channels.len(),
            num_joints
        );

        Ok(Model {
            vao: vertex_array,
            num_indices: indices_data.len() / 2, // u16 indices
            nodes,
            animation_channels,
            joint_ids,
            joint_inverse_mats,
        })
    }
}

// === ANIMATION ===

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn apply_animation(time_since_start: f32, model: &mut Model) {
    for channel in &model.animation_channels {
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

        if let Some(node) = model.nodes.get_mut(channel.target as usize) {
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
            self.gl.bind_vertex_array(Some(self.model.vao));

            let angle = time_since_start * 0.5;
            let mut world_txfm = mat4x4_translate(0.0, 0.0, 0.0);
            world_txfm = mat4x4_mul(world_txfm, mat4x4_rot_y(angle));
            world_txfm = mat4x4_mul(mat4x4_translate(0.0, -3.0, -5.0), world_txfm);

            let viewport_txfm = mat4x4_perspective(0.1, 10.0);

            // Calculate bone matrices
            let mut bone_matrices = vec![mat4x4_identity(); 20];
            let mut inverse_bone_matrices = vec![mat4x4_identity(); 20];

            for (i, &joint_id) in self.model.joint_ids.iter().enumerate() {
                if i >= 20 {
                    break;
                }
                inverse_bone_matrices[i] = self.model.joint_inverse_mats[i];
                bone_matrices[i] = node_world_txfm(&self.model.nodes, joint_id as usize);
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
                self.model.num_indices as i32,
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
            self.gl.delete_vertex_array(self.model.vao);
        }
    }
}
