use glow::HasContext;

pub struct Program {
    gl: glow::Context,
    shader_program: glow::Program,
    vao: glow::VertexArray,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        unsafe {
            // Set viewport
            gl.viewport(0, 0, 800, 600);

            // Compile shaders
            let vertex_shader = compile_vertex_shader(&gl)?;
            let fragment_shader = compile_fragment_shader(&gl)?;
            
            // Link shader program
            let shader_program = link_shader_program(&gl, vertex_shader, fragment_shader)?;
            
            // Setup vertex array object
            let vao = setup_vertex_attributes(&gl)?;
            
            Ok(Program {
                gl,
                shader_program,
                vao,
            })
        }
    }
    
    pub fn render(&self) -> Result<(), String> {
        unsafe {
            // Clear screen with dark blue background
            self.gl.clear_color(0.1, 0.2, 0.3, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            
            // Use our shader program
            self.gl.use_program(Some(self.shader_program));
            
            // Bind vertex array
            self.gl.bind_vertex_array(Some(self.vao));
            
            // Draw the triangle (3 vertices)
            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
            
            // Cleanup bindings
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

// Triangle vertices definition (positions in normalized device coordinates)
fn create_triangle_vertices() -> [f32; 6] {
    [
        0.0,  0.5,   // Top vertex
       -0.5, -0.5,   // Bottom left vertex
        0.5, -0.5,   // Bottom right vertex
    ]
}

// Vertex shader - handles vertex positions
fn compile_vertex_shader(gl: &glow::Context) -> Result<glow::Shader, String> {
    let vertex_shader_source = include_str!("assets/vertex.glsl");
    compile_shader(gl, glow::VERTEX_SHADER, vertex_shader_source)
}

// Fragment shader - handles triangle color
fn compile_fragment_shader(gl: &glow::Context) -> Result<glow::Shader, String> {
    let fragment_shader_source = include_str!("assets/fragment.glsl");
    compile_shader(gl, glow::FRAGMENT_SHADER, fragment_shader_source)
}

// Generic shader compilation function
fn compile_shader(gl: &glow::Context, shader_type: u32, source: &str) -> Result<glow::Shader, String> {
    unsafe {
        let shader = gl.create_shader(shader_type)
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

// Link vertex and fragment shaders into a program
fn link_shader_program(gl: &glow::Context, vertex_shader: glow::Shader, fragment_shader: glow::Shader) -> Result<glow::Program, String> {
    unsafe {
        let program = gl.create_program()
            .map_err(|e| format!("Failed to create program: {}", e))?;
        
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);
        
        // Clean up shaders (they're now linked into the program)
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

// Setup vertex array object for triangle rendering
fn setup_vertex_attributes(gl: &glow::Context) -> Result<glow::VertexArray, String> {
    unsafe {
        let vao = gl.create_vertex_array()
            .map_err(|e| format!("Failed to create VAO: {}", e))?;
        
        gl.bind_vertex_array(Some(vao));
        
        // Note: We're using hardcoded vertices in the vertex shader,
        // so no vertex buffer setup is needed here
        
        gl.bind_vertex_array(None);
        
        Ok(vao)
    }
}
