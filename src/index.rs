use glow::HasContext;

// Import ECS functionality
#[macro_use]
mod entity_component_system {
    include!("engine/systems/entityComponentSystem.rs");
}
use entity_component_system::*;

// Import Transform component
mod transform {
    include!("engine/components/Tranform.rs");
}
use transform::Transform;

mod math {
    include!("engine/utils/math.rs");
}
use math::*;

mod inputUtils {
    include!("engine/utils/inputUtils.rs");
}

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

// Import System trait
mod system_trait {
    include!("engine/components/System.rs");
}
use system_trait::System;

// Import render system
mod render_system {
    include!("game/systems/renderSystem.rs");
}
use render_system::RenderSystem;

// Import movement systems
mod movement_systems {
    include!("game/systems/movementSystem.rs");
}
use movement_systems::{MovementSystem, CameraRotationSystem};

pub mod movement_listeners {
    include!("game/listeners/MovementsListeners.rs");
}

use movement_listeners::{ MovementListener, CameraRotationListener };

// Import event system
mod event_system {
    include!("engine/systems/eventSystem.rs");
}
pub use event_system::{EventSystem, EventType};

// Create engine module structure for compatibility
pub mod engine {
    pub mod systems {
        pub use super::super::entity_component_system::*;
        pub use super::super::event_system::*;
        
        // Import input system  
        mod input_system {
            include!("engine/systems/inputSystem.rs");
        }
        pub use input_system::*;
    }
}

// === MAIN PROGRAM ===

pub struct Program {
    gl: glow::Context,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        initialize_asset_manager(&gl);
        initialize_game_state();

        // Create chair entity with ECS
        let chair_entity = spawn();
        let chair_object = get_static_object_copy(Assets::Chair);
        let mut chair_transform = Transform::new();
        chair_transform.translate(2.0, -3.0, -5.0);
        insert_many!(chair_entity, chair_object, chair_transform);

        // Create animated doll entity with ECS
        let doll_entity = spawn();
        let doll_object = get_animated_object_copy(Assets::TestingDoll);
        let mut doll_transform = Transform::new();
        doll_transform.translate(-2.0, -3.0, -5.0);
        insert_many!(doll_entity, doll_object, doll_transform);

        // Subscribe to events using clean singleton
        use std::sync::Arc;
        EventSystem::subscribe(EventType::Move, Arc::new(MovementListener));
        EventSystem::subscribe(EventType::RotateCamera, Arc::new(CameraRotationListener));

        unsafe {
            gl.enable(glow::DEPTH_TEST);
        }

        println!(
            "✅ Program initialized successfully with ECS-based architecture"
        );

        Ok(Self {
            gl,
        })
    }

    pub fn render(&mut self, width: u32, height: u32, _delta_time: f32) -> Result<(), String> {
        // Use the ECS-based RenderSystem instead of manual rendering
        RenderSystem::update(&self.gl, width, height);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        // Cleanup is now handled by the ECS system
        // Individual entities and their components will be cleaned up automatically
        println!("Program cleanup completed");
    }
}
