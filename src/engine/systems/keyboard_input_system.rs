use std::collections::HashSet;
use std::sync::Mutex;
use winit::keyboard::KeyCode;
use winit::event::ElementState;
use slint::winit_030::{ WinitWindowAccessor, winit };
use crate::index::engine::systems::event_system::{ Event, EventSystem, EventType };

pub struct KeyboardInputSystem {
    pressed_keys: Mutex<HashSet<KeyCode>>,
    is_locked: Mutex<bool>,
}

impl KeyboardInputSystem {
    pub fn new() -> Self {
        Self {
            pressed_keys: Mutex::new(HashSet::new()),
            is_locked: Mutex::new(false),
        }
    }

    /// Public method: Receive and process keyboard events
    pub fn receive_key_event(
        &self,
        key_event: &winit::event::KeyEvent,
        slint_window: &slint::Window
    ) {
        if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
            match key_event.state {
                ElementState::Pressed => {
                    match key_code {
                        KeyCode::Tab => {
                            // Toggle cursor lock on Tab press
                            let mut is_locked = self.is_locked.lock().unwrap();
                            *is_locked = !*is_locked;
                            let locked = *is_locked;
                            drop(is_locked); // Release lock before window operations

                            println!("[INPUT] Cursor lock toggled: {}", locked);

                            // Handle cursor visibility and grabbing
                            slint_window.with_winit_window(|winit_window| {
                                if locked {
                                    // Lock cursor: hide it and grab it to the window
                                    println!("[FPS] Locking cursor (entering FPS mode)...");
                                    winit_window.set_cursor_visible(false);

                                    // Try confined first, if not supported then try locked
                                    let grab_result = winit_window
                                        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                        .or_else(|_|
                                            winit_window.set_cursor_grab(
                                                winit::window::CursorGrabMode::Locked
                                            )
                                        );

                                    if grab_result.is_err() {
                                        println!(
                                            "[FPS] Warning: Cursor grab not supported on this platform"
                                        );
                                    }

                                    // Center the cursor in the window
                                    let size = winit_window.inner_size();
                                    let center_pos = winit::dpi::PhysicalPosition::new(
                                        (size.width as f64) / 2.0,
                                        (size.height as f64) / 2.0
                                    );
                                    if let Err(e) = winit_window.set_cursor_position(center_pos) {
                                        println!("[FPS] Warning: Could not center cursor: {}", e);
                                    }
                                    println!("[FPS] Cursor grabbed and hidden.");
                                } else {
                                    // Unlock cursor: release grab and show cursor
                                    println!("[FPS] Unlocking cursor (exiting FPS mode)...");
                                    let _ = winit_window.set_cursor_grab(
                                        winit::window::CursorGrabMode::None
                                    );
                                    winit_window.set_cursor_visible(true);
                                    println!("[FPS] Cursor released and visible.");
                                }
                            });
                        }
                        KeyCode::Escape => {
                            // Unlock cursor on Escape press
                            let mut is_locked = self.is_locked.lock().unwrap();
                            if *is_locked {
                                *is_locked = false;
                                drop(is_locked); // Release lock before window operations

                                println!("[INPUT] Cursor unlocked via Escape");

                                // Release cursor
                                slint_window.with_winit_window(|winit_window| {
                                    println!("[FPS] Unlocking cursor (exiting FPS mode)...");
                                    let _ = winit_window.set_cursor_grab(
                                        winit::window::CursorGrabMode::None
                                    );
                                    winit_window.set_cursor_visible(true);
                                    println!("[FPS] Cursor released and visible.");
                                });
                            }
                        }
                        _ => {
                            // Handle regular keys for movement
                            let mut pressed_keys = self.pressed_keys.lock().unwrap();
                            pressed_keys.insert(key_code);
                            println!("[INPUT] Key pressed: {:?}", key_code);
                        }
                    }
                }
                ElementState::Released => {
                    // Only track release for movement keys (not Tab/Escape)
                    match key_code {
                        KeyCode::Tab | KeyCode::Escape => {
                            // Don't track Tab/Escape releases
                        }
                        _ => {
                            let mut pressed_keys = self.pressed_keys.lock().unwrap();
                            pressed_keys.remove(&key_code);
                            println!("[INPUT] Key released: {:?}", key_code);
                        }
                    }
                }
            }
        }
    }

    /// Public method: Receive and process mouse movement events
    pub fn receive_mouse_event(
        &self,
        position: &winit::dpi::PhysicalPosition<f64>,
        slint_window: &slint::Window
    ) {
        let is_locked = *self.is_locked.lock().unwrap();

        if is_locked {
            // Enhanced mouse handling when cursor is locked
            slint_window.with_winit_window(|winit_window| {
                let size = winit_window.inner_size();
                let center_x = (size.width as f64) / 2.0;
                let center_y = (size.height as f64) / 2.0;

                // Calculate delta from current window center
                let delta_x = position.x - center_x;
                let delta_y = position.y - center_y;

                // Only process significant movement
                if delta_x.abs() > 2.0 || delta_y.abs() > 2.0 {
                    // Generate camera rotation event with proper delta
                    let euler_deltas =
                        crate::index::engine::utils::input_utils::mouse_delta_to_euler(
                            delta_x,
                            delta_y
                        );
                    let camera_event = Event {
                        event_type: EventType::RotateCamera,
                        payload: Box::new(euler_deltas),
                    };
                    EventSystem::notify(camera_event);
                }

                // Always reset cursor to current window center
                let center_pos = winit::dpi::PhysicalPosition::new(center_x, center_y);
                let _ = winit_window.set_cursor_position(center_pos);
            });
        }
    }

    /// Public method: Update called each frame
    pub fn update(&self) {
        let direction = self.calculate_direction();

        // Only send movement event if there's actual movement
        if !direction.is_empty() {
            let move_event = Event {
                event_type: EventType::Move,
                payload: Box::new(direction),
            };
            EventSystem::notify(move_event);
        }
    }

    /// Private method: Calculate movement direction from pressed keys
    fn calculate_direction(&self) -> String {
        let pressed_keys = self.pressed_keys.lock().unwrap();

        let mut w = pressed_keys.contains(&KeyCode::KeyW);
        let mut a = pressed_keys.contains(&KeyCode::KeyA);
        let mut s = pressed_keys.contains(&KeyCode::KeyS);
        let mut d = pressed_keys.contains(&KeyCode::KeyD);
        let mut e = pressed_keys.contains(&KeyCode::KeyE);
        let mut q = pressed_keys.contains(&KeyCode::KeyQ);

        if w && s {
            w = false;
            s = false;
        }

        if d && a {
            d = false;
            a = false;
        }

        if e && q {
            e = false;
            q = false;
        }

        // Build direction string with all dimensions
        let mut directions = Vec::new();
        if w { directions.push("forward"); }
        if s { directions.push("backward"); }
        if a { directions.push("left"); }
        if d { directions.push("right"); }
        if e { directions.push("up"); }
        if q { directions.push("down"); }

        directions.join("-")
    }
}
