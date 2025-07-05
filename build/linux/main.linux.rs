use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ ContextApi, ContextAttributesBuilder, Version };
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{ SurfaceAttributesBuilder, WindowSurface };
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ ActiveEventLoop, EventLoop };
use winit::window::{ Window, WindowId };

mod index;
use index::Program;

// Platform-specific shader source functions for Linux/Native
#[no_mangle]
pub fn get_vertex_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/vertex.glsl");
    source.replace("#VERSION", "#version 460 core")
}

#[no_mangle]
pub fn get_fragment_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/fragment.glsl");
    source.replace("#VERSION", "#version 460 core")
}

struct App {
    window: Option<Window>,
    gl_context: Option<glutin::context::PossiblyCurrentContext>,
    gl_surface: Option<glutin::surface::Surface<WindowSurface>>,
    program: Option<Program>,
    start_time: Option<Instant>,
    last_frame_time: Option<Instant>,
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
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
