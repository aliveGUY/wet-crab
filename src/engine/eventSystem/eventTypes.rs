use std::any::Any;

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
    pub payload: Box<dyn Any>,
}

pub trait EventListener {
    fn update(&self, event: &Event);
}
