use std::collections::HashMap;
use std::any::Any;
use super::eventTypes::{Event, EventType, EventListener};
use super::eventTrait::NativeEventHandler;

pub struct EventSystem {
    subscribers: HashMap<EventType, Vec<Box<dyn EventListener>>>,
    native_handler: Box<dyn NativeEventHandler>,
}

impl EventSystem {
    pub fn new(native_handler: Box<dyn NativeEventHandler>) -> Self {
        Self {
            subscribers: HashMap::new(),
            native_handler,
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

    pub fn receive_native_keyboard_event(&self, event: &dyn Any) {
        if let Some(evt) = self.native_handler.parse_keyboard_event(event) {
            self.notify(&evt);
        }
    }

    pub fn receive_native_mouse_event(&self, event: &dyn Any) {
        if let Some(evt) = self.native_handler.parse_mouse_event(event) {
            self.notify(&evt);
        }
    }
}
