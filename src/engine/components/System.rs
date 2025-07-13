// Import Event type from parent scope
use crate::index::event_system::Event;

pub trait System: Send + Sync {
    fn handle_event(event: &Event) {
        // Default implementation - do nothing
    }

    fn update() {
        // Default implementation - do nothing
    }
}
