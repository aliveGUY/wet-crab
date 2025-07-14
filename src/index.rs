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

        let chair_entity_id = spawn();
        insert_many!(
            chair_entity_id,
            get_static_object_copy(Assets::Chair),
            engine::components::SharedComponents::Transform::with_translation(2.0, -3.0, -5.0),
            engine::components::SharedComponents::Collider::new()
        );

        let doll_entity_id = spawn();
        insert_many!(
            doll_entity_id,
            get_animated_object_copy(Assets::TestingDoll),
            engine::components::SharedComponents::Transform::with_translation(-2.0, -3.0, -5.0)
        );

        let player_entity_id = spawn();
        *PLAYER_ENTITY_ID.write().unwrap() = Some(player_entity_id.clone());
        insert_many!(player_entity_id, engine::components::CameraComponent::new());

        EventSystem::subscribe(EventType::Move, Arc::new(MovementSystem));
        EventSystem::subscribe(EventType::RotateCamera, Arc::new(CameraRotationSystem));

        unsafe {
            gl.enable(glow::DEPTH_TEST);
        }

        println!("✅ Program initialized successfully with ECS-based architecture");

        Ok(Self { gl })
    }

    pub fn render(&mut self, width: u32, height: u32, _delta_time: f32) -> Result<(), String> {
        RenderSystem::update(&self.gl, width, height);
        ColliderSystem::update();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        println!("Program cleanup completed");
    }
}
