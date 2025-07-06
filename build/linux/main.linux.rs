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
use winit::event::{WindowEvent, KeyEvent, ElementState};
use winit::event_loop::{ ActiveEventLoop, EventLoop };
use winit::window::{ Window, WindowId };
use winit::keyboard::{KeyCode, PhysicalKey};

mod index;
use index::{Program, Event, EventType};

// Platform-specific shader source functions for Linux/Native
#[no_mangle]
pub fn get_vertex_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/vertex_animated.glsl");
    source.replace("#VERSION", "#version 460 core")
}

#[no_mangle]
pub fn get_fragment_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/fragment_animated.glsl");
    source.replace("#VERSION", "#version 460 core")
}

#[no_mangle]
pub fn get_static_vertex_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/vertex_static.glsl");
    source.replace("#VERSION", "#version 460 core")
}

#[no_mangle]
pub fn get_static_fragment_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/fragment_static.glsl");
    source.replace("#VERSION", "#version 460 core")
}

struct InputState {
    pressed_keys: HashSet<KeyCode>,
    last_mouse_pos: Option<(f64, f64)>,
    last_direction: String,
}

impl InputState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            last_mouse_pos: None,
            last_direction: "idle".to_string(),
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

    fn mouse_delta_to_quaternion(delta_x: f64, delta_y: f64) -> [f32; 4] {
        let sensitivity = 0.002;
        let yaw = (delta_x * sensitivity) as f32;
        let pitch = (delta_y * sensitivity) as f32;

        // Create quaternion from yaw and pitch
        let cos_yaw = (yaw * 0.5).cos();
        let sin_yaw = (yaw * 0.5).sin();
        let cos_pitch = (pitch * 0.5).cos();
        let sin_pitch = (pitch * 0.5).sin();

        // Combine yaw and pitch quaternions
        [
            cos_yaw * cos_pitch,
            sin_yaw * cos_pitch,
            cos_yaw * sin_pitch,
            -sin_yaw * sin_pitch,
        ]
    }
}

struct App {
    window: Option<Window>,
    gl_context: Option<glutin::context::PossiblyCurrentContext>,
    gl_surface: Option<glutin::surface::Surface<WindowSurface>>,
    program: Option<Program>,
    start_time: Option<Instant>,
    last_frame_time: Option<Instant>,
    input_state: InputState,
}

impl App {
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
                    
                    let new_direction = self.input_state.calculate_direction();
                    if new_direction != self.input_state.last_direction {
                        self.input_state.last_direction = new_direction.clone();
                        
                        let event = Event {
                            event_type: EventType::Move,
                            payload: Box::new(new_direction),
                        };
                        
                        if let Some(program) = &mut self.program {
                            program.receive_event(&event);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_movement(&mut self, x: f64, y: f64) {
        let current_pos = (x, y);
        
        if let Some(last_pos) = self.input_state.last_mouse_pos {
            let delta_x = current_pos.0 - last_pos.0;
            let delta_y = current_pos.1 - last_pos.1;
            
            // Only send event if there's significant movement
            if delta_x.abs() > 1.0 || delta_y.abs() > 1.0 {
                let quaternion = InputState::mouse_delta_to_quaternion(delta_x, delta_y);
                
                let event = Event {
                    event_type: EventType::RotateCamera,
                    payload: Box::new(quaternion),
                };
                
                if let Some(program) = &mut self.program {
                    program.receive_event(&event);
                }
            }
        }
        
        self.input_state.last_mouse_pos = Some(current_pos);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes =
                Window::default_attributes().with_title("Native OpenGL Triangle");

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

            let attrs = SurfaceAttributesBuilder::<WindowSurface>
                ::new()
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

            let program = Program::new(gl).expect("Failed to create graphics program");

            // Initialize timing
            let now = Instant::now();
            self.start_time = Some(now);
            self.last_frame_time = Some(now);

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
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if
                    let (Some(surface), Some(context), Some(program)) = (
                        &self.gl_surface,
                        &self.gl_context,
                        &mut self.program,
                    )
                {
                    if let Some(window) = &self.window {
                        let current_time = Instant::now();
                        
                        // Calculate elapsed time since start
                        let elapsed_time = if let Some(start_time) = self.start_time {
                            current_time.duration_since(start_time).as_secs_f32()
                        } else {
                            0.0
                        };
                        
                        // Update last frame time
                        self.last_frame_time = Some(current_time);
                        
                        let size = window.inner_size();
                        program.render(size.width, size.height, elapsed_time)
                            .expect("Failed to render triangle");
                    }

                    surface.swap_buffers(context).unwrap();
                    
                    // Request continuous rendering
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::Resized(_) => {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    let mut app = App {
        window: None,
        gl_context: None,
        gl_surface: None,
        program: None,
        start_time: None,
        last_frame_time: None,
        input_state: InputState::new(),
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
