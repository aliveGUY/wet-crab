pub mod event_system;
pub mod input_system;
pub mod interface_system;
#[macro_use]
pub mod entity_component_system;

// Re-export the main types for easy access
pub use event_system::{ EventSystem, EventType };
pub use input_system::{ InputSystem, DesktopInputHandler };
pub use interface_system::{ InterfaceSystem };
pub use entity_component_system::*;
