#[cfg(not(target_arch = "wasm32"))]
use winit::event::{ KeyEvent, ElementState };
#[cfg(not(target_arch = "wasm32"))]
use winit::keyboard::{ KeyCode, PhysicalKey };
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;
use std::any::Any;
use std::collections::HashSet;
use std::cell::RefCell;
use super::eventTypes::{ Event, EventType };
use super::eventTrait::NativeEventHandler;
use crate::index::engine::utils::inputUtils::{ calculate_movement_direction, mouse_delta_to_euler };

#[cfg(not(target_arch = "wasm32"))]
struct DesktopInputState {
    pressed_keys: HashSet<KeyCode>,
    last_mouse_pos: Option<(f64, f64)>,
    last_direction: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl DesktopInputState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            last_mouse_pos: None,
            last_direction: "idle".to_string(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct DesktopEventHandler {
    input_state: RefCell<DesktopInputState>,
}

#[cfg(not(target_arch = "wasm32"))]
impl DesktopEventHandler {
    pub fn new() -> Self {
        Self {
            input_state: RefCell::new(DesktopInputState::new()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeEventHandler for DesktopEventHandler {
    fn parse_keyboard_event(&self, event: &dyn Any) -> Option<Event> {
        if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
            if let PhysicalKey::Code(key_code) = key_event.physical_key {
                match key_code {
                    KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD => {
                        let mut state = self.input_state.borrow_mut();

                        match key_event.state {
                            ElementState::Pressed => {
                                state.pressed_keys.insert(key_code);
                            }
                            ElementState::Released => {
                                state.pressed_keys.remove(&key_code);
                            }
                        }

                        let w = state.pressed_keys.contains(&KeyCode::KeyW);
                        let a = state.pressed_keys.contains(&KeyCode::KeyA);
                        let s = state.pressed_keys.contains(&KeyCode::KeyS);
                        let d = state.pressed_keys.contains(&KeyCode::KeyD);

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
        }
        None
    }

    fn parse_mouse_event(&self, event: &dyn Any) -> Option<Event> {
        if let Some(position) = event.downcast_ref::<PhysicalPosition<f64>>() {
            let current_pos = (position.x, position.y);
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
