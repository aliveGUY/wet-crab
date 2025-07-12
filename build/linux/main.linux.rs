use std::time::Instant;
use sdl2::event::Event;
use sdl2::keyboard::{ Scancode, KeyboardState };
use sdl2::mouse::MouseButton;
use sdl2::video::GLProfile;
use glow::Context;

mod index;
use index::{ Program };
use index::engine::systems::{ EventSystem, InputSystem, DesktopInputHandler };

struct App {
    program: Program,
    start_time: Instant,
    cursor_locked: bool,
    last_mouse_pos: Option<(i32, i32)>,
}

impl App {
    fn new(gl: Context) -> Result<Self, String> {
        // Initialize clean systems architecture
        EventSystem::initialize();
        InputSystem::initialize(Box::new(DesktopInputHandler::new()));

        let program = Program::new(gl).map_err(|e| format!("Failed to create program: {}", e))?;

        Ok(App {
            program,
            start_time: Instant::now(),
            cursor_locked: false,
            last_mouse_pos: None,
        })
    }

    fn process_keyboard_state(&mut self, keyboard_state: &KeyboardState) {
        let mut movement_direction = String::new();

        if keyboard_state.is_scancode_pressed(Scancode::W) {
            movement_direction.push_str("forward");
        }
        if keyboard_state.is_scancode_pressed(Scancode::S) {
            if !movement_direction.is_empty() {
                movement_direction.push('-');
            }
            movement_direction.push_str("backward");
        }
        if keyboard_state.is_scancode_pressed(Scancode::A) {
            if !movement_direction.is_empty() {
                movement_direction.push('-');
            }
            movement_direction.push_str("left");
        }
        if keyboard_state.is_scancode_pressed(Scancode::D) {
            if !movement_direction.is_empty() {
                movement_direction.push('-');
            }
            movement_direction.push_str("right");
        }

        if !movement_direction.is_empty() {
            // Send to InputSystem - clean bridge pattern
            InputSystem::instance().receive_key_event(&movement_direction);
        }
    }

    fn handle_mouse_motion(&mut self, x: i32, y: i32, xrel: i32, yrel: i32) {
        if self.cursor_locked {
            if xrel != 0 || yrel != 0 {
                InputSystem::instance().receive_mouse_event(&(xrel, yrel));
            }
        } else {
            if let Some(last_pos) = self.last_mouse_pos {
                let delta = (x - last_pos.0, y - last_pos.1);
                if delta.0.abs() > 1 || delta.1.abs() > 1 {
                    InputSystem::instance().receive_mouse_event(&delta);
                }
            }
            self.last_mouse_pos = Some((x, y));
        }
    }

    fn render(
        &mut self,
        width: u32,
        height: u32,
        keyboard_state: &KeyboardState
    ) -> Result<(), String> {
        let elapsed = self.start_time.elapsed().as_secs_f32();

        self.process_keyboard_state(keyboard_state);

        self.program.render(width, height, elapsed).map_err(|e| format!("Render error: {}", e))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_double_buffer(true);
    gl_attr.set_depth_size(24);

    let window = video_subsystem
        .window("SDL2 Concurrent Input Demo", 800, 600)
        .opengl()
        .resizable()
        .build()?;

    let _gl_context = window.gl_create_context()?;
    let gl = unsafe {
        Context::from_loader_function(|s| video_subsystem.gl_get_proc_address(s) as *const _)
    };

    let mut app = App::new(gl)?;

    let mut event_pump = sdl_context.event_pump()?;

    println!("🎮 SDL2 event polling started - concurrent input ready!");
    println!("📝 Instructions:");
    println!("   - Hold WASD keys for movement");
    println!("   - Move mouse for camera rotation");
    println!("   - Left click to lock cursor");
    println!("   - Escape to unlock cursor");
    println!("   - Both inputs work simultaneously!");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }

                Event::KeyDown { scancode: Some(Scancode::Escape), repeat: false, .. } => {
                    if app.cursor_locked {
                        sdl_context.mouse().set_relative_mouse_mode(false);
                        sdl_context.mouse().show_cursor(true);
                        app.cursor_locked = false;
                        app.last_mouse_pos = None;
                        println!("🔓 Cursor unlocked via Escape key");
                    }
                }

                Event::MouseMotion { x, y, xrel, yrel, .. } => {
                    app.handle_mouse_motion(x, y, xrel, yrel);
                }

                Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                    if !app.cursor_locked {
                        sdl_context.mouse().set_relative_mouse_mode(true);
                        sdl_context.mouse().show_cursor(false);
                        app.cursor_locked = true;
                        app.last_mouse_pos = None;
                        println!("🔒 Cursor locked via left mouse click");
                    }
                }

                Event::Window { win_event, .. } => {
                    match win_event {
                        sdl2::event::WindowEvent::Resized(width, height) => {
                            println!("Window resized: {}x{}", width, height);
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        let keyboard_state = event_pump.keyboard_state();

        let (width, height) = window.size();
        if let Err(e) = app.render(width, height, &keyboard_state) {
            eprintln!("Render error: {}", e);
            break 'running;
        }

        window.gl_swap_window();

        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    println!("🏁 SDL2 demo finished!");
    Ok(())
}
