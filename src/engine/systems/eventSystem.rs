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

pub trait EventListener: Send + Sync {
    fn update(&self, event: &Event);
}

static EVENT_SYSTEM: OnceLock<EventSystem> = OnceLock::new();

pub struct EventSystem {
    subscribers: DashMap<EventType, Vec<Arc<dyn EventListener>>>,
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

    pub fn subscribe(event_type: EventType, listener: Arc<dyn EventListener>) {
        let instance = Self::instance();
        instance.subscribers.entry(event_type).or_insert_with(Vec::new).push(listener);
    }

    pub fn unsubscribe(event_type: EventType) {
        let instance = Self::instance();
        instance.subscribers.remove(&event_type);
    }

    pub fn notify(event: Event) {
        let instance = Self::instance();

        if let Some(listeners) = instance.subscribers.get(&event.event_type) {
            for listener in listeners.iter() {
                listener.update(&event);
            }
        }
    }
}

impl std::fmt::Debug for EventSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSystem").field("subscribers_count", &self.subscribers.len()).finish()
    }
}
