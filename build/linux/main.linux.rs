use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use glow::HasContext;
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Window>,
    gl_context: Option<glutin::context::PossiblyCurrentContext>,
    gl_surface: Option<glutin::surface::Surface<WindowSurface>>,
    gl: Option<glow::Context>,
    shader_program: Option<glow::Program>,
    vao: Option<glow::VertexArray>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Native OpenGL Example");
            
            let window = event_loop.create_window(window_attributes).unwrap();
            
            // Create display builder
            let display_builder = DisplayBuilder::new();
            
            // Build the display
            let (_, gl_config) = display_builder
                .build(event_loop, ConfigTemplateBuilder::new(), |mut configs| {
                    configs.next().unwrap()
                })
                .unwrap();

            let raw_display = gl_config.display();

            // Create context
            let context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
                .build(Some(window.window_handle().unwrap().as_raw()));

            let not_current_gl_context = unsafe {
                raw_display
                    .create_context(&gl_config, &context_attributes)
                    .unwrap()
            };

            // Create surface
            let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
                window.window_handle().unwrap().as_raw(),
                NonZeroU32::new(800).unwrap(),
                NonZeroU32::new(600).unwrap(),
            );

            let surface = unsafe {
                raw_display.create_window_surface(&gl_config, &attrs).unwrap()
            };

            // Make context current
            let gl_context = not_current_gl_context.make_current(&surface).unwrap();

            // Load OpenGL functions
            let gl = unsafe {
                glow::Context::from_loader_function(|s| {
                    raw_display.get_proc_address(&std::ffi::CString::new(s).unwrap()) as *const _
                })
            };

            // Setup triangle rendering
            let (shader_program, vao) = unsafe {
                // Load shader sources
                let vertex_shader_source = include_str!("assets/vertex.glsl");
                let fragment_shader_source = include_str!("assets/fragment.glsl");

                // Create vertex shader
                let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
                gl.shader_source(vertex_shader, vertex_shader_source);
                gl.compile_shader(vertex_shader);

                if !gl.get_shader_compile_status(vertex_shader) {
                    let error = gl.get_shader_info_log(vertex_shader);
                    panic!("Vertex shader compilation failed: {}", error);
                }

                // Create fragment shader
                let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
                gl.shader_source(fragment_shader, fragment_shader_source);
                gl.compile_shader(fragment_shader);

                if !gl.get_shader_compile_status(fragment_shader) {
                    let error = gl.get_shader_info_log(fragment_shader);
                    panic!("Fragment shader compilation failed: {}", error);
                }

                // Create shader program
                let program = gl.create_program().unwrap();
                gl.attach_shader(program, vertex_shader);
                gl.attach_shader(program, fragment_shader);
                gl.link_program(program);

                if !gl.get_program_link_status(program) {
                    let error = gl.get_program_info_log(program);
                    panic!("Program linking failed: {}", error);
                }

                // Clean up shaders (they're now linked into the program)
                gl.delete_shader(vertex_shader);
                gl.delete_shader(fragment_shader);

                // Create VAO
                let vao = gl.create_vertex_array().unwrap();
                gl.bind_vertex_array(Some(vao));

                // Set viewport
                let size = window.inner_size();
                gl.viewport(0, 0, size.width as i32, size.height as i32);

                (program, vao)
            };

            // Request initial redraw
            window.request_redraw();
            
            self.window = Some(window);
            self.gl_context = Some(gl_context);
            self.gl_surface = Some(surface);
            self.gl = Some(gl);
            self.shader_program = Some(shader_program);
            self.vao = Some(vao);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(gl), Some(surface), Some(context), Some(program), Some(vao)) = 
                    (&self.gl, &self.gl_surface, &self.gl_context, &self.shader_program, &self.vao) {
                    unsafe {
                        // Clear the screen
                        gl.clear_color(0.1, 0.2, 0.3, 1.0);
                        gl.clear(glow::COLOR_BUFFER_BIT);

                        // Use shader program and VAO
                        gl.use_program(Some(*program));
                        gl.bind_vertex_array(Some(*vao));

                        // Draw the triangle
                        gl.draw_arrays(glow::TRIANGLES, 0, 3);

                        // Clean up
                        gl.bind_vertex_array(None);
                        gl.use_program(None);
                    }
                    surface.swap_buffers(context).unwrap();
                }
            }
            _ => (),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    
    let mut app = App {
        window: None,
        gl_context: None,
        gl_surface: None,
        gl: None,
        shader_program: None,
        vao: None,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
