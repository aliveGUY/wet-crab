use std::any::Any;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum EventType {
    Move,
    RotateCamera,
}

pub struct Event {
    pub event_type: EventType,
    pub payload: Box<dyn Any>,
}

pub trait EventListener {
    fn update(&self, event: &Event);
}

pub struct EventSystem {
    subscribers: HashMap<EventType, Vec<Box<dyn EventListener>>>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, event_type: EventType, listener: Box<dyn EventListener>) {
        self.subscribers.entry(event_type).or_default().push(listener);
    }

    pub fn notify(&self, event: &Event) {
        if let Some(listeners) = self.subscribers.get(&event.event_type) {
            for listener in listeners {
                listener.update(event);
            }
        }
    }
}
