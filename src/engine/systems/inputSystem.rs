use std::sync::{ Arc, OnceLock };
use std::any::Any;
use std::collections::HashMap;
use crate::index::engine::systems::eventSystem::{ Event, EventSystem, EventType };

// Import Winit types for enhanced DesktopInputHandler
use winit::event::{WindowEvent, ElementState};
use winit::keyboard::{KeyCode, PhysicalKey};

pub trait InputHandler: Send + Sync {
    fn receive_mouse_event(&self, raw_event: &dyn Any) -> Option<Event>;
    fn receive_key_event(&self, raw_event: &dyn Any) -> Option<Event>;
}

static INPUT_SYSTEM: OnceLock<InputSystem> = OnceLock::new();

pub struct InputSystem {
    handler: Arc<dyn InputHandler>,
}

impl InputSystem {
    pub fn initialize(handler: Box<dyn InputHandler>) {
        INPUT_SYSTEM.set(InputSystem {
            handler: Arc::from(handler),
        }).expect("InputSystem already initialized");
    }

    pub fn instance() -> &'static InputSystem {
        INPUT_SYSTEM.get().expect("InputSystem not initialized")
    }

    pub fn receive_mouse_event(&self, raw_event: &dyn Any) {
        if let Some(event) = self.handler.receive_mouse_event(raw_event) {
            EventSystem::notify(event);
        }
    }

    pub fn receive_key_event(&self, raw_event: &dyn Any) {
        if let Some(event) = self.handler.receive_key_event(raw_event) {
            EventSystem::notify(event);
        }
    }
}

use std::sync::Mutex;
use std::collections::HashSet;

pub struct DesktopInputHandler {
    // WASD state tracking for Winit events (for multi-key combinations)
    wasd_keys: Mutex<HashMap<KeyCode, bool>>,
    // Track currently pressed keys for continuous movement
    pressed_keys: Mutex<HashSet<KeyCode>>,
    // FPS cursor lock system
    cursor_locked: Mutex<bool>,
    // Track previous mouse position for proper FPS delta calculation
    last_mouse_position: Mutex<Option<(f64, f64)>>,
}

impl DesktopInputHandler {
    pub fn new() -> Self {
        Self {
            wasd_keys: Mutex::new(HashMap::new()),
            pressed_keys: Mutex::new(HashSet::new()),
            cursor_locked: Mutex::new(false),
            last_mouse_position: Mutex::new(None),
        }
    }
    
    /// Get current cursor lock state
    pub fn is_cursor_locked(&self) -> bool {
        self.cursor_locked.lock().map(|locked| *locked).unwrap_or(false)
    }
    
    /// Toggle cursor lock state
    pub fn toggle_cursor_lock(&self) {
        if let Ok(mut cursor_locked) = self.cursor_locked.lock() {
            *cursor_locked = !*cursor_locked;
            println!("[INPUT] Cursor lock toggled: {}", if *cursor_locked { "LOCKED" } else { "UNLOCKED" });
        }
    }
    
    /// Set cursor lock state directly
    pub fn set_cursor_locked(&self, locked: bool) {
        if let Ok(mut cursor_locked) = self.cursor_locked.lock() {
            *cursor_locked = locked;
            println!("[INPUT] Cursor lock set to: {}", if locked { "LOCKED" } else { "UNLOCKED" });
        }
    }
    
    /// Get currently pressed WASD keys for continuous movement dispatch
    pub fn get_pressed_keys(&self) -> HashSet<KeyCode> {
        self.pressed_keys.lock().map(|keys| keys.clone()).unwrap_or_else(|_| HashSet::new())
    }
    
