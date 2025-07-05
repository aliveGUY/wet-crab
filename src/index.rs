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
    shader_program: glow::Program,
    object3d1: Object3D, // Left character
    object3d2: Object3D, // Right character
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        initialize(&gl);
        let mut object3d1 = get_object3d_copy(Assets::TestingDoll);
        let mut object3d2 = get_object3d_copy(Assets::TestingDoll);

        // Set initial positions using existing transform methods
        object3d1.transform.translate(-2.0, -3.0, -5.0);
        object3d2.transform.translate(2.0, -3.0, -5.0);

        unsafe {
            // Platform-specific shader source preparation is handled by platform code
            let vs_src = get_vertex_shader_source();
            let fs_src = get_fragment_shader_source();

            let vs = compile_shader(&gl, glow::VERTEX_SHADER, vs_src)?;
            let fs = compile_shader(&gl, glow::FRAGMENT_SHADER, fs_src)?;

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

            println!("✅ Program initialized successfully with assets from singleton manager");

            Ok(Self {
                gl,
                shader_program: program,
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
            self.gl.use_program(Some(self.shader_program));

            let fov = (90.0_f32).to_radians();
            let aspect_ratio = (width as f32) / (height as f32);
            let viewport_txfm = mat4x4_perspective(fov, aspect_ratio, 0.1, 10.0);

            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.gl.get_uniform_location(self.shader_program, "viewport_txfm").unwrap()),
                true,
                &viewport_txfm
            );

            if
                let Some(loc) = self.gl.get_uniform_location(
                    self.shader_program,
                    "baseColorTexture"
                )
            {
                self.gl.uniform_1_i32(Some(&loc), 0); // Texture unit 0
            }
            if let Some(loc) = self.gl.get_uniform_location(self.shader_program, "hasTexture") {
                self.gl.uniform_1_i32(Some(&loc), 1); // Both objects have textures
            }

            self.object3d1.render(&self.gl, &self.shader_program);
            self.object3d2.render(&self.gl, &self.shader_program);

            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_program(self.shader_program);
            self.gl.delete_vertex_array(self.object3d1.mesh.vao);
            self.gl.delete_vertex_array(self.object3d2.mesh.vao);
        }
    }
}

// Platform-specific functions to be implemented by platform code
extern "Rust" {
    fn get_vertex_shader_source() -> String;
    fn get_fragment_shader_source() -> String;
}
