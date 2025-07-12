pub mod eventTypes;
pub mod eventTrait;
pub mod eventSystem;
pub mod desktopImplementation;
pub mod browserImplementation;

// Re-export commonly used types
pub use eventTypes::{Event, EventType, EventListener};
pub use eventTrait::NativeEventHandler;
pub use eventSystem::EventSystem;

#[cfg(not(target_arch = "wasm32"))]
pub use desktopImplementation::DesktopEventHandler;

#[cfg(target_arch = "wasm32")]
pub use browserImplementation::BrowserEventHandler;
