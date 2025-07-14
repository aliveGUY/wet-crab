use std::sync::{ Arc, RwLock };

use glow::HasContext;
use once_cell::sync::Lazy;

#[macro_use]
mod entity_component_system {
    include!("engine/systems/entityComponentSystem.rs");
}
use entity_component_system::*;

mod transform {
    include!("engine/components/Transform.rs");
}
use transform::Transform;

mod camera {
    include!("engine/components/Camera.rs");
}
use camera::Camera;

mod static_object3d {
    include!("engine/components/StaticObject3D.rs");
}
use static_object3d::StaticObject3D;

mod animated_object3d {
    include!("engine/components/AnimatedObject3D.rs");
}
use animated_object3d::AnimatedObject3D;

mod system_trait {
    include!("engine/components/System.rs");
}
use system_trait::System;

mod math {
    include!("engine/utils/math.rs");
}
use math::*;

mod input_utils {
    include!("engine/utils/input_utils.rs");
}

mod gltf_loader_utils {
    include!("engine/utils/GLTFLoaderUtils.rs");
}

mod shared_components {
    include!("engine/components/SharedComponents.rs");
}

mod assets_manager {
    include!("engine/managers/AssetsManager.rs");
}
use assets_manager::{
    initialize_asset_manager,
    get_static_object_copy,
    get_animated_object_copy,
    Assets,
    ASSETS_MANAGER,
};

mod event_system {
    include!("engine/systems/eventSystem.rs");
}
use event_system::{ EventSystem, EventType };

mod render_system {
    include!("game/systems/renderSystem.rs");
}
use render_system::RenderSystem;

mod movement_systems {
    include!("game/systems/movementSystem.rs");
}
use movement_systems::{ MovementSystem, CameraRotationSystem };

pub mod engine {
    pub mod systems {
        pub use super::super::entity_component_system::*;
        pub use super::super::event_system::*;

        mod input_system {
            include!("engine/systems/inputSystem.rs");
        }
        pub use input_system::*;
    }
}

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
            Transform::with_translation(2.0, -3.0, -5.0)
        );

        let doll_entity_id = spawn();
        insert_many!(
            doll_entity_id,
            get_animated_object_copy(Assets::TestingDoll),
            Transform::with_translation(-2.0, -3.0, -5.0)
        );

        let player_entity_id = spawn();
        *PLAYER_ENTITY_ID.write().unwrap() = Some(player_entity_id.clone());
        insert_many!(player_entity_id, Camera::new());

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
        Ok(())
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        println!("Program cleanup completed");
    }
}
