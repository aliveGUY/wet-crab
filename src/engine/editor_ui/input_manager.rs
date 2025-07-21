//! Input Manager for Slint Integration
//! 
//! This module handles direct winit event subscription and bridges
//! game input events to both the existing InputSystem and Slint UI.

use std::collections::HashMap;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::event::{ElementState, MouseButton as WinitMouseButton};

/// Input state manager for hybrid Slint + Game input system
#[derive(Debug, Clone)]
pub struct SlintInputManager {
    // WASD state tracking
    wasd_keys: HashMap<KeyCode, bool>,
    last_wasd_state: String,
    
    // Mouse position tracking
    mouse_position: (f64, f64),
    
    // Frame counter for performance tracking
    frame_count: u64,
    last_fps_time: std::time::Instant,
    current_fps: u32,
}

impl SlintInputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            wasd_keys: HashMap::new(),
            last_wasd_state: "idle".to_string(),
            mouse_position: (0.0, 0.0),
            frame_count: 0,
            last_fps_time: std::time::Instant::now(),
            current_fps: 60,
        }
    }
    
    /// Process winit keyboard input and return whether to prevent Slint handling
    pub fn process_winit_keyboard(&mut self, physical_key: &PhysicalKey, state: ElementState) -> bool {
        if let PhysicalKey::Code(code) = physical_key {
            let pressed = state == ElementState::Pressed;
            
            match code {
                KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD => {
                    // Update WASD state
                    self.wasd_keys.insert(*code, pressed);
                    
                    // Calculate movement direction
                    let new_state = self.calculate_wasd_state();
                    if new_state != self.last_wasd_state {
                        self.last_wasd_state = new_state.clone();
                        println!("[SLINT-INPUT] WASD State: {} (Key: {:?}, Pressed: {})", 
                                new_state, code, pressed);
                    }
                    
                    // Prevent Slint handling for game movement keys
                    true
                }
                KeyCode::Escape => {
                    if pressed {
                        println!("[SLINT-INPUT] Escape key pressed - cursor unlock");
                    }
                    // Allow both game and Slint to handle escape
                    false
                }
                _ => {
                    // Let Slint handle other keys
                    false
                }
            }
        } else {
            false
        }
    }
    
    /// Process winit mouse movement
    pub fn process_mouse_movement(&mut self, x: f64, y: f64) {
        self.mouse_position = (x, y);
        println!("[SLINT-INPUT] Mouse Position: ({:.1}, {:.1})", x, y);
    }
    
    /// Process winit mouse button input
    pub fn process_mouse_button(&mut self, button: WinitMouseButton, state: ElementState) -> bool {
        let pressed = state == ElementState::Pressed;
        
        match button {
            WinitMouseButton::Left => {
                if pressed {
                    println!("[SLINT-INPUT] Left mouse button pressed - cursor lock attempt");
                }
                // Allow both game and Slint to handle left click
                false
            }
            WinitMouseButton::Right => {
                if pressed {
                    println!("[SLINT-INPUT] Right mouse button pressed");
                }
                // Allow both to handle right click
                false
            }
            _ => {
                if pressed {
                    println!("[SLINT-INPUT] Other mouse button pressed: {:?}", button);
                }
                false
            }
        }
    }
    
    /// Update frame-based calculations (call once per frame)
    pub fn update_frame(&mut self) {
        self.frame_count += 1;
        
        // Calculate FPS every second
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_fps_time);
        if elapsed.as_secs() >= 1 {
            self.current_fps = (self.frame_count as f64 / elapsed.as_secs_f64()) as u32;
            self.frame_count = 0;
            self.last_fps_time = now;
        }
    }
    
    /// Get current WASD state as string
    pub fn get_wasd_state(&self) -> String {
        self.last_wasd_state.clone()
    }
    
    /// Get current mouse position as formatted string
    pub fn get_mouse_position_string(&self) -> String {
        format!("({:.0}, {:.0})", self.mouse_position.0, self.mouse_position.1)
    }
    
    /// Get current FPS
    pub fn get_fps(&self) -> u32 {
        self.current_fps
    }
    
    /// Calculate WASD movement state
    fn calculate_wasd_state(&self) -> String {
        let w = self.wasd_keys.get(&KeyCode::KeyW).copied().unwrap_or(false);
        let a = self.wasd_keys.get(&KeyCode::KeyA).copied().unwrap_or(false);
        let s = self.wasd_keys.get(&KeyCode::KeyS).copied().unwrap_or(false);
        let d = self.wasd_keys.get(&KeyCode::KeyD).copied().unwrap_or(false);
        
        // Apply cancellation logic
        let forward = w && !s;
        let back = s && !w;
        let left = a && !d;
        let right = d && !a;
        
        match (forward, back, left, right) {
            (true, false, true, false) => "forward-left".to_string(),
            (true, false, false, true) => "forward-right".to_string(),
            (false, true, true, false) => "backward-left".to_string(),
            (false, true, false, true) => "backward-right".to_string(),
            (true, false, false, false) => "forward".to_string(),
            (false, true, false, false) => "backward".to_string(),
            (false, false, true, false) => "left".to_string(),
            (false, false, false, true) => "right".to_string(),
            _ => "idle".to_string(),
        }
    }
}

impl Default for SlintInputManager {
    fn default() -> Self {
        Self::new()
    }
}
