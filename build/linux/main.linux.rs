//! Hybrid OpenGL + Slint Application for runst-poc
//! 
//! This application demonstrates integration of:
//! - OpenGL 3D rendering (game underlay)
//! - Slint UI (debug overlay)
//! - Direct winit event subscription for hybrid input
//! - FPS-style camera controls with cursor locking

use slint::{ComponentHandle, RenderingState, GraphicsAPI};
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use glow::HasContext;

// Import our game engine
mod index;
use index::{ Program };
use index::engine::systems::{ EventSystem, InputSystem, DesktopInputHandler };
use index::engine::editor_ui::SlintInputManager;

// Include the Slint UI from external file
slint::include_modules!();

// Winit integration imports
use slint::winit_030::{WinitWindowAccessor, WinitWindowEventResult, winit};
use winit::event::{WindowEvent, DeviceEvent, DeviceId};
use winit::keyboard::PhysicalKey;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[HYBRID] Starting runst-poc with Slint + OpenGL integration");
    
    // Ensure Winit backend is selected for Slint
    println!("[DEBUG] Selecting Winit backend for Slint...");
    slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap()))?;
    println!("[DEBUG] Winit backend selected");
    
    // Create Slint UI using Winit backend
    println!("[DEBUG] Creating Slint debug UI...");
    let ui_app = GameDebugUI::new().expect("Failed to create Slint UI");
    println!("[DEBUG] Slint UI created successfully");
    
    // Initialize input state for hybrid input system
    let slint_input = Rc::new(RefCell::new(SlintInputManager::new()));
    let slint_input_for_events = slint_input.clone();
    let slint_input_for_ui = slint_input.clone();
    println!("[DEBUG] Slint input manager initialized");
    
    // Set up direct Winit event interception for hybrid input control
    println!("[DEBUG] Setting up direct Winit event interception...");
    ui_app.window().on_winit_window_event({
        move |_slint_window, event| {
            let mut input = slint_input_for_events.borrow_mut();
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    input.process_mouse_movement(position.x, position.y);
                    WinitWindowEventResult::Propagate
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let prevent = input.process_mouse_button(*button, *state);
                    if prevent {
                        WinitWindowEventResult::PreventDefault
                    } else {
                        WinitWindowEventResult::Propagate
                    }
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let prevent = input.process_winit_keyboard(&event.physical_key, event.state);
                    if prevent {
                        WinitWindowEventResult::PreventDefault
                    } else {
                        WinitWindowEventResult::Propagate
                    }
                }
                WindowEvent::Focused(focused) => {
                    if *focused {
                        println!("[SLINT-INPUT] Window gained focus");
                    } else {
                        println!("[SLINT-INPUT] Window lost focus");
                    }
                    WinitWindowEventResult::Propagate
                }
                _ => WinitWindowEventResult::Propagate
            }
        }
    });
    println!("[DEBUG] Direct Winit event interception enabled");
    
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
    ui_window.set_rendering_notifier({
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
                                    eprintln!("[ERROR] No depth buffer allocated! Depth testing will not work.");
                                    eprintln!("[ERROR] This will cause rear-facing polygons to render in front.");
                                } else {
                                    println!("[DEBUG] Depth buffer: {} bits - depth testing available", depth_bits);
                                }
                                
                                // Check other buffer configurations
                                let color_bits = gl.get_parameter_i32(glow::RED_BITS) + 
                                                gl.get_parameter_i32(glow::GREEN_BITS) + 
                                                gl.get_parameter_i32(glow::BLUE_BITS) + 
                                                gl.get_parameter_i32(glow::ALPHA_BITS);
                                println!("[DEBUG] Color buffer: {} bits total", color_bits);
                                
                                let stencil_bits = gl.get_parameter_i32(glow::STENCIL_BITS);
                                println!("[DEBUG] Stencil buffer: {} bits", stencil_bits);
                            }
                            
                            // Initialize game engine systems
                            println!("[DEBUG] Initializing game engine systems...");
                            EventSystem::initialize();
                            InputSystem::initialize(Box::new(DesktopInputHandler::new()));
                            
                            // Create game program
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
                        }
                        _ => {
                            println!("[UNDERLAY] Non-OpenGL graphics API detected");
                        }
                    }
                }
                RenderingState::BeforeRendering => {
                    // Render game 3D scene before Slint draws UI with proper state management
                    if let (Some(ref mut program), Some(start_time)) = (
                        &mut *game_program_for_callback.borrow_mut(),
                        &*start_time_for_callback.borrow()
                    ) {
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
                                    gl.get_parameter_i32_slice(glow::CURRENT_PROGRAM, std::slice::from_mut(&mut current_program));
                                    
                                    let mut depth_func = 0;
                                    gl.get_parameter_i32_slice(glow::DEPTH_FUNC, std::slice::from_mut(&mut depth_func));
                                    
                                    let mut depth_writemask = 0;
                                    gl.get_parameter_i32_slice(glow::DEPTH_WRITEMASK, std::slice::from_mut(&mut depth_writemask));
                                    
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
                                    
                                    (viewport, depth_test_enabled, cull_face_enabled, blend_enabled, 
                                     current_program as u32, depth_func as u32, depth_writemask != 0)
                                }
                            };
                            
                            // Render our 3D game scene
                            if let Err(e) = program.render(size.width as u32, size.height as u32, elapsed_time) {
                                eprintln!("[UNDERLAY] Game render error: {}", e);
                            }
                            
                            // Restore OpenGL state for Slint UI rendering
                            {
                                let gl = program.get_gl_context();
                                unsafe {
                                    let (viewport, depth_test_enabled, cull_face_enabled, blend_enabled, 
                                         current_program, depth_func, depth_writemask) = saved_state;
                                    
                                    // Restore viewport
                                    gl.viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
                                    
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
                                        let program_handle = std::mem::transmute::<u32, glow::Program>(current_program);
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
    }).expect("Failed to set rendering notifier");

    // Set up UI callbacks
    println!("[DEBUG] Setting up UI callbacks...");
    ui_app.on_toggle_wireframe({
        move || {
            println!("[UI] Wireframe mode toggled");
            // TODO: Implement wireframe toggle in game engine
        }
    });
    
    ui_app.on_reset_camera({
        move || {
            println!("[UI] Camera reset requested");
            // TODO: Implement camera reset in game engine
        }
    });
    
    ui_app.on_set_animation_speed({
        move |speed| {
            println!("[UI] Animation speed set to: {}", speed);
            // TODO: Implement animation speed control in game engine
        }
    });

    // Set up continuous animation and UI updates
    println!("[DEBUG] Setting up animation timer with UI updates...");
    let animation_timer = slint::Timer::default();
    let slint_input_for_timer = slint_input_for_ui.clone();
    
    animation_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16), // ~60 FPS
        move || {
            // Update input state frame management
            slint_input_for_timer.borrow_mut().update_frame();
            
            // Update UI with current game state
            if let Some(app) = ui_app_weak_for_animation.upgrade() {
                let input = slint_input_for_timer.borrow();
                
                // Update UI properties with real-time data
                app.set_wasd_state(input.get_wasd_state().into());
                app.set_mouse_position(input.get_mouse_position_string().into());
                app.set_fps(input.get_fps() as i32);
                app.set_camera_rotation("(0, 0, 0)".into()); // TODO: Get from camera
                
                // Request redraw for animation
                app.window().request_redraw();
            }
        }
    );

    println!("[HYBRID] Starting Slint event loop with game integration");
    println!("🎮 Game Controls:");
    println!("   - WASD: Movement (logged to console)");
    println!("   - Mouse: Camera look (position logged)");
    println!("   - Left Click: Lock cursor for FPS mode");
    println!("   - Escape: Unlock cursor");
    println!("📱 Debug UI: Real-time game state overlay");
    
    ui_app.run()?;
    
    Ok(())
}
