#[cfg(target_arch = "wasm32")]
use web_sys::{ KeyboardEvent, MouseEvent };
use std::any::Any;
use std::collections::HashSet;
use std::cell::RefCell;
use super::eventTypes::{ Event, EventType };
use super::eventTrait::NativeEventHandler;
use crate::index::engine::utils::inputUtils::{ calculate_movement_direction, mouse_delta_to_euler };

#[cfg(target_arch = "wasm32")]
struct BrowserInputState {
    pressed_keys: HashSet<String>,
    last_mouse_pos: Option<(f64, f64)>,
    last_direction: String,
}

#[cfg(target_arch = "wasm32")]
impl BrowserInputState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            last_mouse_pos: None,
            last_direction: "idle".to_string(),
        }
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
}

#[cfg(target_arch = "wasm32")]
impl NativeEventHandler for BrowserEventHandler {
    fn parse_keyboard_event(&self, event: &dyn Any) -> Option<Event> {
        if let Some(ke) = event.downcast_ref::<KeyboardEvent>() {
            let key_code = ke.code();

            match key_code.as_str() {
                "KeyW" | "KeyA" | "KeyS" | "KeyD" => {
                    let mut state = self.input_state.borrow_mut();

                    let event_type = ke.type_();
                    match event_type.as_str() {
                        "keydown" => {
                            state.pressed_keys.insert(key_code);
                        }
                        "keyup" => {
                            state.pressed_keys.remove(&key_code);
                        }
                        _ => {
                            return None;
                        }
                    }

                    let w = state.pressed_keys.contains("KeyW");
                    let a = state.pressed_keys.contains("KeyA");
                    let s = state.pressed_keys.contains("KeyS");
                    let d = state.pressed_keys.contains("KeyD");

                    let new_direction = calculate_movement_direction(w, a, s, d);
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
            let current_pos = (me.client_x() as f64, me.client_y() as f64);
            let mut state = self.input_state.borrow_mut();

            if let Some(last_pos) = state.last_mouse_pos {
                let delta_x = current_pos.0 - last_pos.0;
                let delta_y = current_pos.1 - last_pos.1;

                if delta_x.abs() > 1.0 || delta_y.abs() > 1.0 {
                    let euler_deltas = mouse_delta_to_euler(delta_x, delta_y);

                    state.last_mouse_pos = Some(current_pos);

                    return Some(Event {
                        event_type: EventType::RotateCamera,
                        payload: Box::new(euler_deltas),
                    });
                }
            }

            state.last_mouse_pos = Some(current_pos);
        }
        None
    }
}
