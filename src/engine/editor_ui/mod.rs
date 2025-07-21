//! Editor UI Module
//! 
//! This module provides Slint-based debug UI integration with OpenGL underlay
//! and direct winit event subscription for hybrid input handling.

pub mod input_manager;

pub use input_manager::SlintInputManager;

// Re-export for convenience
pub use slint;
