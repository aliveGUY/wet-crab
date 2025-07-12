use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ ContextApi, ContextAttributesBuilder, Version };
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{ SurfaceAttributesBuilder, WindowSurface };
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::cell::RefCell;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ ElementState, KeyEvent, WindowEvent };
use winit::event_loop::{ ActiveEventLoop, EventLoop };
use winit::keyboard::{ KeyCode, PhysicalKey };
use winit::window::{ Window, WindowId };

mod index;
use index::{ Program, GlobalEventSystem, DesktopEventHandler };

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
        if self.window.is_some() {
            return;
        }

        let window = event_loop
            .create_window(Window::default_attributes().with_title("Bridge-pattern GL demo"))
            .unwrap();

        let display_builder = DisplayBuilder::new();
        let (_, gl_config) = display_builder
            .build(event_loop, ConfigTemplateBuilder::new(), |mut c| c.next().unwrap())
            .unwrap();

        let display = gl_config.display();
        let ctx_attrs = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(window.window_handle().unwrap().as_raw()));

        let not_current = unsafe { display.create_context(&gl_config, &ctx_attrs).unwrap() };

        let attrs = SurfaceAttributesBuilder::<WindowSurface>
            ::new()
            .build(
                window.window_handle().unwrap().as_raw(),
                NonZeroU32::new(800).unwrap(),
                NonZeroU32::new(600).unwrap()
            );
        let surface = unsafe { display.create_window_surface(&gl_config, &attrs).unwrap() };
        let ctx = not_current.make_current(&surface).unwrap();

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                display.get_proc_address(&std::ffi::CString::new(s).unwrap()) as *const _
            })
        };

        let desktop_handler = Box::new(DesktopEventHandler::new());
        GlobalEventSystem::initialize(desktop_handler);

        let program = Program::new(gl).expect("Failed to create graphics program");

        let now = Instant::now();
        self.start_time = Some(now);
        self.last_frame_time = Some(now);

        window.request_redraw();

        self.window = Some(window);
        self.gl_context = Some(ctx);
        self.gl_surface = Some(surface);
        self.program = Some(program);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => {
                if
                    let (Some(surface), Some(ctx), Some(prog)) = (
                        &self.gl_surface,
                        &self.gl_context,
                        &mut self.program,
                    )
                {
                    if let Some(window) = &self.window {
                        let t_now = Instant::now();
                        let elapsed = self.start_time
                            .map(|s| (t_now - s).as_secs_f32())
                            .unwrap_or(0.0);
                        self.last_frame_time = Some(t_now);

                        let size = window.inner_size();
                        prog.render(size.width, size.height, elapsed).expect("render failed");
                    }
                    surface.swap_buffers(ctx).unwrap();

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::Resized(_) => {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                GlobalEventSystem::receive_native_keyboard_event(&event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                GlobalEventSystem::receive_native_mouse_event(&position);
            }

            _ => {}
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(p) = &self.program {
            p.cleanup();
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
