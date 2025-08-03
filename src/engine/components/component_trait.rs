use std::any::Any;

/// Simplified Component trait without UI dependencies
pub trait Component: Any + Send + Sync {
    // Components now only need to implement the Any trait
    // All serialization is handled by serde derives
}

// Blanket implementation for all types that are Any + Send + Sync
impl<T> Component for T where T: Any + Send + Sync {}
