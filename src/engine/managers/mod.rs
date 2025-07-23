pub mod assets_manager;

// Re-export for convenience
pub use assets_manager::AssetsManager;

// Re-export commonly used types
pub use assets_manager::{initialize_asset_manager, get_static_object_copy, get_animated_object_copy, Assets};
