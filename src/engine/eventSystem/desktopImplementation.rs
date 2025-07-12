#[cfg(not(target_arch = "wasm32"))]
use winit::event::{MouseScrollDelta, ElementState, KeyEvent, DeviceEvent};
#[cfg(not(target_arch = "wasm32"))]
use winit::keyboard::{KeyCode, PhysicalKey};
use std::any::Any;
use std::collections::HashSet;
use std::cell::RefCell;
use super::eventTypes::{Event, EventType};
use super::eventTrait::NativeEventHandler;

#[cfg(not(target_arch = "wasm32"))]
struct DesktopInputState {
    pressed_keys: HashSet<KeyCode>,
    last_mouse_pos: Option<(f64, f64)>,
    last_direction: String,
    cursor_locked: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl DesktopInputState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            last_mouse_pos: None,
            last_direction: "idle".to_string(),
            cursor_locked: false,
        }
    }

    fn calculate_direction(&self) -> String {
        let w = self.pressed_keys.contains(&KeyCode::KeyW);
        let a = self.pressed_keys.contains(&KeyCode::KeyA);
        let s = self.pressed_keys.contains(&KeyCode::KeyS);
        let d = self.pressed_keys.contains(&KeyCode::KeyD);

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
        }
        None
    }

    fn parse_mouse_event(&self, event: &dyn Any) -> Option<Event> {
        // Handle scroll wheel events
        if let Some(delta) = event.downcast_ref::<MouseScrollDelta>() {
            return Some(Event {
                event_type: EventType::RotateCamera,
                payload: Box::new(delta.clone()),
            });
        }

        // Handle mouse motion events (for cursor locked mode)
        if let Some(device_event) = event.downcast_ref::<DeviceEvent>() {
            if let DeviceEvent::MouseMotion { delta } = device_event {
                let state = self.input_state.borrow();
                if state.cursor_locked {
                    let euler_deltas = DesktopInputState::mouse_delta_to_euler(delta.0, delta.1);
                    return Some(Event {
                        event_type: EventType::RotateCamera,
                        payload: Box::new(euler_deltas),
                    });
                }
            }
        }

        // Handle cursor position events (for unlocked mode)
        if let Some((x, y)) = event.downcast_ref::<(f64, f64)>() {
            let mut state = self.input_state.borrow_mut();
            if !state.cursor_locked {
                let current_pos = (*x, *y);

                if let Some(last_pos) = state.last_mouse_pos {
                    let delta_x = current_pos.0 - last_pos.0;
                    let delta_y = current_pos.1 - last_pos.1;

                    // Only send event if there's significant movement
                    if delta_x.abs() > 1.0 || delta_y.abs() > 1.0 {
                        let euler_deltas = DesktopInputState::mouse_delta_to_euler(delta_x, delta_y);
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
