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

mod assets_manager {
    include!("engine/managers/AssetsManager.rs");
}

mod game_state {
    include!("game/gloabals/GameState.rs");
}

use assets_manager::{ initialize_asset_manager, get_static_object_copy, get_animated_object_copy, Assets };
use game_state::{initialize_game_state, get_camera_transform};

pub mod event_system {
    include!("engine/systems/EventSystem.rs");
}

pub mod movement_listeners {
    include!("game/listeners/MovementsListeners.rs");
}

use event_system::EventSystem;
use movement_listeners::{ MovementListener, CameraRotationListener };

// Re-export for platform-specific builds
pub use event_system::{Event, EventType};

// === MAIN PROGRAM ===

pub struct Program {
    gl: glow::Context,
    animated_object: AnimatedObject3D,
    static_object: StaticObject3D,
    event_system: EventSystem,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        initialize_asset_manager(&gl);
        initialize_game_state();

        // Load objects with correct types
        let mut animated_object = get_animated_object_copy(Assets::TestingDoll);
        let mut static_object = get_static_object_copy(Assets::Chair);

        // Set initial positions
        animated_object.transform.translate(-2.0, -3.0, -5.0);
        static_object.transform.translate(2.0, -3.0, -5.0);

        let mut event_system = EventSystem::new();

        event_system.subscribe(event_system::EventType::Move, Box::new(MovementListener));
        event_system.subscribe(event_system::EventType::RotateCamera, Box::new(CameraRotationListener));

        unsafe {
            gl.enable(glow::DEPTH_TEST);
        }

        println!(
            "✅ Program initialized successfully with shared components and shader-in-material architecture"
        );

        Ok(Self {
            gl,
            animated_object,
            static_object,
            event_system,
        })
    }

    pub fn render(&mut self, width: u32, height: u32, _delta_time: f32) -> Result<(), String> {
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            // Create projection matrix
            let fov = (90.0_f32).to_radians();
            let aspect_ratio = (width as f32) / (height as f32);
            let projection_matrix = mat4x4_perspective(fov, aspect_ratio, 0.1, 10.0);

            // Get camera transform (view matrix)
            let view_matrix = get_camera_transform();

            // Combine projection and view matrices for final viewport transform
            let viewport_txfm = mat4x4_mul(projection_matrix, view_matrix);

            // Set viewport transform for both objects (they handle their own shaders)
            self.setup_viewport_uniform(
                &viewport_txfm,
                self.animated_object.material.shader_program
            );
            self.setup_viewport_uniform(&viewport_txfm, self.static_object.material.shader_program);

            // Render objects - they handle their own shader binding and uniforms
            self.animated_object.render(&self.gl);
            self.static_object.render(&self.gl);

            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }

    pub fn receive_event(&mut self, event: &event_system::Event) {
        self.event_system.notify(event);
    }

    fn setup_viewport_uniform(&self, viewport_txfm: &[f32; 16], shader_program: glow::Program) {
        unsafe {
            self.gl.use_program(Some(shader_program));

            // Set viewport transform
            if let Some(loc) = self.gl.get_uniform_location(shader_program, "viewport_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&loc), true, viewport_txfm);
            }

            // Set texture uniforms
            if let Some(loc) = self.gl.get_uniform_location(shader_program, "baseColorTexture") {
                self.gl.uniform_1_i32(Some(&loc), 0); // Texture unit 0
            }
            if let Some(loc) = self.gl.get_uniform_location(shader_program, "hasTexture") {
                self.gl.uniform_1_i32(Some(&loc), 1); // Both objects have textures
            }
        }
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_vertex_array(self.animated_object.mesh.vao);
            self.gl.delete_vertex_array(self.static_object.mesh.vao);
        }
    }
}
