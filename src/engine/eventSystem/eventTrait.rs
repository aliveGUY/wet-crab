use std::any::Any;
use super::eventTypes::Event;

pub trait NativeEventHandler: Send + Sync {
    fn parse_keyboard_event(&self, event: &dyn Any) -> Option<Event>;
    fn parse_mouse_event(&self, event: &dyn Any) -> Option<Event>;
    fn parse_mouse_click_event(&self, event: &dyn Any) -> Option<Event>;
    fn should_process_mouse_movement(&self) -> bool;
}
