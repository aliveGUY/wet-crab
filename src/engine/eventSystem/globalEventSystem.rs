use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;
use std::any::Any;
use super::eventTypes::{Event, EventType, EventListener};
use super::eventTrait::NativeEventHandler;

static GLOBAL_EVENT_SYSTEM: OnceLock<Mutex<GlobalEventSystem>> = OnceLock::new();

pub struct GlobalEventSystem {
    subscribers: HashMap<EventType, Vec<Box<dyn EventListener>>>,
    native_handler: Box<dyn NativeEventHandler>,
}

impl std::fmt::Debug for GlobalEventSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalEventSystem")
            .field("subscribers_count", &self.subscribers.len())
            .field("has_handler", &true)
            .finish()
    }
}

impl GlobalEventSystem {
    pub fn initialize(handler: Box<dyn NativeEventHandler>) {
        GLOBAL_EVENT_SYSTEM.set(Mutex::new(GlobalEventSystem {
            subscribers: HashMap::new(),
            native_handler: handler,
        })).unwrap();
    }
    
    pub fn instance() -> &'static Mutex<GlobalEventSystem> {
        GLOBAL_EVENT_SYSTEM.get().expect("GlobalEventSystem not initialized")
    }
    
    pub fn subscribe(event_type: EventType, listener: Box<dyn EventListener>) {
        let instance = Self::instance();
        let mut system = instance.lock().unwrap();
        system.subscribers.entry(event_type).or_default().push(listener);
    }
    
    pub fn notify(event: &Event) {
        let instance = Self::instance();
        let system = instance.lock().unwrap();
        
        let listeners = system.subscribers.get(&event.event_type);
        if listeners.is_none() {
            return;
        }
        
        for listener in listeners.unwrap() {
            listener.update(event);
        }
    }
    
    pub fn receive_native_keyboard_event(raw_event: &dyn Any) {
        let instance = Self::instance();
        let system = instance.lock().unwrap();
        let event = system.native_handler.parse_keyboard_event(raw_event);
        drop(system);
        
        if event.is_none() {
            return;
        }
        
        Self::notify(&event.unwrap());
    }
    
    pub fn receive_native_mouse_event(raw_event: &dyn Any) {
        let instance = Self::instance();
        let system = instance.lock().unwrap();
        let event = system.native_handler.parse_mouse_event(raw_event);
        drop(system);
        
        if event.is_none() {
            return;
        }
        
        Self::notify(&event.unwrap());
    }
}
