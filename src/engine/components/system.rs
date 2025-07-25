// Import Event type from parent scope
use crate::index::engine::systems::event_system::Event;

pub trait SystemTrait: Send + Sync {
    fn event(&self, _event: &Event) {
        // Default implementation - do nothing
    }

    #[allow(dead_code)]
    fn update() where Self: Sized {
        // Default static implementation - do nothing
        // This method can now be called statically: MySystem::update()
        // The 'where Self: Sized' constraint makes this trait dyn-compatible
    }
}
