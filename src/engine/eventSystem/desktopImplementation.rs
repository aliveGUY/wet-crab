#[cfg(not(target_arch = "wasm32"))]
use winit::event::{ KeyEvent, ElementState };
#[cfg(not(target_arch = "wasm32"))]
use winit::keyboard::{ KeyCode, PhysicalKey };
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;
use std::any::Any;
use std::collections::HashSet;
use std::sync::RwLock;
use super::eventTypes::{ Event, EventType };
use super::eventTrait::NativeEventHandler;
use crate::index::engine::utils::inputUtils::{ calculate_movement_direction, mouse_delta_to_euler };

#[cfg(not(target_arch = "wasm32"))]
pub struct DesktopEventHandler {
    pressed_keys: RwLock<HashSet<KeyCode>>,
    last_mouse_pos: RwLock<Option<(f64, f64)>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl DesktopEventHandler {
    pub fn new() -> Self {
        Self {
            pressed_keys: RwLock::new(HashSet::new()),
            last_mouse_pos: RwLock::new(None),
        }
    }
    
    fn extract_key_event<'a>(&self, event: &'a dyn Any) -> Option<&'a KeyEvent> {
        event.downcast_ref::<KeyEvent>()
    }
    
    fn get_key_code(&self, key_event: &KeyEvent) -> Option<KeyCode> {
        if let PhysicalKey::Code(key_code) = key_event.physical_key {
            return Some(key_code);
        }
        None
    }
    
    fn is_movement_key(&self, key_code: KeyCode) -> bool {
        matches!(key_code, KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD)
    }
    
    fn is_pressed_state(&self, key_event: &KeyEvent) -> bool {
        key_event.state == ElementState::Pressed
    }
    
    fn update_key_state(&self, key_code: KeyCode, is_pressed: bool) {
        let mut keys = self.pressed_keys.write().unwrap();
        if is_pressed {
            keys.insert(key_code);
        } else {
            keys.remove(&key_code);
        }
    }
    
    fn calculate_current_direction(&self) -> String {
        let keys = self.pressed_keys.read().unwrap();
        let w = keys.contains(&KeyCode::KeyW);
        let a = keys.contains(&KeyCode::KeyA);
        let s = keys.contains(&KeyCode::KeyS);
        let d = keys.contains(&KeyCode::KeyD);
        calculate_movement_direction(w, a, s, d)
    }
    
    fn process_key_input(&self, key_code: KeyCode, is_pressed: bool) -> Option<Event> {
        self.update_key_state(key_code, is_pressed);
        let direction = self.calculate_current_direction();
        Some(Event {
            event_type: EventType::Move,
            payload: Box::new(direction),
        })
    }
    
    fn extract_mouse_position(&self, event: &dyn Any) -> Option<(f64, f64)> {
        let position = event.downcast_ref::<PhysicalPosition<f64>>()?;
        Some((position.x, position.y))
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

#[cfg(not(target_arch = "wasm32"))]
impl NativeEventHandler for DesktopEventHandler {
    fn parse_keyboard_event(&self, event: &dyn Any) -> Option<Event> {
        let key_event = self.extract_key_event(event)?;
        let key_code = self.get_key_code(key_event)?;
        
        if !self.is_movement_key(key_code) {
            return None;
        }
        
        let is_pressed = self.is_pressed_state(key_event);
        self.process_key_input(key_code, is_pressed)
    }

    fn parse_mouse_event(&self, event: &dyn Any) -> Option<Event> {
        let position = self.extract_mouse_position(event)?;
        let delta = self.calculate_mouse_delta(position)?;
        
        if !self.is_significant_movement(delta) {
            return None;
        }
        
        self.create_camera_event(delta)
    }
}
