use std::sync::{ Arc, OnceLock };
use std::any::Any;
use dashmap::DashMap;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum EventType {
    Move,
    RotateCamera,
    KeyboardDown,
    KeyboardUp,
    MouseMove,
}

pub struct Event {
    pub event_type: EventType,
    pub payload: Box<dyn Any + Send + Sync>,
}

// Import System trait from parent scope
use crate::index::engine::components::SystemTrait;

static EVENT_SYSTEM: OnceLock<EventSystem> = OnceLock::new();

pub struct EventSystem {
    subscribers: DashMap<EventType, Vec<Arc<dyn SystemTrait>>>,
}

impl EventSystem {
    pub fn initialize() {
        EVENT_SYSTEM.set(EventSystem {
            subscribers: DashMap::new(),
        }).expect("EventSystem already initialized");
    }

    pub fn instance() -> &'static EventSystem {
        EVENT_SYSTEM.get().expect("EventSystem not initialized")
    }

    pub fn subscribe(event_type: EventType, system: Arc<dyn SystemTrait>) {
        let instance = Self::instance();
        instance.subscribers.entry(event_type).or_insert_with(Vec::new).push(system);
    }

    pub fn unsubscribe(event_type: EventType) {
        let instance = Self::instance();
        instance.subscribers.remove(&event_type);
    }

    pub fn notify(event: Event) {
        let instance = Self::instance();

        if let Some(systems) = instance.subscribers.get(&event.event_type) {
            for system in systems.iter() {
                system.event(&event);
            }
        }
    }
}

impl std::fmt::Debug for EventSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSystem").field("subscribers_count", &self.subscribers.len()).finish()
    }
}
