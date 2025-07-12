#[cfg(target_arch = "wasm32")]
use web_sys::{ KeyboardEvent, MouseEvent };
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
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
    is_cursor_locked: RwLock<bool>,
}

#[cfg(target_arch = "wasm32")]
impl BrowserEventHandler {
    pub fn new() -> Self {
        Self {
            pressed_keys: RwLock::new(HashSet::new()),
            last_mouse_pos: RwLock::new(None),
            is_cursor_locked: RwLock::new(false),
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
    
    fn is_escape_key(&self, key_code: &str) -> bool {
        key_code == "Escape"
    }
    
    fn handle_escape_key(&self) -> Option<Event> {
        let is_locked = *self.is_cursor_locked.read().unwrap();
        if is_locked {
            *self.is_cursor_locked.write().unwrap() = false;
            // Exit pointer lock
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            document.exit_pointer_lock();
            println!("Cursor unlocked via Escape key");
        }
        None
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
        
        // Handle Escape key for cursor unlocking
        if self.is_escape_key(&key_code) {
            let is_pressed = self.determine_key_state(keyboard_event)?;
            if is_pressed {
                return self.handle_escape_key();
            }
        }
        
        if !self.is_movement_key(&key_code) {
            return None;
        }
        
        let is_pressed = self.determine_key_state(keyboard_event)?;
        self.process_key_input(&key_code, is_pressed)
    }

    fn parse_mouse_event(&self, event: &dyn Any) -> Option<Event> {
        let mouse_event = self.extract_mouse_event(event)?;
        
        // Check if pointer is actually locked using browser API
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let is_actually_locked = document.pointer_lock_element().is_some();
        
        if !is_actually_locked {
            return None; // Don't process mouse movement when not locked
        }
        
        // When pointer is locked, use movement deltas instead of position
        let delta = (mouse_event.movement_x() as f64, mouse_event.movement_y() as f64);
        
        if !self.is_significant_movement(delta) {
            return None;
        }
        
        self.create_camera_event(delta)
    }
    
    fn parse_mouse_click_event(&self, event: &dyn Any) -> Option<Event> {
        let mouse_event = self.extract_mouse_event(event)?;
        
        // Handle left mouse button click (button 0)
        if mouse_event.button() == 0 {
            let is_locked = *self.is_cursor_locked.read().unwrap();
            if !is_locked {
                *self.is_cursor_locked.write().unwrap() = true;
                // Request pointer lock on canvas
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                if let Some(canvas) = document.get_element_by_id("webgl-canvas") {
                    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
                    canvas.request_pointer_lock();
                    println!("Cursor locked via left mouse click");
                }
            }
        }
        
        None
    }
    
    fn should_process_mouse_movement(&self) -> bool {
        // Check if pointer is actually locked using browser API
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document.pointer_lock_element().is_some()
    }
}
