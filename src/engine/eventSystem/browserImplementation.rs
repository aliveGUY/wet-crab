#[cfg(target_arch = "wasm32")]
use web_sys::{ KeyboardEvent, MouseEvent };
use std::any::Any;
use std::collections::HashSet;
use std::sync::RwLock;
use super::eventTypes::{ Event, EventType };
use super::eventTrait::NativeEventHandler;
use crate::index::engine::utils::inputUtils::{ calculate_movement_direction, mouse_delta_to_euler };

#[cfg(target_arch = "wasm32")]
pub struct BrowserEventHandler {
    pressed_keys: RwLock<HashSet<String>>,
    last_mouse_pos: RwLock<Option<(f64, f64)>>,
}

#[cfg(target_arch = "wasm32")]
impl BrowserEventHandler {
    pub fn new() -> Self {
        Self {
            pressed_keys: RwLock::new(HashSet::new()),
            last_mouse_pos: RwLock::new(None),
        }
    }
    
    fn extract_keyboard_event<'a>(&self, event: &'a dyn Any) -> Option<&'a KeyboardEvent> {
        event.downcast_ref::<KeyboardEvent>()
    }
    
    fn get_key_code(&self, event: &KeyboardEvent) -> String {
        event.code()
    }
    
    fn is_movement_key(&self, key_code: &str) -> bool {
        matches!(key_code, "KeyW" | "KeyA" | "KeyS" | "KeyD")
    }
    
    fn determine_key_state(&self, event: &KeyboardEvent) -> Option<bool> {
        let event_type = event.type_();
        
        if event_type == "keydown" {
            return Some(true);
        }
        
        if event_type == "keyup" {
            return Some(false);
        }
        
        None
    }
    
    fn update_key_state(&self, key_code: &str, is_pressed: bool) {
        let mut keys = self.pressed_keys.write().unwrap();
        if is_pressed {
            keys.insert(key_code.to_string());
        } else {
            keys.remove(key_code);
        }
    }
    
    fn calculate_current_direction(&self) -> String {
        let keys = self.pressed_keys.read().unwrap();
        let w = keys.contains("KeyW");
        let a = keys.contains("KeyA");
        let s = keys.contains("KeyS");
        let d = keys.contains("KeyD");
        calculate_movement_direction(w, a, s, d)
    }
    
    fn process_key_input(&self, key_code: &str, is_pressed: bool) -> Option<Event> {
        self.update_key_state(key_code, is_pressed);
        let direction = self.calculate_current_direction();
        Some(Event {
            event_type: EventType::Move,
            payload: Box::new(direction),
        })
    }
    
    fn extract_mouse_event<'a>(&self, event: &'a dyn Any) -> Option<&'a MouseEvent> {
        event.downcast_ref::<MouseEvent>()
    }
    
    fn get_mouse_position(&self, event: &MouseEvent) -> (f64, f64) {
        (event.client_x() as f64, event.client_y() as f64)
    }
    
    fn calculate_mouse_delta(&self, position: (f64, f64)) -> Option<(f64, f64)> {
        let mut last_pos = self.last_mouse_pos.write().unwrap();
        let last = *last_pos;
        *last_pos = Some(position);
        
        if last.is_none() {
            return None;
        }
        
        let last_position = last.unwrap();
        let delta = (position.0 - last_position.0, position.1 - last_position.1);
        Some(delta)
    }
    
    fn is_significant_movement(&self, delta: (f64, f64)) -> bool {
        delta.0.abs() > 1.0 || delta.1.abs() > 1.0
    }
    
    fn create_camera_event(&self, delta: (f64, f64)) -> Option<Event> {
        let euler_deltas = mouse_delta_to_euler(delta.0, delta.1);
        Some(Event {
            event_type: EventType::RotateCamera,
            payload: Box::new(euler_deltas),
        })
    }
}

#[cfg(target_arch = "wasm32")]
impl NativeEventHandler for BrowserEventHandler {
    fn parse_keyboard_event(&self, event: &dyn Any) -> Option<Event> {
        let keyboard_event = self.extract_keyboard_event(event)?;
        let key_code = self.get_key_code(keyboard_event);
        
        if !self.is_movement_key(&key_code) {
            return None;
        }
        
        let is_pressed = self.determine_key_state(keyboard_event)?;
        self.process_key_input(&key_code, is_pressed)
    }

    fn parse_mouse_event(&self, event: &dyn Any) -> Option<Event> {
        let mouse_event = self.extract_mouse_event(event)?;
        let position = self.get_mouse_position(mouse_event);
        let delta = self.calculate_mouse_delta(position)?;
        
        if !self.is_significant_movement(delta) {
            return None;
        }
        
        self.create_camera_event(delta)
    }
}
