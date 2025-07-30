use std::sync::{ Arc, RwLock };

use glow::HasContext;
use once_cell::sync::Lazy;

#[path = "engine/mod.rs"]
#[macro_use]
pub mod engine;
#[path = "game/mod.rs"]
pub mod game;

use engine::*;
use game::*;

pub static PLAYER_ENTITY_ID: Lazy<RwLock<Option<EntityId>>> = Lazy::new(|| RwLock::new(None));

pub struct Program {
    gl: glow::Context,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        initialize_asset_manager(&gl);

        EventSystem::subscribe(EventType::Move, Arc::new(MovementSystem));
        EventSystem::subscribe(EventType::RotateCamera, Arc::new(CameraRotationSystem));

        InterfaceSystem::update_entity_tree_global();

        load_world!("src/assets/scenes/test_world.json");

        // Update UI to reflect loaded entities
        InterfaceSystem::update_entity_tree_global();

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.depth_mask(true);

            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
            gl.front_face(glow::CCW);

            let depth_bits = gl.get_parameter_i32(glow::DEPTH_BITS);
            if depth_bits == 0 {
                eprintln!("[WARNING] No depth buffer detected in Program::new()");
                eprintln!("[WARNING] Depth testing may not work correctly");
            } else {
                println!("[DEBUG] Program initialized with {} bit depth buffer", depth_bits);
            }
        }

        println!("✅ Program initialized successfully with ECS-based architecture");

        Ok(Self { gl })
    }

    pub fn render(&mut self, width: u32, height: u32, delta_time: f32) {
        let mut viewport = [0i32; 4];
        let mut program = 0i32;
        let mut depth_func = 0;
        let mut writemask = 0i32;
        unsafe {
            self.gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);
            self.gl.get_parameter_i32_slice(glow::CURRENT_PROGRAM, std::slice::from_mut(&mut program));
            self.gl.get_parameter_i32_slice(glow::DEPTH_FUNC, std::slice::from_mut(&mut depth_func));
            self.gl.get_parameter_i32_slice(glow::DEPTH_WRITEMASK, std::slice::from_mut(&mut writemask));
            self.gl.enable(glow::DEPTH_TEST);
            self.gl.depth_func(glow::LESS);
            self.gl.depth_mask(true);
            self.gl.enable(glow::CULL_FACE);
            self.gl.cull_face(glow::BACK);
            self.gl.front_face(glow::CCW);
            self.gl.viewport(0, 0, width as i32, height as i32);
        }

        RenderSystem::update(&self.gl, width, height);

        unsafe {
            self.gl.viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.depth_func(depth_func as u32);
            self.gl.depth_mask(writemask != 0);
            self.gl.disable(glow::CULL_FACE);
            self.gl.disable(glow::BLEND);
            // Note: Skipping program restoration as it requires proper OpenGL program handle management
            self.gl.clear(glow::DEPTH_BUFFER_BIT);
        }
    }
}
