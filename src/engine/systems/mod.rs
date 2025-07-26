pub mod event_system;
pub mod input_system;
pub mod interface_system;
pub mod serialization;
#[macro_use]
pub mod entity_component_system;

// Re-export the main types for easy access
pub use event_system::{ EventSystem, EventType };
pub use input_system::{ InputSystem, DesktopInputHandler };
pub use interface_system::{ InterfaceSystem };
pub use entity_component_system::*;

// Re-export serialization macros
pub use crate::save_world;
pub use crate::load_world;
