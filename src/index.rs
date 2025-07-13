use glow::HasContext;

mod math {
    include!("engine/utils/math.rs");
}
use math::*;

mod shared_components {
    include!("engine/components/SharedComponents.rs");
}

mod static_object3d {
    include!("engine/components/StaticObject3D.rs");
}

mod animated_object3d {
    include!("engine/components/AnimatedObject3D.rs");
}

use static_object3d::StaticObject3D;
use animated_object3d::AnimatedObject3D;

mod gltf_loader_utils {
    include!("engine/utils/GLTFLoaderUtils.rs");
}

mod collider_system {
    include!("engine/systems/ColliderSystem.rs");
}

use collider_system::{ColliderSystem, ColliderShape};

mod assets_manager {
    include!("engine/managers/AssetsManager.rs");
}

use assets_manager::{initialize, get_static_object_copy, get_animated_object_copy, Assets};

pub mod event_system {
    include!("engine/systems/EventSystem.rs");
}

pub mod movement_listeners {
    include!("game/listeners/MovementsListeners.rs");
}

use event_system::EventSystem;
use movement_listeners::{MovementListener, CameraRotationListener};

pub use event_system::{Event, EventType};

pub struct Program {
    gl: glow::Context,
    animated_object: AnimatedObject3D,
    static_object: StaticObject3D,
    event_system: EventSystem,

    debug_shader: glow::Program,
    debug_circle_vao: glow::VertexArray,
    debug_box_vao: glow::VertexArray,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        initialize(&gl);

        let mut animated_object = get_animated_object_copy(Assets::TestingDoll);
        let mut static_object = get_static_object_copy(Assets::Chair);

        animated_object.transform.translate(-2.0, -3.0, -5.0);
        static_object.transform.translate(4.0, -5.0, -7.0);

        let mut event_system = EventSystem::new();
        event_system.subscribe(EventType::Move, Box::new(MovementListener));
        event_system.subscribe(EventType::RotateCamera, Box::new(CameraRotationListener));

        let (debug_shader, debug_circle_vao, debug_box_vao) = unsafe {
            gl.enable(glow::DEPTH_TEST);

            let vertex_src = include_str!("assets/shaders/debug_shader.vert");
            let fragment_src = include_str!("assets/shaders/debug_shader.frag");

            let vs = gl.create_shader(glow::VERTEX_SHADER)?;
            gl.shader_source(vs, vertex_src);
            gl.compile_shader(vs);
            if !gl.get_shader_compile_status(vs) {
                return Err(gl.get_shader_info_log(vs));
            }

            let fs = gl.create_shader(glow::FRAGMENT_SHADER)?;
            gl.shader_source(fs, fragment_src);
            gl.compile_shader(fs);
            if !gl.get_shader_compile_status(fs) {
                return Err(gl.get_shader_info_log(fs));
            }

            let program = gl.create_program()?;
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(gl.get_program_info_log(program));
            }

            gl.delete_shader(vs);
            gl.delete_shader(fs);

            // === Circle VAO ===
            let circle_vao = gl.create_vertex_array().unwrap();
            let circle_vbo = gl.create_buffer().unwrap();

            let mut vertices = Vec::new();
            for i in 0..=32 {
                let angle = (i as f32) / 32.0 * std::f32::consts::TAU;
                vertices.push(angle.cos() * 0.6);
                vertices.push(angle.sin() * 0.6);
                vertices.push(0.0);
            }

            gl.bind_vertex_array(Some(circle_vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(circle_vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);

            // === Box VAO ===
            let box_vao = gl.create_vertex_array().unwrap();
            let box_vbo = gl.create_buffer().unwrap();

            let box_vertices: [f32; 12] = [
                -0.8, -0.8, 0.0,
                 0.8, -0.8, 0.0,
                 0.8,  0.8, 0.0,
                -0.8,  0.8, 0.0,
            ];

            gl.bind_vertex_array(Some(box_vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(box_vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&box_vertices), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);

            (program, circle_vao, box_vao)
        };

        println!("✅ Program initialized successfully with debug collider rendering");

        Ok(Self {
            gl,
            animated_object,
            static_object,
            event_system,
            debug_shader,
            debug_circle_vao,
            debug_box_vao,
        })
    }

    pub fn render(&mut self, width: u32, height: u32, _delta_time: f32) -> Result<(), String> {
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            let fov = (90.0_f32).to_radians();
            let aspect_ratio = (width as f32) / (height as f32);
            let viewport_txfm = mat4x4_perspective(fov, aspect_ratio, 0.1, 10.0);

            self.setup_viewport_uniform(&viewport_txfm, self.animated_object.material.shader_program);
            self.setup_viewport_uniform(&viewport_txfm, self.static_object.material.shader_program);

            let person_pos = self.animated_object.transform.get_position_xy();
            let chair_pos = self.static_object.transform.get_position_xy();

            let person_shape = ColliderShape::Circle { radius: 0.6 };
            let chair_shape = ColliderShape::AABB { half_extents: [0.8, 0.8] };

            if ColliderSystem::check_collision(person_pos, person_shape, chair_pos, chair_shape) {
                println!("🟥 Collision detected");
            }

            self.animated_object.render(&self.gl);
            self.static_object.render(&self.gl);

            self.render_debug_colliders(&viewport_txfm);
            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }

    fn setup_viewport_uniform(&self, viewport_txfm: &[f32; 16], shader_program: glow::Program) {
        unsafe {
            self.gl.use_program(Some(shader_program));

            if let Some(loc) = self.gl.get_uniform_location(shader_program, "viewport_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, viewport_txfm);
            }

            if let Some(loc) = self.gl.get_uniform_location(shader_program, "baseColorTexture") {
                self.gl.uniform_1_i32(Some(&loc), 0);
            }
            if let Some(loc) = self.gl.get_uniform_location(shader_program, "hasTexture") {
                self.gl.uniform_1_i32(Some(&loc), 1);
            }
        }
    }

    fn render_debug_colliders(&self, viewport_txfm: &[f32; 16]) {
        unsafe {
            self.gl.use_program(Some(self.debug_shader));

            if let Some(loc) = self.gl.get_uniform_location(self.debug_shader, "viewport_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, viewport_txfm);
            }

            // Circle
            let pos_xy = self.animated_object.transform.get_position_xy();
            let world = mat4x4_translate(pos_xy[0], pos_xy[1], 0.0);
            if let Some(loc) = self.gl.get_uniform_location(self.debug_shader, "world_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, &world);
            }
            self.gl.bind_vertex_array(Some(self.debug_circle_vao));
            self.gl.draw_arrays(glow::LINE_LOOP, 0, 33);

            // Box
            let pos_xy = self.static_object.transform.get_position_xy();
            let world = mat4x4_translate(pos_xy[0], pos_xy[1], 0.0);
            if let Some(loc) = self.gl.get_uniform_location(self.debug_shader, "world_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, &world);
            }
            self.gl.bind_vertex_array(Some(self.debug_box_vao));
            self.gl.draw_arrays(glow::LINE_LOOP, 0, 4);
        }
    }

    pub fn receive_event(&mut self, event: &event_system::Event) {
        self.event_system.notify(event);
    }

    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_vertex_array(self.animated_object.mesh.vao);
            self.gl.delete_vertex_array(self.static_object.mesh.vao);
            self.gl.delete_vertex_array(self.debug_circle_vao);
            self.gl.delete_vertex_array(self.debug_box_vao);
            self.gl.delete_program(self.debug_shader);
        }
    }
}
