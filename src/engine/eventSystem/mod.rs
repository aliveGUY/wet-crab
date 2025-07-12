pub mod eventTypes;
pub mod eventTrait;
pub mod desktopImplementation;
pub mod browserImplementation;
pub mod globalEventSystem;

pub use eventTypes::*;
pub use eventTrait::*;
pub use globalEventSystem::*;

#[cfg(not(target_arch = "wasm32"))]
pub use desktopImplementation::DesktopEventHandler;

#[cfg(target_arch = "wasm32")]
pub use browserImplementation::BrowserEventHandler;
