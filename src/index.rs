use glow::HasContext;

// === VERTEX DATA ===

fn mat2x2_rot(angle_radians: f32) -> [f32; 4] {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();

    [cos, -sin, sin, cos]
}

// === MATRIX MATH ===

fn create_orthographic_projection(aspect_ratio: f32) -> [f32; 16] {
    // Create an orthographic projection that maintains aspect ratio
    // We'll use a coordinate system from -1 to 1, but adjust for aspect ratio
    let (left, right, bottom, top) = if aspect_ratio >= 1.0 {
        // Wide screen: expand horizontal range
        (-aspect_ratio, aspect_ratio, -1.0, 1.0)
    } else {
        // Tall screen: expand vertical range
        (-1.0, 1.0, -1.0 / aspect_ratio, 1.0 / aspect_ratio)
    };
    
    let near = -1.0;
    let far = 1.0;
    
    // Orthographic projection matrix (column-major order for OpenGL)
    [
        2.0 / (right - left), 0.0, 0.0, 0.0,
        0.0, 2.0 / (top - bottom), 0.0, 0.0,
        0.0, 0.0, -2.0 / (far - near), 0.0,
        -(right + left) / (right - left), -(top + bottom) / (top - bottom), -(far + near) / (far - near), 1.0,
    ]
}

const VERTICES: [f32; 15] = [
    // Equilateral triangle centered at origin
    // vPos      vCol
    0.0,
    0.6,
    1.0,
    0.0,
    0.0, // red - top vertex
    -0.5196,
    -0.3,
    0.0,
    1.0,
    0.0, // green - bottom left vertex
    0.5196,
    -0.3,
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

fn setup_buffers(gl: &glow::Context) -> Result<(glow::VertexArray, glow::Buffer), String> {
    unsafe {
        let vao = gl.create_vertex_array()?;
        let vbo = gl.create_buffer()?;

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&VERTICES),
            glow::STATIC_DRAW
        );

        let stride = 5 * (std::mem::size_of::<f32>() as i32);

        // vPos (attribute 0): vec2, offset 0
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);

        // vCol (attribute 1): vec3, offset = 2 floats
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 2 * 4);
        gl.enable_vertex_attrib_array(1);

        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);

        Ok((vao, vbo))
    }
}

pub struct Program {
    gl: glow::Context,
    shader_program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        unsafe {
            // Compile shaders
            let vertex_shader_source = include_str!("assets/vertex.glsl");
            let fragment_shader_source = include_str!("assets/fragment.glsl");
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
            gl.bind_attrib_location(program, 1, "vCol");
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

            // Set the rotation matrix uniform
            let rot_matrix = mat2x2_rot(3.14 / 2.0);
            if let Some(location) = gl.get_uniform_location(program, "rot") {
                gl.uniform_matrix_2_f32_slice(Some(&location), true, &rot_matrix);
            } else {
                return Err("Failed to get uniform location for 'rot'".to_string());
            }

            // Setup VAO and VBO
            let (vao, vbo) = setup_buffers(&gl)?;

            Ok(Program {
                gl,
                shader_program: program,
                vao,
                vbo,
            })
        }
    }

    pub fn render(&self, width: u32, height: u32, delta_time: f32) -> Result<(), String> {
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.use_program(Some(self.shader_program));
            
            // Calculate aspect ratio and create projection matrix
            let aspect_ratio = width as f32 / height as f32;
            let projection_matrix = create_orthographic_projection(aspect_ratio);
            
            // Set projection matrix uniform
            if let Some(location) = self.gl.get_uniform_location(self.shader_program, "projection") {
                self.gl.uniform_matrix_4_f32_slice(Some(&location), false, &projection_matrix);
            } else {
                return Err("Failed to get uniform location for 'projection'".to_string());
            }
            
            // Update rotation based on delta time for smooth animation
            let rotation_speed = 1.0; // radians per second
            let angle = delta_time * rotation_speed;
            let rot_matrix = mat2x2_rot(angle);
            
            if let Some(location) = self.gl.get_uniform_location(self.shader_program, "rot") {
                self.gl.uniform_matrix_2_f32_slice(Some(&location), true, &rot_matrix);
            }

            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
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
        }
    }
}
