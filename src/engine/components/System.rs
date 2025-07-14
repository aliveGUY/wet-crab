// Import Event type from parent scope
use crate::index::engine::systems::eventSystem::Event;

pub trait System: Send + Sync {
    fn event(&self, event: &Event) {
        // Default implementation - do nothing
    }

    fn update(&self) {
        // Default implementation - do nothing
    }
}
