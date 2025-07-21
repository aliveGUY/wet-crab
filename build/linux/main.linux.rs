//! Hybrid OpenGL + Slint Application for runst-poc
//!
//! This application demonstrates integration of:
//! - OpenGL 3D rendering (game underlay)
//! - Slint UI (debug overlay)
//! - Direct winit event subscription for hybrid input
//! - FPS-style camera controls with cursor locking

use slint::{ ComponentHandle, RenderingState, GraphicsAPI };
use slint::winit_030::{ WinitWindowAccessor, WinitWindowEventResult, winit };
use winit::event::{ WindowEvent, ElementState };
use winit::keyboard::{ PhysicalKey, KeyCode };
use winit::window::CursorGrabMode;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use glow::HasContext;

// Import our game engine
mod index;
use index::{ Program };
use index::engine::systems::{ EventSystem, InputSystem, DesktopInputHandler };

// Include the Slint UI from external file
slint::include_modules!();

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

    // Initialize input systems with shared state
    let desktop_handler = Rc::new(RefCell::new(DesktopInputHandler::new()));
    EventSystem::initialize();
    // Use a new DesktopInputHandler for the InputSystem (engine)
    // We'll manually keep the Rc instance in sync with this
    InputSystem::initialize(Box::new(DesktopInputHandler::new()));
    println!("[DEBUG] InputSystem initialized with DesktopInputHandler");
    
    // Set up Winit event handling for FPS controls and UI input
    println!("[DEBUG] Setting up clean event handling with synchronized cursor lock...");
    let desktop_handler_events = desktop_handler.clone();
    ui_app.window().on_winit_window_event(move |slint_window, event| {
        match event {
            // Mouse moved: if FPS mode is active, calculate delta from current window center and reset cursor
            WindowEvent::CursorMoved { position, .. } => {
                let locked = desktop_handler_events.borrow().is_cursor_locked();
                if locked {
                    // Calculate delta from current window center (not cached)
                    slint_window.with_winit_window(|winit_window| {
                        let size = winit_window.inner_size();
                        let center_x = size.width as f64 / 2.0;
                        let center_y = size.height as f64 / 2.0;
                        
                        // Calculate delta from current window center
                        let delta_x = position.x - center_x;
                        let delta_y = position.y - center_y;
                        
                        // Only process significant movement
                        if delta_x.abs() > 2.0 || delta_y.abs() > 2.0 {
                            // Generate camera rotation event with proper delta
                            let euler_deltas = crate::index::engine::utils::input_utils::mouse_delta_to_euler(delta_x, delta_y);
                            let camera_event = crate::index::engine::systems::eventSystem::Event {
                                event_type: crate::index::engine::systems::eventSystem::EventType::RotateCamera,
                                payload: Box::new(euler_deltas),
                            };
                            crate::index::engine::systems::eventSystem::EventSystem::notify(camera_event);
                        }
                        
                        // Always reset cursor to current window center
                        let center_pos = winit::dpi::PhysicalPosition::new(center_x, center_y);
                        let _ = winit_window.set_cursor_position(center_pos);
                    });
                    WinitWindowEventResult::PreventDefault  // consume event (don't let UI handle it)
                } else {
                    // When not locked, forward to InputSystem normally
                    InputSystem::instance().receive_mouse_event(event);
                    WinitWindowEventResult::Propagate  // not locked, allow Slint UI to process it
                }
            }
            // Mouse button input: always forward to InputSystem (no special handling needed here)
            WindowEvent::MouseInput { .. } => {
                InputSystem::instance().receive_mouse_event(event);
                WinitWindowEventResult::Propagate
            }
            // Keyboard input: handle FPS toggle keys (Tab, Escape) and movement keys (WASD)
            WindowEvent::KeyboardInput { event: keyboard_event, .. } => {
                if let PhysicalKey::Code(code) = keyboard_event.physical_key {
                    match code {
                        // Tab: Toggle cursor lock (FPS mode on/off)
                        KeyCode::Tab => {
                            if keyboard_event.state == ElementState::Pressed {
                                let was_locked = desktop_handler_events.borrow().is_cursor_locked();
                                // Inform the InputSystem about the key (it may toggle internal state)
                                InputSystem::instance().receive_key_event(keyboard_event);
                                // Manually toggle our shared DesktopInputHandler state
                                desktop_handler_events.borrow_mut().toggle_cursor_lock();
                                let is_locked = desktop_handler_events.borrow().is_cursor_locked();
                                // Apply cursor changes based on new state
                                slint_window.with_winit_window(|winit_window| {
                                    if !was_locked && is_locked {
                                        // **Locking cursor**: hide it and grab it to the window
                                        println!("[FPS] Locking cursor (entering FPS mode)...");
                                        winit_window.set_cursor_visible(false);
                                        // Try confined first, if not supported then try locked
                                        let grab_result = winit_window.set_cursor_grab(CursorGrabMode::Confined)
                                            .or_else(|_| winit_window.set_cursor_grab(CursorGrabMode::Locked));
                                        if grab_result.is_err() {
                                            println!("[FPS] Warning: Cursor grab not supported on this platform");
                                        }
                                        // Center the cursor in the window
                                        let size = winit_window.inner_size();
                                        let center_pos = winit::dpi::PhysicalPosition::new(
                                            size.width as f64 / 2.0, 
                                            size.height as f64 / 2.0
                                        );
                                        if let Err(e) = winit_window.set_cursor_position(center_pos) {
                                            println!("[FPS] Warning: Could not center cursor: {}", e);
                                        }
                                        println!("[FPS] Cursor grabbed and hidden.");
                                    } else if was_locked && !is_locked {
                                        // **Unlocking cursor**: release grab and show cursor
                                        println!("[FPS] Unlocking cursor (exiting FPS mode)...");
                                        let _ = winit_window.set_cursor_grab(CursorGrabMode::None);
                                        winit_window.set_cursor_visible(true);
                                        println!("[FPS] Cursor released and visible.");
                                    }
                                });
                            }
                            WinitWindowEventResult::PreventDefault  // consume Tab key
                        }
                        // Escape: Unlock cursor (if in FPS mode)
                        KeyCode::Escape => {
                            if keyboard_event.state == ElementState::Pressed {
                                let was_locked = desktop_handler_events.borrow().is_cursor_locked();
                                InputSystem::instance().receive_key_event(keyboard_event);
                                if was_locked {
                                    // Update our handler's state to unlocked
                                    desktop_handler_events.borrow_mut().set_cursor_locked(false);
                                }
                                // If we were locked and now unlocked, show cursor
                                if was_locked {
                                    slint_window.with_winit_window(|winit_window| {
                                        let _ = winit_window.set_cursor_grab(CursorGrabMode::None);
                                        winit_window.set_cursor_visible(true);
                                        println!("[FPS] Cursor unlocked via Escape.");
                                    });
                                }
                            }
                            WinitWindowEventResult::PreventDefault  // consume Escape key
                        }
                        // Other keys: forward to InputSystem, but prevent Slint from handling WASD (movement keys)
                        _ => {
                            InputSystem::instance().receive_key_event(keyboard_event);
                            match code {
                                KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD => {
                                    WinitWindowEventResult::PreventDefault  // game movement keys, don't propagate to UI
                                }
                                _ => WinitWindowEventResult::Propagate  // other keys can be handled by UI normally
                            }
                        }
                    }
                } else {
                    // Not a physical key code (e.g., a modifier or IME event) – let it propagate
                    WinitWindowEventResult::Propagate
                }
            }
            // Window resized: recenter cursor if locked (no caching needed)
            WindowEvent::Resized(new_size) => {
                // Fix camera rotation jitter: recenter cursor immediately if in FPS mode
                if desktop_handler_events.borrow().is_cursor_locked() {
                    slint_window.with_winit_window(|winit_window| {
                        let center_pos = winit::dpi::PhysicalPosition::new(
                            new_size.width as f64 / 2.0,
                            new_size.height as f64 / 2.0
                        );
                        if let Err(e) = winit_window.set_cursor_position(center_pos) {
                            println!("[FPS] Warning: Could not recenter cursor after resize: {}", e);
                        } else {
                            println!("[FPS] Cursor recentered after window resize to ({:.0}, {:.0})", 
                                   new_size.width as f64 / 2.0, new_size.height as f64 / 2.0);
                        }
                    });
                }
                
                WinitWindowEventResult::Propagate
            }
            // Window focus change: if focus is lost while in FPS mode, unlock the cursor for safety
            WindowEvent::Focused(focused) => {
                if !focused && desktop_handler_events.borrow().is_cursor_locked() {
                    slint_window.with_winit_window(|winit_window| {
                        let _ = winit_window.set_cursor_grab(CursorGrabMode::None);
                        winit_window.set_cursor_visible(true);
                    });
                    desktop_handler_events.borrow_mut().set_cursor_locked(false);
                    println!("[INPUT] Window lost focus -> cursor unlocked (for safety)");
                } else if *focused {
                    println!("[INPUT] Window gained focus");
                }
                WinitWindowEventResult::Propagate
            }
            // Other events: no special handling
            _ => WinitWindowEventResult::Propagate,
        }
    });
    println!("[DEBUG] FPS cursor lock system enabled");

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

                        // Initialize OpenGL function loading
                        match graphics_api {
                            GraphicsAPI::NativeOpenGL { get_proc_address } => {
                                println!("[DEBUG] Loading OpenGL functions for game engine...");

                                // Create glow context for our game engine
                                let gl = unsafe {
                                    glow::Context::from_loader_function(|s| {
                                        let symbol = std::ffi::CString::new(s).unwrap();
                                        get_proc_address(&symbol)
                                    })
                                };

                                // Verify OpenGL context and depth buffer configuration
                                unsafe {
                                    // Check OpenGL version and type
                                    let version_str = gl.get_parameter_string(glow::VERSION);
                                    println!("[DEBUG] OpenGL Version: {}", version_str);

                                    let renderer_str = gl.get_parameter_string(glow::RENDERER);
                                    println!("[DEBUG] OpenGL Renderer: {}", renderer_str);

                                    // Critical: Verify depth buffer exists
                                    let depth_bits = gl.get_parameter_i32(glow::DEPTH_BITS);
                                    if depth_bits == 0 {
                                        eprintln!(
                                            "[ERROR] No depth buffer allocated! Depth testing will not work."
                                        );
                                        eprintln!(
                                            "[ERROR] This will cause rear-facing polygons to render in front."
                                        );
                                    } else {
                                        println!("[DEBUG] Depth buffer: {} bits - depth testing available", depth_bits);
                                    }

                                    // Check other buffer configurations
                                    let color_bits =
                                        gl.get_parameter_i32(glow::RED_BITS) +
                                        gl.get_parameter_i32(glow::GREEN_BITS) +
                                        gl.get_parameter_i32(glow::BLUE_BITS) +
                                        gl.get_parameter_i32(glow::ALPHA_BITS);
                                    println!("[DEBUG] Color buffer: {} bits total", color_bits);

                                    let stencil_bits = gl.get_parameter_i32(glow::STENCIL_BITS);
                                    println!("[DEBUG] Stencil buffer: {} bits", stencil_bits);
                                }

                                // Initialize game engine systems
                                println!("[DEBUG] Initializing game engine systems...");
                                // Note: EventSystem and InputSystem are already initialized

                                // Create game program
                                println!("[DEBUG] Creating game program...");
                                match Program::new(gl) {
                                    Ok(program) => {
                                        *game_program_for_callback.borrow_mut() = Some(program);
                                        *start_time_for_callback.borrow_mut() = Some(
                                            Instant::now()
                                        );
                                        println!("[UNDERLAY] Game engine initialized successfully");
                                    }
                                    Err(e) => {
                                        eprintln!("[UNDERLAY] Failed to create game program: {}", e);
                                    }
                                }
                            }
                            _ => {
                                println!("[UNDERLAY] Non-OpenGL graphics API detected");
                            }
                        }
                    }
                    RenderingState::BeforeRendering => {
                        // Render game 3D scene before Slint draws UI with proper state management
                        if
                            let (Some(ref mut program), Some(start_time)) = (
                                &mut *game_program_for_callback.borrow_mut(),
                                &*start_time_for_callback.borrow(),
                            )
                        {
                            // Get actual window size for responsive rendering
                            if let Some(app) = ui_app_weak_for_rendering.upgrade() {
                                let size = app.window().size();
                                let elapsed_time = start_time.elapsed().as_secs_f32();

                                // Save and configure OpenGL state, then render
                                let saved_state = {
                                    let gl = program.get_gl_context();
                                    unsafe {
                                        // Save current OpenGL state
                                        let mut viewport = [0i32; 4];
                                        gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);

                                        let depth_test_enabled = gl.is_enabled(glow::DEPTH_TEST);
                                        let cull_face_enabled = gl.is_enabled(glow::CULL_FACE);
                                        let blend_enabled = gl.is_enabled(glow::BLEND);

                                        let mut current_program = 0;
                                        gl.get_parameter_i32_slice(
                                            glow::CURRENT_PROGRAM,
                                            std::slice::from_mut(&mut current_program)
                                        );

                                        let mut depth_func = 0;
                                        gl.get_parameter_i32_slice(
                                            glow::DEPTH_FUNC,
                                            std::slice::from_mut(&mut depth_func)
                                        );

                                        let mut depth_writemask = 0;
                                        gl.get_parameter_i32_slice(
                                            glow::DEPTH_WRITEMASK,
                                            std::slice::from_mut(&mut depth_writemask)
                                        );

                                        // Configure OpenGL for 3D rendering
                                        gl.enable(glow::DEPTH_TEST);
                                        gl.depth_func(glow::LESS);
                                        gl.depth_mask(true);

                                        // Enable face culling to hide back faces
                                        gl.enable(glow::CULL_FACE);
                                        gl.cull_face(glow::BACK);
                                        gl.front_face(glow::CCW);

                                        // Ensure proper viewport for 3D rendering
                                        gl.viewport(0, 0, size.width as i32, size.height as i32);

                                        (
                                            viewport,
                                            depth_test_enabled,
                                            cull_face_enabled,
                                            blend_enabled,
                                            current_program as u32,
                                            depth_func as u32,
                                            depth_writemask != 0,
                                        )
                                    }
                                };

                                // Render our 3D game scene
                                if
                                    let Err(e) = program.render(
                                        size.width as u32,
                                        size.height as u32,
                                        elapsed_time
                                    )
                                {
                                    eprintln!("[UNDERLAY] Game render error: {}", e);
                                }

                                // Restore OpenGL state for Slint UI rendering
                                {
                                    let gl = program.get_gl_context();
                                    unsafe {
                                        let (
                                            viewport,
                                            depth_test_enabled,
                                            cull_face_enabled,
                                            blend_enabled,
                                            current_program,
                                            depth_func,
                                            depth_writemask,
                                        ) = saved_state;

                                        // Restore viewport
                                        gl.viewport(
                                            viewport[0],
                                            viewport[1],
                                            viewport[2],
                                            viewport[3]
                                        );

                                        // Restore depth testing state
                                        if depth_test_enabled {
                                            gl.enable(glow::DEPTH_TEST);
                                        } else {
                                            gl.disable(glow::DEPTH_TEST);
                                        }
                                        gl.depth_func(depth_func);
                                        gl.depth_mask(depth_writemask);

                                        // Restore face culling state
                                        if cull_face_enabled {
                                            gl.enable(glow::CULL_FACE);
                                        } else {
                                            gl.disable(glow::CULL_FACE);
                                        }

                                        // Restore blending state
                                        if blend_enabled {
                                            gl.enable(glow::BLEND);
                                        } else {
                                            gl.disable(glow::BLEND);
                                        }

                                        // Restore shader program
                                        if current_program != 0 {
                                            // Create a Program handle from the raw ID
                                            let program_handle = std::mem::transmute::<
                                                u32,
                                                glow::Program
                                            >(current_program);
                                            gl.use_program(Some(program_handle));
                                        } else {
                                            gl.use_program(None);
                                        }

                                        // Clear depth buffer to ensure UI renders on top
                                        gl.clear(glow::DEPTH_BUFFER_BIT);
                                    }
                                }
                            }
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

    // Set up continuous animation and UI updates with continuous movement dispatch
    println!("[DEBUG] Setting up animation timer with UI updates and continuous movement...");
    let animation_timer = slint::Timer::default();
    let desktop_handler_for_timer = desktop_handler.clone();

    animation_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16), // ~60 FPS
        move || {
            // Update UI with current game state
            if let Some(app) = ui_app_weak_for_animation.upgrade() {
                // Request redraw for animation
                app.window().request_redraw();
            }
            
            // Continuous movement dispatch: check pressed keys and calculate direction dynamically
            let pressed_keys = desktop_handler_for_timer.borrow().get_pressed_keys();
            let w = pressed_keys.contains(&winit::keyboard::KeyCode::KeyW);
            let a = pressed_keys.contains(&winit::keyboard::KeyCode::KeyA);
            let s = pressed_keys.contains(&winit::keyboard::KeyCode::KeyS);
            let d = pressed_keys.contains(&winit::keyboard::KeyCode::KeyD);
            
            // Only dispatch if any WASD keys are pressed
            if w || a || s || d {
                let direction = crate::index::engine::utils::input_utils::calculate_movement_direction(w, a, s, d);
                // Create movement event directly and dispatch it
                let move_event = crate::index::engine::systems::eventSystem::Event {
                    event_type: crate::index::engine::systems::eventSystem::EventType::Move,
                    payload: Box::new(direction),
                };
                crate::index::engine::systems::eventSystem::EventSystem::notify(move_event);
            }
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
