use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ HtmlCanvasElement, WebGl2RenderingContext, KeyboardEvent, MouseEvent };
use glow::Context;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;

mod index;
use index::{ Program };
use index::engine::systems::{ EventSystem, InputSystem, BrowserInputHandler };

struct RenderState {
    program: Program,
    canvas: HtmlCanvasElement,
    start_time: f64,
    last_frame_time: f64,
    pressed_keys: HashSet<String>,
}

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    log("🦀 Rust WASM module loaded successfully!");

    if let Err(e) = start_render_loop() {
        web_sys::console::error_1(&e);
    }
}

fn start_render_loop() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let canvas = document
        .get_element_by_id("webgl-canvas")
        .ok_or("Canvas not found")?
        .dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl2")?
        .ok_or("No WebGL2 context")?
        .dyn_into::<WebGl2RenderingContext>()?;

    let gl = Context::from_webgl2_context(context);

    // Initialize clean systems architecture
    EventSystem::initialize();
    InputSystem::initialize(Box::new(BrowserInputHandler::new()));

    let program = Program::new(gl).map_err(|e| JsValue::from_str(&e))?;

    let render_state = Rc::new(
        RefCell::new(RenderState {
            program,
            canvas,
            start_time: 0.0,
            last_frame_time: 0.0,
            pressed_keys: HashSet::new(),
        })
    );

    // Setup keyboard state tracking for concurrent movement
    {
        let state_clone = render_state.clone();
        let keydown_closure = Closure::<dyn FnMut(KeyboardEvent)>::wrap(
            Box::new(move |ke: KeyboardEvent| {
                // Handle Escape key for cursor unlocking
                if ke.code() == "Escape" {
                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    if document.pointer_lock_element().is_some() {
                        document.exit_pointer_lock();
                        log("Cursor unlocked via Escape key");
                    }
                }
                
                // Track pressed keys for concurrent movement
                let key_code = ke.code();
                if matches!(key_code.as_str(), "KeyW" | "KeyA" | "KeyS" | "KeyD" | "Space" | "ShiftLeft" | "ShiftRight") {
                    state_clone.borrow_mut().pressed_keys.insert(key_code);
                }
            })
        );
        document.add_event_listener_with_callback(
            "keydown",
            keydown_closure.as_ref().unchecked_ref()
        )?;
        keydown_closure.forget();
    }

    {
        let state_clone = render_state.clone();
        let keyup_closure = Closure::<dyn FnMut(KeyboardEvent)>::wrap(
            Box::new(move |ke: KeyboardEvent| {
                // Remove key from pressed set
                let key_code = ke.code();
                state_clone.borrow_mut().pressed_keys.remove(&key_code);
            })
        );
        document.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())?;
        keyup_closure.forget();
    }

    {
        let mousemove_closure = Closure::<dyn FnMut(MouseEvent)>::wrap(
            Box::new(move |me: MouseEvent| {
                InputSystem::instance().receive_mouse_event(&me);
            })
        );
        document.add_event_listener_with_callback(
            "mousemove",
            mousemove_closure.as_ref().unchecked_ref()
        )?;
        mousemove_closure.forget();
    }

    {
        let mousedown_closure = Closure::<dyn FnMut(MouseEvent)>::wrap(
            Box::new(move |me: MouseEvent| {
                // Handle cursor locking on left mouse click
                if me.button() == 0 {
                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    if let Some(canvas) = document.get_element_by_id("webgl-canvas") {
                        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
                        canvas.request_pointer_lock();
                        log("Cursor locked via left mouse click");
                    }
                }
                InputSystem::instance().receive_mouse_event(&me);
            })
        );
        document.add_event_listener_with_callback(
            "mousedown",
            mousedown_closure.as_ref().unchecked_ref()
        )?;
        mousedown_closure.forget();
    }

    request_animation_frame(render_state)?;

    log("🎮 Browser concurrent input ready!");
    log("📝 Instructions:");
    log("   - Hold WASD keys for movement");
    log("   - Move mouse for camera rotation");
    log("   - Left click to lock cursor");
    log("   - Escape to unlock cursor");
    log("   - Both inputs work simultaneously!");
    log("🔺 Continuous rendering loop started!");
    Ok(())
}

fn process_keyboard_state(pressed_keys: &HashSet<String>) {
    let mut movement_direction = String::new();

    if pressed_keys.contains("KeyW") {
        movement_direction.push_str("forward");
    }
    if pressed_keys.contains("KeyS") {
        if !movement_direction.is_empty() {
            movement_direction.push('-');
        }
        movement_direction.push_str("backward");
    }
    if pressed_keys.contains("KeyA") {
        if !movement_direction.is_empty() {
            movement_direction.push('-');
        }
        movement_direction.push_str("left");
    }
    if pressed_keys.contains("KeyD") {
        if !movement_direction.is_empty() {
            movement_direction.push('-');
        }
        movement_direction.push_str("right");
    }

    if !movement_direction.is_empty() {
        // Send to InputSystem - clean bridge pattern (same as Linux)
        InputSystem::instance().receive_key_event(&movement_direction);
    }
}

fn request_animation_frame(state_rc: Rc<RefCell<RenderState>>) -> Result<(), JsValue> {
    let cb = Closure::<dyn FnMut(f64)>::wrap(
        Box::new(move |now_ms| {
            let mut state = state_rc.borrow_mut();

            if state.start_time == 0.0 {
                state.start_time = now_ms;
                state.last_frame_time = now_ms;
            }

            let elapsed = ((now_ms - state.start_time) / 1000.0) as f32;
            state.last_frame_time = now_ms;

            // Process keyboard state for concurrent movement (like Linux version)
            process_keyboard_state(&state.pressed_keys);

            let (w, h) = (state.canvas.width(), state.canvas.height());
            if let Err(e) = state.program.render(w, h, elapsed) {
                web_sys::console::error_1(&format!("Render error: {e}").into());
                return;
            }

            let _ = request_animation_frame(state_rc.clone());
        })
    );

    web_sys::window().unwrap().request_animation_frame(cb.as_ref().unchecked_ref())?;
    cb.forget();
    Ok(())
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}
