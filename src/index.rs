use glow::HasContext;

pub struct Program {
    gl: glow::Context,
    shader_program: glow::Program,
    vao: glow::VertexArray,
    aspect_ratio_location: Option<glow::UniformLocation>,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        unsafe {
            let vertex_shader = compile_vertex_shader(&gl)?;
            let fragment_shader = compile_fragment_shader(&gl)?;

            let shader_program = link_shader_program(&gl, vertex_shader, fragment_shader)?;

            let aspect_ratio_location = gl.get_uniform_location(shader_program, "u_aspect_ratio");

            let vao = setup_vertex_attributes(&gl)?;

            Ok(Program {
                gl,
                shader_program,
                vao,
                aspect_ratio_location,
            })
        }
    }

    pub fn render(&self, width: u32, height: u32) -> Result<(), String> {
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);

            let aspect_ratio = (width as f32) / (height as f32);

            self.gl.clear_color(0.1, 0.2, 0.3, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.use_program(Some(self.shader_program));

            if let Some(ref location) = self.aspect_ratio_location {
                self.gl.uniform_1_f32(Some(location), aspect_ratio);
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
        }
    }
}

fn create_triangle_vertices() -> [f32; 6] {
    [
        0.0,
        0.5, // Top vertex
        -0.5,
        -0.5, // Bottom left vertex
        0.5,
        -0.5, // Bottom right vertex
    ]
}

fn compile_vertex_shader(gl: &glow::Context) -> Result<glow::Shader, String> {
    let vertex_shader_source = include_str!("assets/vertex.glsl");
    compile_shader(gl, glow::VERTEX_SHADER, vertex_shader_source)
}

fn compile_fragment_shader(gl: &glow::Context) -> Result<glow::Shader, String> {
    let fragment_shader_source = include_str!("assets/fragment.glsl");
    compile_shader(gl, glow::FRAGMENT_SHADER, fragment_shader_source)
}

fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str
) -> Result<glow::Shader, String> {
    unsafe {
        let shader = gl
            .create_shader(shader_type)
            .map_err(|e| format!("Failed to create shader: {}", e))?;

        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let error = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            return Err(format!("Shader compilation failed: {}", error));
        }

        Ok(shader)
    }
}

fn link_shader_program(
    gl: &glow::Context,
    vertex_shader: glow::Shader,
    fragment_shader: glow::Shader
) -> Result<glow::Program, String> {
    unsafe {
        let program = gl.create_program().map_err(|e| format!("Failed to create program: {}", e))?;

        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);

        gl.delete_shader(vertex_shader);
        gl.delete_shader(fragment_shader);

        if !gl.get_program_link_status(program) {
            let error = gl.get_program_info_log(program);
            gl.delete_program(program);
            return Err(format!("Program linking failed: {}", error));
        }

        Ok(program)
    }
}

fn setup_vertex_attributes(gl: &glow::Context) -> Result<glow::VertexArray, String> {
    unsafe {
        let vao = gl.create_vertex_array().map_err(|e| format!("Failed to create VAO: {}", e))?;

        gl.bind_vertex_array(Some(vao));

        gl.bind_vertex_array(None);

        Ok(vao)
    }
}
