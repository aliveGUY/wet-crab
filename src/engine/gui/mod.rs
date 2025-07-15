pub mod bridge;
pub mod bridge_factory;
pub mod ui_manager;

#[cfg(target_arch = "wasm32")]
pub mod wasm_bridge;

#[cfg(not(target_arch = "wasm32"))]
pub mod native_bridge;

pub use bridge::PlatformBridge;
pub use bridge_factory::{create_platform_bridge, get_platform_info};
pub use ui_manager::UIManager;
