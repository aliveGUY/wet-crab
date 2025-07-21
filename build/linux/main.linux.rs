use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ ContextApi, ContextAttributesBuilder, Version };
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{ SurfaceAttributesBuilder, WindowSurface };
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;
use std::time::Instant;
use std::collections::HashSet;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, KeyEvent, ElementState, DeviceEvent, DeviceId};
use winit::event_loop::{ ActiveEventLoop, EventLoop };
use winit::window::{ Window, WindowId, CursorGrabMode };
use winit::keyboard::{KeyCode, PhysicalKey};

mod index;
use index::{ Program };
use index::engine::systems::{ EventSystem, InputSystem, DesktopInputHandler };

struct InputState {
    pressed_keys: HashSet<KeyCode>,
    last_mouse_pos: Option<(f64, f64)>,
    cursor_locked: bool,
}

impl InputState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            last_mouse_pos: None,
            cursor_locked: false,
        }
    }

    fn calculate_movement_direction(&self) -> String {
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
            (false, true, true, false) => "backward-left".to_string(),
            (false, true, false, true) => "backward-right".to_string(),
            (true, false, false, false) => "forward".to_string(),
            (false, true, false, false) => "backward".to_string(),
            (false, false, true, false) => "left".to_string(),
            (false, false, false, true) => "right".to_string(),
            _ => "".to_string(),
        }
    }
}

