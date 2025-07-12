use glow::HasContext;
use std::rc::Rc;
use std::cell::RefCell;

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

use assets_manager::{
    initialize_asset_manager,
    get_static_object_copy,
    get_animated_object_copy,
    Assets,
};
use game_state::{ initialize_game_state, get_camera_transform };

#[path = "engine/mod.rs"]
pub mod engine;

pub mod movement_listeners {
    include!("game/listeners/MovementsListeners.rs");
}

use movement_listeners::{ MovementListener, CameraRotationListener };

// Re-export for platform-specific builds
pub use engine::eventSystem::{ Event, EventType, EventSystem };

#[cfg(not(target_arch = "wasm32"))]
pub use engine::eventSystem::DesktopEventHandler;

#[cfg(target_arch = "wasm32")]
pub use engine::eventSystem::BrowserEventHandler;

// === MAIN PROGRAM ===

pub struct Program {
    gl: glow::Context,
    animated_object: AnimatedObject3D,
    static_object: StaticObject3D,
    event_system: Rc<RefCell<EventSystem>>,
}

impl Program {
    pub fn new(gl: glow::Context, event_system: Rc<RefCell<EventSystem>>) -> Result<Self, String> {
        initialize_asset_manager(&gl);
        initialize_game_state();

        // Load objects with correct types
        let mut animated_object = get_animated_object_copy(Assets::TestingDoll);
        let mut static_object = get_static_object_copy(Assets::Chair);

        // Set initial positions
        animated_object.transform.translate(-2.0, -3.0, -5.0);
        static_object.transform.translate(2.0, -3.0, -5.0);

        {
            let mut es = event_system.borrow_mut();
            es.subscribe(EventType::Move, Box::new(MovementListener));
            es.subscribe(EventType::RotateCamera, Box::new(CameraRotationListener));
        }

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

            let fov = (90.0_f32).to_radians();
            let aspect_ratio = (width as f32) / (height as f32);
            let projection_matrix = mat4x4_perspective(fov, aspect_ratio, 0.1, 10.0);

            let view_matrix = get_camera_transform();

            let viewport_txfm = mat4x4_mul(projection_matrix, view_matrix);

            self.setup_viewport_uniform(
                &viewport_txfm,
                self.animated_object.material.shader_program
            );
            self.setup_viewport_uniform(&viewport_txfm, self.static_object.material.shader_program);

            self.animated_object.render(&self.gl);
            self.static_object.render(&self.gl);

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

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_vertex_array(self.animated_object.mesh.vao);
            self.gl.delete_vertex_array(self.static_object.mesh.vao);
        }
    }
}
