#[cfg(target_arch = "wasm32")]
use web_sys::{KeyboardEvent, MouseEvent};
use std::any::Any;
use std::collections::HashSet;
use std::cell::RefCell;
use super::eventTypes::{Event, EventType};
use super::eventTrait::NativeEventHandler;

#[cfg(target_arch = "wasm32")]
struct BrowserInputState {
    pressed_keys: HashSet<String>,
    last_mouse_pos: Option<(f64, f64)>,
    last_direction: String,
    cursor_locked: bool,
}

#[cfg(target_arch = "wasm32")]
impl BrowserInputState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            last_mouse_pos: None,
            last_direction: "idle".to_string(),
            cursor_locked: false,
        }
    }

    fn calculate_direction(&self) -> String {
        let w = self.pressed_keys.contains("KeyW");
        let a = self.pressed_keys.contains("KeyA");
        let s = self.pressed_keys.contains("KeyS");
        let d = self.pressed_keys.contains("KeyD");

        // Apply cancellation logic
        let forward = w && !s;
        let back = s && !w;
        let left = a && !d;
        let right = d && !a;

        match (forward, back, left, right) {
            (true, false, true, false) => "forward-left".to_string(),
            (true, false, false, true) => "forward-right".to_string(),
            (false, true, true, false) => "back-left".to_string(),
            (false, true, false, true) => "back-right".to_string(),
            (true, false, false, false) => "forward".to_string(),
            (false, true, false, false) => "back".to_string(),
            (false, false, true, false) => "left".to_string(),
            (false, false, false, true) => "right".to_string(),
            _ => "idle".to_string(),
        }
    }

    fn mouse_delta_to_euler(delta_x: f64, delta_y: f64) -> [f32; 2] {
        let sensitivity = 0.002;
        let yaw_delta = (delta_x * sensitivity) as f32;
        let pitch_delta = (delta_y * sensitivity) as f32;

        // Return pitch and yaw deltas directly
        [pitch_delta, yaw_delta]
    }
}

#[cfg(target_arch = "wasm32")]
pub struct BrowserEventHandler {
    input_state: RefCell<BrowserInputState>,
}

#[cfg(target_arch = "wasm32")]
impl BrowserEventHandler {
    pub fn new() -> Self {
        Self {
            input_state: RefCell::new(BrowserInputState::new()),
        }
    }

    pub fn set_cursor_locked(&self, locked: bool) {
        self.input_state.borrow_mut().cursor_locked = locked;
    }
}

#[cfg(target_arch = "wasm32")]
impl NativeEventHandler for BrowserEventHandler {
    fn parse_keyboard_event(&self, event: &dyn Any) -> Option<Event> {
        if let Some(ke) = event.downcast_ref::<KeyboardEvent>() {
            let key_code = ke.code();
            
            match key_code.as_str() {
                "KeyW" | "KeyA" | "KeyS" | "KeyD" => {
                    let mut state = self.input_state.borrow_mut();
                    
                    // Determine if this is a keydown or keyup event based on event type
                    let event_type = ke.type_();
                    match event_type.as_str() {
                        "keydown" => {
                            state.pressed_keys.insert(key_code);
                        }
                        "keyup" => {
                            state.pressed_keys.remove(&key_code);
                        }
                        _ => return None,
                    }

                    let new_direction = state.calculate_direction();
                    state.last_direction = new_direction.clone();

                    return Some(Event {
                        event_type: EventType::Move,
                        payload: Box::new(new_direction),
                    });
                }
                _ => {}
            }
        }
        None
    }

    fn parse_mouse_event(&self, event: &dyn Any) -> Option<Event> {
        if let Some(me) = event.downcast_ref::<MouseEvent>() {
            let state = self.input_state.borrow();
            
            if state.cursor_locked {
                // Use movement deltas when locked
                let delta_x = me.movement_x() as f64;
                let delta_y = me.movement_y() as f64;

                // Send event for any movement when locked
                if delta_x != 0.0 || delta_y != 0.0 {
                    let euler_deltas = BrowserInputState::mouse_delta_to_euler(delta_x, delta_y);
                    return Some(Event {
                        event_type: EventType::RotateCamera,
                        payload: Box::new(euler_deltas),
                    });
                }
            } else {
                // Use position deltas when not locked
                let current_pos = (me.client_x() as f64, me.client_y() as f64);
                drop(state); // Release borrow before mutable borrow
                let mut state = self.input_state.borrow_mut();

                if let Some(last_pos) = state.last_mouse_pos {
                    let delta_x = current_pos.0 - last_pos.0;
                    let delta_y = current_pos.1 - last_pos.1;

                    // Only send event if there's significant movement
                    if delta_x.abs() > 1.0 || delta_y.abs() > 1.0 {
                        let euler_deltas = BrowserInputState::mouse_delta_to_euler(delta_x, delta_y);
                        state.last_mouse_pos = Some(current_pos);
                        return Some(Event {
                            event_type: EventType::RotateCamera,
                            payload: Box::new(euler_deltas),
                        });
                    }
                }

                state.last_mouse_pos = Some(current_pos);
            }
        }
        None
    }
}