struct App {
    window: Option<Window>,
    gl_context: Option<glutin::context::PossiblyCurrentContext>,
    gl_surface: Option<glutin::surface::Surface<WindowSurface>>,
    program: Option<Program>,
    start_time: Option<Instant>,
    input_state: InputState,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            gl_context: None,
            gl_surface: None,
            program: None,
            start_time: None,
            input_state: InputState::new(),
        }
    }

    fn handle_keyboard_input(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            match key_code {
                KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD => {
                    match event.state {
                        ElementState::Pressed => {
                            self.input_state.pressed_keys.insert(key_code);
                        }
                        ElementState::Released => {
                            self.input_state.pressed_keys.remove(&key_code);
                        }
                    }
                    
                    // Send movement direction to InputSystem
                    let movement_direction = self.input_state.calculate_movement_direction();
                    if !movement_direction.is_empty() {
                        InputSystem::instance().receive_key_event(&movement_direction);
                    }
                }
                KeyCode::Escape => {
                    if event.state == ElementState::Pressed && self.input_state.cursor_locked {
                        if let Some(window) = &self.window {
                            let _ = window.set_cursor_grab(CursorGrabMode::None);
                            window.set_cursor_visible(true);
                            self.input_state.cursor_locked = false;
                            self.input_state.last_mouse_pos = None;
                            println!("🔓 Cursor unlocked via Escape key");
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_movement(&mut self, x: f64, y: f64) {
        if !self.input_state.cursor_locked {
            // Only handle absolute movement when unlocked
            if let Some(last_pos) = self.input_state.last_mouse_pos {
                let delta_x = x - last_pos.0;
                let delta_y = y - last_pos.1;
                
                if delta_x.abs() > 1.0 || delta_y.abs() > 1.0 {
                    let delta = (delta_x as i32, delta_y as i32);
                    InputSystem::instance().receive_mouse_event(&delta);
                }
            }
            self.input_state.last_mouse_pos = Some((x, y));
        }
        // When locked, ignore CursorMoved events - use DeviceEvent instead for FPS-style input
    }

    fn handle_raw_mouse_movement(&mut self, delta: (f64, f64)) {
        // FPS-style raw mouse input - works infinitely beyond window boundaries
        if self.input_state.cursor_locked {
            let (delta_x, delta_y) = delta;
            if delta_x.abs() > 0.1 || delta_y.abs() > 0.1 {
                let delta = (delta_x as i32, delta_y as i32);
                InputSystem::instance().receive_mouse_event(&delta);
            }
        }
    }

    fn handle_mouse_click(&mut self) {
        if !self.input_state.cursor_locked {
            if let Some(window) = &self.window {
                // Center cursor in window before locking
                let window_size = window.inner_size();
                let center_x = window_size.width as f64 / 2.0;
                let center_y = window_size.height as f64 / 2.0;
                
                // Set cursor to window center
                let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(center_x, center_y));
                
                // Confine cursor to window and hide it completely
                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                window.set_cursor_visible(false);
                
                self.input_state.cursor_locked = true;
                self.input_state.last_mouse_pos = None;
                println!("🔒 Cursor confined and hidden via left mouse click");
            }
        }
        // Ignore mouse clicks when already locked to prevent accidental unlocking
    }

    fn recenter_cursor_if_needed(&mut self) {
        // Continuously re-center cursor when locked to prevent edge-hitting
        if self.input_state.cursor_locked {
            if let Some(window) = &self.window {
                let window_size = window.inner_size();
                let center_x = window_size.width as f64 / 2.0;
                let center_y = window_size.height as f64 / 2.0;
                
                // Re-center cursor to prevent it from reaching window edges
                let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(center_x, center_y));
            }
        }
    }

    fn process_continuous_input(&mut self) {
        // Continuous keyboard processing for immediate response (like SDL2 polling)
        let movement_direction = self.input_state.calculate_movement_direction();
        if !movement_direction.is_empty() {
            InputSystem::instance().receive_key_event(&movement_direction);
        }
    }
}

impl ApplicationHandler for App {
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                // FPS-style raw mouse input for infinite camera rotation
                self.handle_raw_mouse_movement(delta);
            }
            _ => {}
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Runst POC - Linux (winit)")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600));

            let window = event_loop.create_window(window_attributes).unwrap();

            let display_builder = DisplayBuilder::new();

            let (_, gl_config) = display_builder
                .build(event_loop, ConfigTemplateBuilder::new(), |mut configs| {
                    configs.next().unwrap()
                })
                .unwrap();

            let raw_display = gl_config.display();

            let context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
                .build(Some(window.window_handle().unwrap().as_raw()));

            let not_current_gl_context = unsafe {
                raw_display.create_context(&gl_config, &context_attributes).unwrap()
            };

            let attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
                .build(
                    window.window_handle().unwrap().as_raw(),
                    NonZeroU32::new(800).unwrap(),
                    NonZeroU32::new(600).unwrap()
                );

            let surface = unsafe { raw_display.create_window_surface(&gl_config, &attrs).unwrap() };

            let gl_context = not_current_gl_context.make_current(&surface).unwrap();

            let gl = unsafe {
                glow::Context::from_loader_function(|s| {
                    raw_display.get_proc_address(&std::ffi::CString::new(s).unwrap()) as *const _
                })
            };

            // Initialize clean systems architecture
            EventSystem::initialize();
            InputSystem::initialize(Box::new(DesktopInputHandler::new()));

            let program = Program::new(gl).expect("Failed to create program");

            // Initialize timing
            let now = Instant::now();
            self.start_time = Some(now);

            println!("🎮 winit event handling started - concurrent input ready!");
            println!("📝 Instructions:");
            println!("   - Hold WASD keys for movement");
            println!("   - Move mouse for camera rotation");
            println!("   - Left click to lock cursor");
            println!("   - Escape to unlock cursor");
            println!("   - Both inputs work simultaneously!");

            window.request_redraw();

            self.window = Some(window);
            self.gl_context = Some(gl_context);
            self.gl_surface = Some(surface);
            self.program = Some(program);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("🏁 winit demo finished!");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Process continuous input every frame (like SDL2 polling)
                self.process_continuous_input();
                
                // Re-center cursor periodically to prevent edge-hitting
                self.recenter_cursor_if_needed();
                
                if let (Some(surface), Some(context), Some(program)) = (
                    &self.gl_surface,
                    &self.gl_context,
                    &mut self.program,
                ) {
                    if let Some(window) = &self.window {
                        let current_time = Instant::now();
                        
                        // Calculate elapsed time since start
                        let elapsed_time = if let Some(start_time) = self.start_time {
                            current_time.duration_since(start_time).as_secs_f32()
                        } else {
                            0.0
                        };
                        
                        let size = window.inner_size();
                        if let Err(e) = program.render(size.width, size.height, elapsed_time) {
                            eprintln!("Render error: {}", e);
                            event_loop.exit();
                        }
                    }

                    surface.swap_buffers(context).unwrap();
                    
                    // Request continuous rendering
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                println!("Window resized: {}x{}", new_size.width, new_size.height);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_mouse_movement(position.x, position.y);
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: winit::event::MouseButton::Left, .. } => {
                self.handle_mouse_click();
            }
            _ => (),
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(program) = &self.program {
            program.cleanup();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
