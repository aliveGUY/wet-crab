pub mod eventSystem;
pub mod inputSystem;
pub mod entityComponentSystem;

// Re-export the main types for easy access
pub use eventSystem::{ EventSystem, Event, EventType, EventListener };
pub use inputSystem::{ InputSystem, InputHandler, DesktopInputHandler, BrowserInputHandler };
pub use entityComponentSystem::*;
