use glow::HasContext;
use std::collections::HashMap;

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

pub fn parse_nodes() -> Result<(), Box<dyn std::error::Error>> {
    let gltf_data = include_bytes!("assets/meshes/guy.gltf");

    let gltf = gltf::Gltf::from_slice(gltf_data)?;
    let document = gltf.document;

    // Build parent-child relationships
    let mut parents: HashMap<usize, usize> = HashMap::new();
    
    for scene in document.scenes() {
        for root_node in scene.nodes() {
            build_parent_map(root_node, None, &mut parents);
        }
    }

    for (index, node) in document.nodes().enumerate() {
        println!("Node {}:", index);

        // Default values per GLTF spec
        let translation = node.transform().decomposed().0;
        let rotation = node.transform().decomposed().1;
        let scale = node.transform().decomposed().2;

        println!(
            "  🧭 Translation: [{:.3}, {:.3}, {:.3}]",
            translation[0],
            translation[1],
            translation[2]
        );
        println!(
            "  🔁 Rotation:    [{:.3}, {:.3}, {:.3}, {:.3}]",
            rotation[0],
            rotation[1],
            rotation[2],
            rotation[3]
        );
        println!("  📏 Scale:       [{:.3}, {:.3}, {:.3}]", scale[0], scale[1], scale[2]);

        if let Some(parent_index) = parents.get(&index) {
            println!("  🔗 Parent: Node {}", parent_index);
        } else {
            println!("  🔗 Parent: (root)");
        }

        println!();
    }

    Ok(())
}

fn build_parent_map(node: gltf::Node, parent_index: Option<usize>, parents: &mut HashMap<usize, usize>) {
    if let Some(parent) = parent_index {
        parents.insert(node.index(), parent);
    }
    
    for child in node.children() {
        build_parent_map(child, Some(node.index()), parents);
    }
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
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        let (positions, normals, indices) = parse_mesh().map_err(|e|
            format!("Failed to parse mesh: {}", e)
        )?;

        parse_nodes();

        let vertices: Vec<f32> = positions
            .iter()
            .zip(normals.iter())
            .flat_map(|(p, n)| [p[0], p[1], p[2], n[0], n[1], n[2]])
            .collect();

        let line_vertices: [f32; 6] = [-0.5, 0.0, 0.0, 0.5, 0.0, 0.0];

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

            // self.gl.bind_vertex_array(Some(self.vao));
            // self.gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);

            self.gl.bind_vertex_array(Some(self.line_vao));
            self.gl.draw_arrays(glow::LINES, 0, 2);

            self.gl.bind_vertex_array(None);
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
