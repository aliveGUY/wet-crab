pub mod eventSystem;
pub mod inputSystem;

// Re-export the main types for easy access
pub use eventSystem::{ EventSystem, Event, EventType, EventListener };
pub use inputSystem::{ InputSystem, InputHandler, DesktopInputHandler, BrowserInputHandler };
