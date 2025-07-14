pub mod gloabals;
pub mod systems;

// Re-export commonly used types
pub use gloabals::GameState::{initialize_game_state, get_camera_transform};
pub use systems::renderSystem::RenderSystem;
pub use systems::movementSystem::{MovementSystem, CameraRotationSystem};
