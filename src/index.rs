use glow::HasContext;
use std::collections::{ HashMap, HashSet };

// === MATH HELPERS ===

type mat4x4 = [f32; 16];

fn mat4x4_rot_y(angle_radians: f32) -> mat4x4 {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();
    [cos, 0.0, -sin, 0.0, 0.0, 1.0, 0.0, 0.0, sin, 0.0, cos, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_translate(x: f32, y: f32, z: f32) -> mat4x4 {
    [1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_mul(a: mat4x4, b: mat4x4) -> mat4x4 {
    let mut result = [0.0; 16];
    for i in 0..16 {
        let row = i / 4;
        let col = i % 4;
        for k in 0..4 {
            result[i] += a[row * 4 + k] * b[k * 4 + col];
        }
    }
    result
}

fn mat4x4_perspective(near: f32, far: f32) -> mat4x4 {
    let a = -far / (far - near);
    let b = (-far * near) / (far - near);
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, a, b, 0.0, 0.0, -1.0, 0.0]
}

fn mat4x4_from_transform(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> mat4x4 {
    // Convert quaternion to rotation matrix
    let [x, y, z, w] = rotation;
    let x2 = x + x;
    let y2 = y + y;
    let z2 = z + z;
    let xx = x * x2;
    let xy = x * y2;
    let xz = x * z2;
    let yy = y * y2;
    let yz = y * z2;
    let zz = z * z2;
    let wx = w * x2;
    let wy = w * y2;
    let wz = w * z2;

    // Create transformation matrix with scale, rotation, and translation
    [
        scale[0] * (1.0 - (yy + zz)),
        scale[0] * (xy - wz),
        scale[0] * (xz + wy),
        translation[0],
        scale[1] * (xy + wz),
        scale[1] * (1.0 - (xx + zz)),
        scale[1] * (yz - wx),
        translation[1],
        scale[2] * (xz - wy),
        scale[2] * (yz + wx),
        scale[2] * (1.0 - (xx + yy)),
        translation[2],
        0.0,
        0.0,
        0.0,
        1.0,
    ]
}

fn mat4x4_identity() -> mat4x4 {
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_extract_translation(matrix: &mat4x4) -> [f32; 3] {
    [matrix[3], matrix[7], matrix[11]]
}

fn mat4x4_extract_rotation_direction(matrix: &mat4x4) -> [f32; 3] {
    // Extract the forward direction (Z-axis) from the rotation matrix
    // This gives us the direction the bone is pointing
    [matrix[2], matrix[6], matrix[10]]
}

fn normalize_vector(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 0.001 {
        [v[0] / length, v[1] / length, v[2] / length]
    } else {
        [0.0, 1.0, 0.0] // Default to up direction
    }
}

fn mat4x4_transform_direction(matrix: &mat4x4, direction: [f32; 3]) -> [f32; 3] {
    // Transform direction vector (w=0) by the rotation part of the matrix
    [
        matrix[0] * direction[0] + matrix[1] * direction[1] + matrix[2] * direction[2],
        matrix[4] * direction[0] + matrix[5] * direction[1] + matrix[6] * direction[2],
        matrix[8] * direction[0] + matrix[9] * direction[1] + matrix[10] * direction[2]
    ]
}

// === SKELETON DATA STRUCTURES ===

#[derive(Debug, Clone)]
pub struct Bone {
    pub start_pos: [f32; 3], // World position of bone start
    pub end_pos: [f32; 3], // World position of bone end
    pub node_index: usize, // Reference to GLTF node
    pub parent_index: Option<usize>, // Parent bone index
}

#[derive(Debug)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub root_bones: Vec<usize>, // Indices of root bones
}

// === MESH PARSING ===

pub fn parse_mesh() -> Result<
    (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>),
    Box<dyn std::error::Error>
> {
    let gltf_data = include_bytes!("assets/meshes/guy.gltf");
    let buffer_data = include_bytes!("assets/meshes/guy.bin");

    let gltf = gltf::Gltf::from_slice(gltf_data)?;
    let document = gltf.document;

    let mesh = document.meshes().next().ok_or("No mesh found")?;
    let primitive = mesh.primitives().next().ok_or("No primitive in mesh")?;

    let buffers = vec![gltf::buffer::Data(buffer_data.to_vec())];

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or("No position attribute")?
        .collect();
    let normals: Vec<[f32; 3]> = reader.read_normals().ok_or("No normal attribute")?.collect();
    let indices: Vec<u32> = reader.read_indices().ok_or("No indices found")?.into_u32().collect();

    Ok((positions, normals, indices))
}

pub fn parse_nodes() -> Result<Skeleton, Box<dyn std::error::Error>> {
    let gltf_data = include_bytes!("assets/meshes/guy.gltf");

    let gltf = gltf::Gltf::from_slice(gltf_data)?;
    let document = gltf.document;

    // Get skeleton joints from skin definition
    let joint_indices = get_skeleton_joints(&document)?;

    println!("🦴 Found skeleton with {} joints: {:?}", joint_indices.len(), joint_indices);

    // Build parent-child relationships for ALL nodes
    let mut parents: HashMap<usize, usize> = HashMap::new();

    for scene in document.scenes() {
        for root_node in scene.nodes() {
            build_parent_map(root_node, None, &mut parents);
        }
    }

    // Calculate world transforms for all nodes
    let world_transforms = calculate_world_transforms(&document, &parents);

    // Generate bones ONLY from skeleton joints
    let bones = generate_skeleton_bones(&document, &world_transforms, &parents, &joint_indices);

    // Find root bones (bones without parents in the skeleton)
    let root_bones: Vec<usize> = bones
        .iter()
        .enumerate()
        .filter_map(|(i, bone)| if bone.parent_index.is_none() { Some(i) } else { None })
        .collect();

    println!("🦴 Generated {} skeleton bones from {} joints", bones.len(), joint_indices.len());
    for (i, bone) in bones.iter().enumerate() {
        let node = document.nodes().nth(bone.node_index).unwrap();
        let node_name = node.name().unwrap_or("unnamed");
        println!(
            "  Bone {}: {} (Node {}) -> [{:.2}, {:.2}, {:.2}] to [{:.2}, {:.2}, {:.2}]",
            i,
            node_name,
            bone.node_index,
            bone.start_pos[0],
            bone.start_pos[1],
            bone.start_pos[2],
            bone.end_pos[0],
            bone.end_pos[1],
            bone.end_pos[2]
        );
    }

    Ok(Skeleton { bones, root_bones })
}

fn build_parent_map(
    node: gltf::Node,
    parent_index: Option<usize>,
    parents: &mut HashMap<usize, usize>
) {
    if let Some(parent) = parent_index {
        parents.insert(node.index(), parent);
    }

    for child in node.children() {
        build_parent_map(child, Some(node.index()), parents);
    }
}

fn calculate_world_transforms(
    document: &gltf::Document,
    parents: &HashMap<usize, usize>
) -> HashMap<usize, mat4x4> {
    let mut world_transforms: HashMap<usize, mat4x4> = HashMap::new();

    // Calculate transforms recursively, starting from root nodes
    for node in document.nodes() {
        if !parents.contains_key(&node.index()) {
            // This is a root node
            calculate_node_world_transform(node, &mat4x4_identity(), &mut world_transforms);
        }
    }

    world_transforms
}

fn calculate_node_world_transform(
    node: gltf::Node,
    parent_world_transform: &mat4x4,
    world_transforms: &mut HashMap<usize, mat4x4>
) {
    // Get local transform
    let (translation, rotation, scale) = node.transform().decomposed();
    let local_transform = mat4x4_from_transform(translation, rotation, scale);

    // Calculate world transform
    let world_transform = mat4x4_mul(*parent_world_transform, local_transform);
    world_transforms.insert(node.index(), world_transform);

    // Recursively calculate for children
    for child in node.children() {
        calculate_node_world_transform(child, &world_transform, world_transforms);
    }
}

fn generate_bones_from_nodes(
    document: &gltf::Document,
    world_transforms: &HashMap<usize, mat4x4>,
    parents: &HashMap<usize, usize>
) -> Vec<Bone> {
    let mut bones = Vec::new();

    for node in document.nodes() {
        if let Some(parent_index) = parents.get(&node.index()) {
            // This node has a parent, so create a bone from parent to this node
            if
                let (Some(parent_transform), Some(child_transform)) = (
                    world_transforms.get(parent_index),
                    world_transforms.get(&node.index()),
                )
            {
                let start_pos = mat4x4_extract_translation(parent_transform);
                let end_pos = mat4x4_extract_translation(child_transform);

                bones.push(Bone {
                    start_pos,
                    end_pos,
                    node_index: node.index(),
                    parent_index: Some(*parent_index),
                });
            }
        }
    }

    bones
}

fn get_skeleton_joints(
    document: &gltf::Document
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    // Get the first skin (assuming single skeleton)
    let skin = document.skins().next().ok_or("No skin found in GLTF")?;

    // Extract joint indices from the skin
    let joint_indices: Vec<usize> = skin
        .joints()
        .map(|joint| joint.index())
        .collect();

    Ok(joint_indices)
}

fn generate_skeleton_bones(
    document: &gltf::Document,
    world_transforms: &HashMap<usize, mat4x4>,
    parents: &HashMap<usize, usize>,
    joint_indices: &[usize]
) -> Vec<Bone> {
    let mut bones = Vec::new();

    // Create a set for fast lookup of joint indices
    let joint_set: HashSet<usize> = joint_indices.iter().cloned().collect();

    // Track which joints have children (are parents)
    let mut joints_with_children: HashSet<usize> = HashSet::new();

    // First pass: create parent-child bones using position-based approach (working method)
    for &joint_index in joint_indices {
        // Only create bones between skeleton joints
        if let Some(parent_index) = parents.get(&joint_index) {
            // Check if parent is also a skeleton joint
            if joint_set.contains(parent_index) {
                if
                    let (Some(parent_transform), Some(child_transform)) = (
                        world_transforms.get(parent_index),
                        world_transforms.get(&joint_index),
                    )
                {
                    let start_pos = mat4x4_extract_translation(parent_transform);
                    let end_pos = mat4x4_extract_translation(child_transform);

                    bones.push(Bone {
                        start_pos,
                        end_pos,
                        node_index: joint_index,
                        parent_index: Some(*parent_index),
                    });

                    // Mark the parent as having children
                    joints_with_children.insert(*parent_index);
                }
            }
        }
    }

    // Second pass: create leaf bones for joints without children
    for &joint_index in joint_indices {
        // If this joint is not a parent of any other joint, it's a leaf
        if !joints_with_children.contains(&joint_index) {
            if let Some(joint_transform) = world_transforms.get(&joint_index) {
                let joint_pos = mat4x4_extract_translation(joint_transform);

                // Create a leaf bone using fixed vector approach (like reference repository)
                let leaf_end_pos = {
                    // Fixed bone vector: 3.0 units in +Y direction
                    let fixed_bone_vector = [0.0, 3.0, 0.0];
                    
                    // Transform the fixed vector by the joint's world matrix
                    let transformed_vector = mat4x4_transform_direction(joint_transform, fixed_bone_vector);
                    
                    // End position = start + transformed vector
                    [
                        joint_pos[0] + transformed_vector[0],
                        joint_pos[1] + transformed_vector[1],
                        joint_pos[2] + transformed_vector[2],
                    ]
                };

                bones.push(Bone {
                    start_pos: joint_pos,
                    end_pos: leaf_end_pos,
                    node_index: joint_index,
                    parent_index: parents.get(&joint_index).copied(),
                });
            }
        }
    }

    bones
}

fn skeleton_to_line_vertices(skeleton: &Skeleton) -> Vec<f32> {
    let mut vertices = Vec::new();

    for bone in &skeleton.bones {
        // Add start position
        vertices.extend_from_slice(&bone.start_pos);
        // Add end position
        vertices.extend_from_slice(&bone.end_pos);
    }

    vertices
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
            source = source.replace("#VERSION", "#version 330 core");
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

// === BUFFER SETUP ===

fn setup_buffers(
    gl: &glow::Context,
    vertices: &[f32],
    indices: &[u32]
) -> Result<(glow::VertexArray, glow::Buffer, glow::Buffer), String> {
    unsafe {
        let vao = gl.create_vertex_array()?;
        let vbo = gl.create_buffer()?;
        let ebo = gl.create_buffer()?;

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(vertices),
            glow::STATIC_DRAW
        );
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(indices),
            glow::STATIC_DRAW
        );

        let stride = 6 * (std::mem::size_of::<f32>() as i32);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 3 * 4);
        gl.enable_vertex_attrib_array(1);

        gl.bind_vertex_array(None);
        Ok((vao, vbo, ebo))
    }
}

fn setup_line_buffers(
    gl: &glow::Context,
    vertices: &[f32]
) -> Result<(glow::VertexArray, glow::Buffer), String> {
    unsafe {
        let vao = gl.create_vertex_array()?;
        let vbo = gl.create_buffer()?;

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(vertices),
            glow::STATIC_DRAW
        );
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.bind_vertex_array(None);
        Ok((vao, vbo))
    }
}

// === PROGRAM ===

pub struct Program {
    gl: glow::Context,
    shader_program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    index_count: i32,
    line_vao: glow::VertexArray,
    line_vbo: glow::Buffer,
    skeleton: Skeleton,
    bone_count: i32,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        let (positions, normals, indices) = parse_mesh().map_err(|e|
            format!("Failed to parse mesh: {}", e)
        )?;

        let skeleton = parse_nodes().map_err(|e| format!("Failed to parse skeleton: {}", e))?;

        let vertices: Vec<f32> = positions
            .iter()
            .zip(normals.iter())
            .flat_map(|(p, n)| [p[0], p[1], p[2], n[0], n[1], n[2]])
            .collect();

        // Generate line vertices from skeleton
        let line_vertices = skeleton_to_line_vertices(&skeleton);
        let bone_count = skeleton.bones.len() as i32;

        unsafe {
            let vs_src = include_str!("assets/shaders/vertex.glsl");
            let fs_src = include_str!("assets/shaders/fragment.glsl");

            let vs = compile_shader(&gl, glow::VERTEX_SHADER, vs_src.to_string())?;
            let fs = compile_shader(&gl, glow::FRAGMENT_SHADER, fs_src.to_string())?;

            let program = gl.create_program()?;
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.bind_attrib_location(program, 0, "vPos");
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                return Err(format!("Program link error: {}", log));
            }

            gl.delete_shader(vs);
            gl.delete_shader(fs);
            gl.use_program(Some(program));
            gl.enable(glow::DEPTH_TEST);

            let (vao, vbo, ebo) = setup_buffers(&gl, &vertices, &indices)?;
            let (line_vao, line_vbo) = setup_line_buffers(&gl, &line_vertices)?;

            Ok(Self {
                gl,
                shader_program: program,
                vao,
                vbo,
                ebo,
                index_count: indices.len() as i32,
                line_vao,
                line_vbo,
                skeleton,
                bone_count,
            })
        }
    }

    pub fn render(&self, width: u32, height: u32, delta_time: f32) -> Result<(), String> {
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.use_program(Some(self.shader_program));

            self.gl.line_width(8.0);

            let angle = delta_time.rem_euclid(std::f32::consts::TAU);
            let world = mat4x4_mul(mat4x4_translate(0.0, 0.0, -6.0), mat4x4_rot_y(angle));
            let view = mat4x4_perspective(0.1, 10.0);

            if let Some(u) = self.gl.get_uniform_location(self.shader_program, "world_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&u), true, &world);
            }
            if let Some(u) = self.gl.get_uniform_location(self.shader_program, "viewport_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&u), true, &view);
            }

            // Render the skeleton bones
            if self.bone_count > 0 {
                self.gl.bind_vertex_array(Some(self.line_vao));
                self.gl.draw_arrays(glow::LINES, 0, self.bone_count * 2);
                self.gl.bind_vertex_array(None);
            }

            // Optionally render the mesh as well
            // self.gl.bind_vertex_array(Some(self.vao));
            // self.gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
        }
        Ok(())
    }

    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_program(self.shader_program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.ebo);
            self.gl.delete_vertex_array(self.line_vao);
            self.gl.delete_buffer(self.line_vbo);
        }
    }
}
