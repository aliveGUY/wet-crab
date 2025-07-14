use crate::index::math::*;
use glow::HasContext;

pub struct DebugRenderer {
    pub shader: glow::Program,
    pub sphere_vao: glow::VertexArray,
    pub sphere_indices_count: usize,
}

impl DebugRenderer {
    pub fn new(gl: &glow::Context) -> Self {
        let vertex_shader = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            uniform mat4 viewport_txfm;
            uniform mat4 world_txfm;
            void main() {
                gl_Position = viewport_txfm * world_txfm * vec4(position, 1.0);
            }
        "#;

        let fragment_shader = r#"
            #version 330 core
            out vec4 FragColor;
            void main() {
                FragColor = vec4(1.0, 0.0, 0.0, 0.3);
            }
        "#;

        unsafe {
            let program = gl.create_program().unwrap();
            let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs, vertex_shader);
            gl.compile_shader(vs);
            gl.attach_shader(program, vs);

            let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs, fragment_shader);
            gl.compile_shader(fs);
            gl.attach_shader(program, fs);

            gl.link_program(program);

            // Создать простую wireframe-сферу (например, икосаэдр с линиями)
            let (vao, indices_count) = create_wireframe_sphere(gl);

            Self {
                shader: program,
                sphere_vao: vao,
                sphere_indices_count: indices_count,
            }
        }
    }

    pub fn render_sphere(&self, gl: &glow::Context, pos: [f32;3], radius: f32, viewport_txfm: &[f32;16]) {
        let model = mat4x4_scale_translate(radius, pos);

        unsafe {
            gl.use_program(Some(self.shader));

            if let Some(loc) = gl.get_uniform_location(self.shader, "viewport_txfm") {
                gl.uniform_matrix_4_f32_slice(Some(&loc), true, viewport_txfm);
            }
            if let Some(loc) = gl.get_uniform_location(self.shader, "world_txfm") {
                gl.uniform_matrix_4_f32_slice(Some(&loc), true, &model);
            }

            gl.bind_vertex_array(Some(self.sphere_vao));
            gl.draw_elements(glow::LINES, self.sphere_indices_count as i32, glow::UNSIGNED_SHORT, 0);
        }
    }
}

// Создаём простую wireframe-сферу (или икосаэдр в линиях)
fn create_wireframe_sphere(gl: &glow::Context) -> (glow::VertexArray, usize) {
    let vertices: &[f32] = &[
        1.0, 0.0, 0.0,  -1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,   0.0, -1.0, 0.0,
        0.0, 0.0, 1.0,   0.0, 0.0, -1.0,
    ];

    let indices: &[u16] = &[
        0,2, 2,1, 1,3, 3,0, // XY plane
        0,4, 4,1, 1,5, 5,0, // XZ plane
        2,4, 4,3, 3,5, 5,2, // YZ plane
    ];

    unsafe {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(vertices), glow::STATIC_DRAW);

        let ebo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(indices), glow::STATIC_DRAW);

        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);

        (vao, indices.len())
    }
}

// Scale + translate matrix
fn mat4x4_scale_translate(scale: f32, pos: [f32;3]) -> [f32;16] {
    let mut m = mat4x4_identity();
    m[0] = scale;
    m[5] = scale;
    m[10] = scale;
    m[12] = pos[0];
    m[13] = pos[1];
    m[14] = pos[2];
    m
}
