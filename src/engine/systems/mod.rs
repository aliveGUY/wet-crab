pub mod eventSystem;
pub mod inputSystem;
pub mod interfaceSystem;
#[macro_use]
pub mod entityComponentSystem;

// Re-export the main types for easy access
pub use eventSystem::{ EventSystem, Event, EventType };
pub use inputSystem::{ InputSystem, InputHandler, DesktopInputHandler, BrowserInputHandler };
pub use interfaceSystem::{ InterfaceSystem };
pub use entityComponentSystem::*;
