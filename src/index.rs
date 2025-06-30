use glow::HasContext;

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

// === VERTEX DATA ===

type mat4x4 = [f32; 16];

fn mat4x4_rot_z(angle_radians: f32) -> mat4x4 {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();

    [cos, -sin, 0.0, 0.0, 
    sin, cos, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0, 
    0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_rot_x(angle_radians: f32) -> mat4x4 {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();

    [1.0, 0.0, 0.0, 0.0, 
    0.0, cos, -sin, 0.0, 
    0.0, sin, cos, 0.0, 
    0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_rot_y(angle_radians: f32) -> mat4x4 {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();

    [cos, 0.0, -sin, 0.0, 
    0.0, 1.0, -0.0, 0.0, 
    sin, 0.0, cos, 0.0, 
    0.0, 0.0, 0.0, 1.0]
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

const VERTICES: [f32; 18] = [
    // Centered equilateral triangle
    // vPos             vCol
    0.0,
    0.577,
    0.0,
    1.0,
    0.0,
    0.0, // red - top vertex
    -0.5,
    -0.289,
    0.0,
    0.0,
    1.0,
    0.0, // green - bottom left vertex
    0.5,
    -0.289,
    0.0,
    0.0,
    0.0,
    1.0, // blue - bottom right vertex
];

// === HELPERS ===

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

        // Upload vertex data (interleaved positions and normals)
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(vertices),
            glow::STATIC_DRAW
        );

        // Upload index data
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(indices),
            glow::STATIC_DRAW
        );

        // Stride is now 6 floats (3 position + 3 normal)
        let stride = 6 * (std::mem::size_of::<f32>() as i32);

        // vPos (attribute 0): vec3, offset 0
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);

        // vNormal (attribute 1): vec3, offset 3 floats = 12 bytes
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 3 * 4);
        gl.enable_vertex_attrib_array(1);

        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);

        Ok((vao, vbo, ebo))
    }
}

pub struct Program {
    gl: glow::Context,
    shader_program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    index_count: i32,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        let (positions, normals, indices) = parse_mesh().map_err(|e|
            format!("Failed to parse mesh: {}", e)
        )?;

        // Interleave positions and normals
        let vertices: Vec<f32> = positions
            .iter()
            .zip(normals.iter())
            .flat_map(|(pos, norm)| [pos[0], pos[1], pos[2], norm[0], norm[1], norm[2]])
            .collect();
        unsafe {
            // Compile shaders
            let vertex_shader_source = include_str!("assets/shaders/vertex.glsl");
            let fragment_shader_source = include_str!("assets/shaders/fragment.glsl");
            let vertex_shader = compile_shader(
                &gl,
                glow::VERTEX_SHADER,
                vertex_shader_source.to_string()
            )?;
            let fragment_shader = compile_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                fragment_shader_source.to_string()
            )?;

            // Create and link program
            let program = gl.create_program()?;
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.bind_attrib_location(program, 0, "vPos");
            gl.bind_attrib_location(program, 1, "vNormal");
            gl.link_program(program);

            // Check for link errors
            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                gl.delete_program(program);
                return Err(format!("Program link error: {}", log));
            }

            // Clean up shaders
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);

            // Use the program before setting uniforms
            gl.use_program(Some(program));
            gl.enable(glow::DEPTH_TEST);

            // Setup VAO, VBO, and EBO
            let (vao, vbo, ebo) = setup_buffers(&gl, &vertices, &indices)?;

            Ok(Program {
                gl,
                shader_program: program,
                vao,
                vbo,
                ebo,
                index_count: indices.len() as i32,
            })
        }
    }

    pub fn render(&self, width: u32, height: u32, delta_time: f32) -> Result<(), String> {
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            self.gl.use_program(Some(self.shader_program));

            let angle = delta_time.rem_euclid(std::f32::consts::TAU);

            let mut world_transform = mat4x4_translate(0.0, 0.8, 0.0);
            world_transform = mat4x4_mul(mat4x4_rot_y(angle), world_transform);
            world_transform = mat4x4_mul(mat4x4_translate(0.0, 0.0, -6.0), world_transform);
            let viewport_transform = mat4x4_perspective(0.1, 10.0);

            if
                let Some(location) = self.gl.get_uniform_location(
                    self.shader_program,
                    "world_transform"
                )
            {
                self.gl.uniform_matrix_4_f32_slice(Some(&location), true, &world_transform);
            }

            if
                let Some(location) = self.gl.get_uniform_location(
                    self.shader_program,
                    "viewport_transform"
                )
            {
                self.gl.uniform_matrix_4_f32_slice(Some(&location), true, &viewport_transform);
            }

            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
            self.gl.bind_vertex_array(None);
            self.gl.use_program(None);
        }

        Ok(())
    }

    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_program(self.shader_program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.ebo);
        }
    }
}
