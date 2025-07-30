//! Hybrid OpenGL + Slint Application for runst-poc
//!
//! This application demonstrates integration of:
//! - OpenGL 3D rendering (game underlay)
//! - Slint UI (debug overlay)
//! - Direct winit event subscription for hybrid input
//! - FPS-style camera controls with cursor locking

use slint::{ ComponentHandle, RenderingState, GraphicsAPI };
use slint::winit_030::{ WinitWindowAccessor, WinitWindowEventResult, winit };
use winit::event::WindowEvent;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;

// Import our game engine
mod index;
use index::{ Program };
use index::engine::systems::{ EventSystem, KeyboardInputSystem, InterfaceSystem };

slint::include_modules!();

fn create_glow_context(
    get_proc_address: &dyn Fn(&std::ffi::CStr) -> *const std::ffi::c_void
) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|s| {
            let symbol = std::ffi::CString::new(s).unwrap();
            get_proc_address(&symbol)
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[HYBRID] Starting runst-poc with Slint + OpenGL integration");

    // Ensure Winit backend is selected for Slint
    println!("[DEBUG] Selecting Winit backend for Slint...");
    slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap()))?;
    println!("[DEBUG] Winit backend selected");

    // Create Slint UI using Winit backend
    println!("[DEBUG] Creating Slint debug UI...");
    let ui_app = LevelEditorUI::new().expect("Failed to create Slint UI");
    println!("[DEBUG] Slint UI created successfully");

    InterfaceSystem::initialize(&ui_app);

    // Initialize systems
    EventSystem::initialize();
    let keyboard_input_system = Rc::new(KeyboardInputSystem::new());
    println!("[DEBUG] KeyboardInputSystem initialized");

    // Set up simplified event handling with KeyboardInputSystem
    println!("[DEBUG] Setting up simplified event handling...");
    let keyboard_system_for_events = keyboard_input_system.clone();
    ui_app.window().on_winit_window_event(move |slint_window, event| {
        match event {
            // Handle cursor movement for camera rotation
            WindowEvent::CursorMoved { position, .. } => {
                keyboard_system_for_events.receive_mouse_event(position, slint_window);
                WinitWindowEventResult::Propagate
            }
            // Handle keyboard input for movement
            WindowEvent::KeyboardInput { event: keyboard_event, .. } => {
                keyboard_system_for_events.receive_key_event(keyboard_event, slint_window);
                WinitWindowEventResult::Propagate
            }
            // Other events: no special handling
            _ => WinitWindowEventResult::Propagate,
        }
    });
    println!("[DEBUG] Simplified event handling enabled");

    // Set up rendering notifier for OpenGL underlay
    let game_program = Rc::new(RefCell::new(None::<Program>));
    let game_program_for_callback = game_program.clone();
    let start_time = Rc::new(RefCell::new(None::<Instant>));
    let start_time_for_callback = start_time.clone();

    // Get weak references for different callbacks
    let ui_app_weak_for_rendering = ui_app.as_weak();
    let ui_app_weak_for_animation = ui_app.as_weak();
    let ui_window = ui_app.window();

    println!("[DEBUG] Setting up OpenGL rendering notifier...");
    ui_window
        .set_rendering_notifier({
            move |state, graphics_api| {
                match state {
                    RenderingState::RenderingSetup => {
                        println!("[UNDERLAY] OpenGL rendering setup for game engine");

                        if let GraphicsAPI::NativeOpenGL { get_proc_address } = graphics_api {
                            println!("[DEBUG] Loading OpenGL functions for game engine...");

                            // Initialize glow context
                            let gl = create_glow_context(get_proc_address);

                            // Initialize game program
                            println!("[DEBUG] Creating game program...");
                            match Program::new(gl) {
                                Ok(program) => {
                                    *game_program_for_callback.borrow_mut() = Some(program);
                                    *start_time_for_callback.borrow_mut() = Some(Instant::now());
                                    println!("[UNDERLAY] Game engine initialized successfully");
                                }
                                Err(e) => {
                                    eprintln!("[UNDERLAY] Failed to create game program: {}", e);
                                }
                            }
                        } else {
                            println!("[UNDERLAY] Non-OpenGL graphics API detected");
                        }
                    }
                    RenderingState::BeforeRendering => {
                        let start_time_borrow = start_time_for_callback.borrow();
                        let Some(start_time) = start_time_borrow.as_ref() else {
                            return;
                        };
                        let Some(app) = ui_app_weak_for_rendering.upgrade() else {
                            return;
                        };

                        let size = app.window().size();
                        let elapsed_time = start_time.elapsed().as_secs_f32();

                        let mut program_borrow = game_program_for_callback.borrow_mut();
                        if let Some(program) = program_borrow.as_mut() {
                            program.render(size.width as u32, size.height as u32, elapsed_time);
                        }
                    }

                    RenderingState::AfterRendering => {
                        // Nothing needed after UI rendering
                    }
                    RenderingState::RenderingTeardown => {
                        println!("[UNDERLAY] OpenGL rendering teardown");
                        *game_program_for_callback.borrow_mut() = None;
                        *start_time_for_callback.borrow_mut() = None;
                    }
                    _ => {}
                }
            }
        })
        .expect("Failed to set rendering notifier");

    // Set up UI callbacks
    println!("[DEBUG] Setting up UI callbacks...");

    // Set up animation timer with KeyboardInputSystem updates
    println!("[DEBUG] Setting up animation timer with KeyboardInputSystem updates...");
    let animation_timer = slint::Timer::default();
    let keyboard_system_for_timer = keyboard_input_system.clone();

    animation_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16), // ~60 FPS
        move || {
            // Update UI with current game state
            if let Some(app) = ui_app_weak_for_animation.upgrade() {
                // Request redraw for animation
                app.window().request_redraw();
            }

            // Call KeyboardInputSystem update method each frame
            keyboard_system_for_timer.update();
        }
    );

    println!("[HYBRID] Starting Slint event loop with game integration");
    println!("🎮 Game Controls:");
    println!("   - WASD: Movement (logged to console)");
    println!("   - Mouse: Camera look (position logged)");
    println!("   - Tab: Toggle cursor lock for FPS mode");
    println!("   - Escape: Unlock cursor");
    println!("📱 Debug UI: Real-time game state overlay");

    ui_app.run()?;

    Ok(())
}
