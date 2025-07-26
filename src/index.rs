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

        spawn_chair();
        spawn_testing_doll();
        spawn_player();
        spawn_blockout_platform();

        EventSystem::subscribe(EventType::Move, Arc::new(MovementSystem));
        EventSystem::subscribe(EventType::RotateCamera, Arc::new(CameraRotationSystem));

        InterfaceSystem::update_entity_tree_global();

        // Demonstrate ECS serialization functionality
        println!("🔄 Testing ECS serialization...");
        save_world!("src/assets/scenes/test_world.json");

        // You can uncomment the line below to test loading:
        // load_world!("test_world.json");

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

    pub fn render(&mut self, width: u32, height: u32, _delta_time: f32) {
        RenderSystem::update(&self.gl, width, height);
    }

    pub fn get_gl_context(&self) -> &glow::Context {
        &self.gl
    }
}