    /// Process cursor movement for FPS camera - delta calculation will be done in main.linux.rs
    /// This method now just generates camera rotation events from any significant movement
    fn process_cursor_moved(&self, position: &winit::dpi::PhysicalPosition<f64>) -> Option<Event> {
        let cursor_locked = self.cursor_locked.lock().ok()?.clone();
        if !cursor_locked {
            return None;
        }
        
        // Note: Delta calculation from window center is now handled in main.linux.rs
        // This method receives the position and generates rotation events
        // The actual center calculation and cursor reset happens in the event handler
        
        // For now, we'll use a simple approach - any movement generates a rotation event
        // The main event handler will calculate proper delta from current window center
        let euler_deltas = crate::index::engine::utils::input_utils::mouse_delta_to_euler(
            position.x, position.y
        );
        
        return Some(Event {
            event_type: EventType::RotateCamera,
            payload: Box::new(euler_deltas),
        });
    }
    
    /// Process mouse button clicks (no cursor lock functionality)
    fn process_mouse_button(&self, button: winit::event::MouseButton, state: ElementState) -> Option<Event> {
        // Mouse buttons no longer handle cursor lock - just pass through
        None
    }
    
    /// Process keyboard input with flattened structure
    fn process_keyboard_input(&self, key_event: &winit::event::KeyEvent) -> Option<Event> {
        let PhysicalKey::Code(key_code) = key_event.physical_key else {
            return None;
        };
        
        let pressed = key_event.state == ElementState::Pressed;
        
        match key_code {
            KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD => {
                self.process_wasd_key(key_code, pressed)
            }
            KeyCode::Tab => {
                self.process_tab_key(pressed)
            }
            KeyCode::Escape => {
                self.process_escape_key(pressed)
            }
            _ => {
                if pressed {
                    println!("[INPUT] Other key pressed: {:?}", key_code);
                }
                None
            }
        }
    }
    
    /// Process WASD keys with responsive movement (only send events when movement changes)
    fn process_wasd_key(&self, key_code: KeyCode, pressed: bool) -> Option<Event> {
        // Update both the old wasd_keys HashMap and new pressed_keys HashSet
        let mut wasd_keys = self.wasd_keys.lock().ok()?;
        wasd_keys.insert(key_code, pressed);
        
        // Update pressed_keys HashSet for continuous movement
        let mut pressed_keys = self.pressed_keys.lock().ok()?;
        if pressed {
            pressed_keys.insert(key_code);
        } else {
            pressed_keys.remove(&key_code);
        }
        
        let w = wasd_keys.get(&KeyCode::KeyW).copied().unwrap_or(false);
        let a = wasd_keys.get(&KeyCode::KeyA).copied().unwrap_or(false);
        let s = wasd_keys.get(&KeyCode::KeyS).copied().unwrap_or(false);
        let d = wasd_keys.get(&KeyCode::KeyD).copied().unwrap_or(false);
        
        // Calculate new direction using input_utils function
        let new_direction = crate::index::engine::utils::input_utils::calculate_movement_direction(w, a, s, d);
        
        // For immediate response, always send an event when keys change
        println!("[INPUT] WASD Key {:?} {} -> Movement: {}", 
                key_code, if pressed { "pressed" } else { "released" }, new_direction);
        
        return Some(Event {
            event_type: EventType::Move,
            payload: Box::new(new_direction),
        });
    }
    
    /// Process Tab key for cursor lock toggle
    fn process_tab_key(&self, pressed: bool) -> Option<Event> {
        if !pressed {
            return None;
        }
        
        // Toggle cursor lock on Tab press
        if let Ok(mut cursor_locked) = self.cursor_locked.lock() {
            *cursor_locked = !*cursor_locked;
            
            // Reset last mouse position when toggling lock to prevent jumps
            if let Ok(mut last_pos) = self.last_mouse_position.lock() {
                *last_pos = None;
            }
            
            println!("[INPUT] Cursor lock toggled: {}", if *cursor_locked { "LOCKED" } else { "UNLOCKED" });
        }
        
        None
    }
    
