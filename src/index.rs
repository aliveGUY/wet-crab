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
    include!("engine/utils/GLTFLoaderUtils.rs");
}

mod assets_manager {
    include!("engine/managers/AssetsManager.rs");
}

use assets_manager::{ initialize, get_object3d_copy, Assets };

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
    static_shader_program: glow::Program,
    animated_shader_program: glow::Program,
    object3d1: Object3D, // Left character
    object3d2: Object3D, // Right character
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        initialize(&gl);
        let mut object3d1 = get_object3d_copy(Assets::TestingDoll);
        let mut object3d2 = get_object3d_copy(Assets::Chair);

        // Set initial positions using existing transform methods
        object3d1.transform.translate(-2.0, -3.0, -5.0);
        object3d2.transform.translate(2.0, -3.0, -5.0);

        unsafe {
            // Create animated shader program
            let animated_vs_src = get_vertex_shader_source();
            let animated_fs_src = get_fragment_shader_source();

            let animated_vs = compile_shader(&gl, glow::VERTEX_SHADER, animated_vs_src)?;
            let animated_fs = compile_shader(&gl, glow::FRAGMENT_SHADER, animated_fs_src)?;

            let animated_program = gl.create_program()?;
            gl.attach_shader(animated_program, animated_vs);
            gl.attach_shader(animated_program, animated_fs);
            gl.link_program(animated_program);

            if !gl.get_program_link_status(animated_program) {
                let log = gl.get_program_info_log(animated_program);
                return Err(format!("Animated shader program link error: {}", log));
            }

            gl.delete_shader(animated_vs);
            gl.delete_shader(animated_fs);

            // Create static shader program
            let static_vs_src = get_static_vertex_shader_source();
            let static_fs_src = get_static_fragment_shader_source();

            let static_vs = compile_shader(&gl, glow::VERTEX_SHADER, static_vs_src)?;
            let static_fs = compile_shader(&gl, glow::FRAGMENT_SHADER, static_fs_src)?;

            let static_program = gl.create_program()?;
            gl.attach_shader(static_program, static_vs);
            gl.attach_shader(static_program, static_fs);
            gl.link_program(static_program);

            if !gl.get_program_link_status(static_program) {
                let log = gl.get_program_info_log(static_program);
                return Err(format!("Static shader program link error: {}", log));
            }

            gl.delete_shader(static_vs);
            gl.delete_shader(static_fs);

            gl.enable(glow::DEPTH_TEST);

            println!("✅ Program initialized successfully with both shader programs");

            Ok(Self {
                gl,
                static_shader_program: static_program,
                animated_shader_program: animated_program,
                object3d1,
                object3d2,
            })
        }
    }

    pub fn render(&mut self, width: u32, height: u32, _delta_time: f32) -> Result<(), String> {
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            let fov = (90.0_f32).to_radians();
            let aspect_ratio = (width as f32) / (height as f32);
            let viewport_txfm = mat4x4_perspective(fov, aspect_ratio, 0.1, 10.0);

            // Extract shader programs and GL context to avoid borrowing issues
            let static_shader = self.static_shader_program;
            let animated_shader = self.animated_shader_program;
            let gl = &self.gl;

            // Render object1 with appropriate shader
            let shader_program1 = if let Some(material) = &self.object3d1.material {
                match material.shader_type {
                    ShaderType::Static => static_shader,
                    ShaderType::Animated => animated_shader,
                }
            } else {
                static_shader
            };
            Self::render_object_static(gl, &mut self.object3d1, &viewport_txfm, shader_program1);
            
            // Render object2 with appropriate shader
            let shader_program2 = if let Some(material) = &self.object3d2.material {
                match material.shader_type {
                    ShaderType::Static => static_shader,
                    ShaderType::Animated => animated_shader,
                }
            } else {
                static_shader
            };
            Self::render_object_static(gl, &mut self.object3d2, &viewport_txfm, shader_program2);

            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }

    fn render_object_static(gl: &glow::Context, object: &mut Object3D, viewport_txfm: &[f32; 16], shader_program: glow::Program) {
        unsafe {
            gl.use_program(Some(shader_program));

            // Set common uniforms
            gl.uniform_matrix_4_f32_slice(
                Some(&gl.get_uniform_location(shader_program, "viewport_txfm").unwrap()),
                true,
                viewport_txfm
            );

            if let Some(loc) = gl.get_uniform_location(shader_program, "baseColorTexture") {
                gl.uniform_1_i32(Some(&loc), 0); // Texture unit 0
            }
            if let Some(loc) = gl.get_uniform_location(shader_program, "hasTexture") {
                gl.uniform_1_i32(Some(&loc), 1); // Both objects have textures
            }

            // Render the object with the selected shader
            object.render(gl, &shader_program);
        }
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_program(self.static_shader_program);
            self.gl.delete_program(self.animated_shader_program);
            self.gl.delete_vertex_array(self.object3d1.mesh.vao);
            self.gl.delete_vertex_array(self.object3d2.mesh.vao);
        }
    }
}

// Platform-specific functions to be implemented by platform code
extern "Rust" {
    fn get_vertex_shader_source() -> String;
    fn get_fragment_shader_source() -> String;
    fn get_static_vertex_shader_source() -> String;
    fn get_static_fragment_shader_source() -> String;
}
