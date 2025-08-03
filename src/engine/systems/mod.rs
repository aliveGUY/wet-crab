pub mod event_system;
pub mod keyboard_input_system;
pub mod interface_system;
pub mod scene_format;

// New ECS system
pub mod ecs;
#[macro_use]
pub mod ecs_macros;

// Re-export the main types for easy access
pub use event_system::{ EventSystem, EventType };
pub use keyboard_input_system::{ KeyboardInputSystem };
pub use interface_system::{ InterfaceSystem };

// Re-export ECS functionality for clean imports
pub use ecs::*;
pub use ecs_macros::spawn;

// Re-export serialization macros
pub use crate::save_world;
pub use crate::load_world;