    /// Process escape key for cursor unlock
    fn process_escape_key(&self, pressed: bool) -> Option<Event> {
        if !pressed {
            return None;
        }
        
        // Unlock cursor on escape
        if let Ok(mut cursor_locked) = self.cursor_locked.lock() {
            if *cursor_locked {
                *cursor_locked = false;
                println!("[INPUT] Cursor unlocked via Escape");
            }
        }
        
        None
    }
}

impl InputHandler for DesktopInputHandler {
    fn receive_mouse_event(&self, raw_event: &dyn Any) -> Option<Event> {
        // Handle Winit WindowEvent with flattened structure
        if let Some(window_event) = raw_event.downcast_ref::<WindowEvent>() {
            return match window_event {
                WindowEvent::CursorMoved { position, .. } => self.process_cursor_moved(position),
                WindowEvent::MouseInput { state, button, .. } => self.process_mouse_button(*button, *state),
                _ => None,
            };
        }
        
        // Keep backward compatibility with (i32, i32) mouse events
        if let Some(position) = raw_event.downcast_ref::<(i32, i32)>() {
            let euler_deltas = crate::index::engine::utils::input_utils::mouse_delta_to_euler(
                position.0 as f64,
                position.1 as f64
            );

            return Some(Event {
                event_type: EventType::RotateCamera,
                payload: Box::new(euler_deltas),
            });
        }

        None
    }

    fn receive_key_event(&self, raw_event: &dyn Any) -> Option<Event> {
        // Handle Winit KeyEvent with flattened structure
        if let Some(keyboard_event) = raw_event.downcast_ref::<winit::event::KeyEvent>() {
            return self.process_keyboard_input(keyboard_event);
        }
        
        // Keep backward compatibility with String direction events
        if let Some(direction) = raw_event.downcast_ref::<String>() {
            return Some(Event {
                event_type: EventType::Move,
                payload: Box::new(direction.clone()),
            });
        }

        None
    }
}

pub struct BrowserInputHandler {}

impl BrowserInputHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputHandler for BrowserInputHandler {
    fn receive_mouse_event(&self, raw_event: &dyn Any) -> Option<Event> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(mouse_event) = raw_event.downcast_ref::<web_sys::MouseEvent>() {
                let movement_x = mouse_event.movement_x();
                let movement_y = mouse_event.movement_y();

                if movement_x != 0 || movement_y != 0 {
                    let euler_deltas =
                        crate::index::engine::utils::input_utils::mouse_delta_to_euler(
                            movement_x as f64,
                            movement_y as f64
                        );

                    return Some(Event {
                        event_type: EventType::RotateCamera,
                        payload: Box::new(euler_deltas),
                    });
                }
            }
        }

        None
    }

    fn receive_key_event(&self, raw_event: &dyn Any) -> Option<Event> {
        #[cfg(target_arch = "wasm32")]
        {
            // Handle String input from concurrent movement system (like DesktopInputHandler)
            if let Some(direction) = raw_event.downcast_ref::<String>() {
                return Some(Event {
                    event_type: EventType::Move,
                    payload: Box::new(direction.clone()),
                });
            }

            // Handle individual KeyboardEvent (legacy support)
            if let Some(key_event) = raw_event.downcast_ref::<web_sys::KeyboardEvent>() {
                let key_code = key_event.code();

                let direction = match key_code.as_str() {
                    "KeyW" => Some("forward"),
                    "KeyS" => Some("backward"),
                    "KeyA" => Some("left"),
                    "KeyD" => Some("right"),
                    "Space" => Some("up"),
                    "ShiftLeft" | "ShiftRight" => Some("down"),
                    _ => None,
                };

                if let Some(dir) = direction {
                    return Some(Event {
                        event_type: EventType::Move,
                        payload: Box::new(dir.to_string()),
                    });
                }
            }
        }

        None
    }
}

impl std::fmt::Debug for InputSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputSystem").field("has_handler", &true).finish()
    }
}
