use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ HtmlCanvasElement, WebGl2RenderingContext };
use glow::Context;
use std::rc::Rc;
use std::cell::RefCell;

mod index;
use index::{ Program };
use index::engine::eventSystem::{EventSystem, BrowserEventHandler, NativeEventHandler};

struct RenderState {
    program: Program,
    canvas: HtmlCanvasElement,
    event_handler: Rc<BrowserEventHandler>,
    start_time: f64,
    last_frame_time: f64,
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
    let doc = web_sys::window().unwrap().document().unwrap();
    let canvas = doc
        .get_element_by_id("webgl-canvas")
        .ok_or("Canvas not found")?
        .dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl2")?
        .ok_or("No WebGL2 context")?
        .dyn_into::<WebGl2RenderingContext>()?;

    let gl = Context::from_webgl2_context(context);

    // Create EventSystem with BrowserEventHandler
    let event_handler = Rc::new(BrowserEventHandler::new());
    let event_system = EventSystem::new(Box::new(BrowserEventHandler::new()));
    
    let program = Program::new(gl, event_system).map_err(|e| JsValue::from_str(&e))?;

    let render_state = Rc::new(
        RefCell::new(RenderState {
            program,
            canvas,
            event_handler,
            start_time: 0.0, // Will be set on first frame
            last_frame_time: 0.0,
        })
    );

    // Setup input event listeners
    setup_input_listeners(render_state.clone())?;

    // Start the animation loop
    request_animation_frame(render_state.clone())?;

    log("🔺 Continuous rendering loop started!");
    Ok(())
}

fn request_animation_frame(render_state: Rc<RefCell<RenderState>>) -> Result<(), JsValue> {
    let render_state_clone = render_state.clone();

    let closure = Closure::wrap(
        Box::new(move |current_time: f64| {
            let mut state = render_state_clone.borrow_mut();

            // Initialize start time on first frame
            if state.start_time == 0.0 {
                state.start_time = current_time;
                state.last_frame_time = current_time;
            }

            // Calculate elapsed time in seconds
            let elapsed_time = ((current_time - state.start_time) / 1000.0) as f32;

            state.last_frame_time = current_time;

            // Get canvas dimensions before mutable borrow
            let canvas_width = state.canvas.width();
            let canvas_height = state.canvas.height();

            // Render frame
            if let Err(e) = state.program.render(canvas_width, canvas_height, elapsed_time) {
                web_sys::console::error_1(&format!("Render error: {}", e).into());
                return;
            }

            // Request next frame
            drop(state); // Release borrow before recursive call
            if let Err(e) = request_animation_frame(render_state.clone()) {
                web_sys::console::error_1(&e);
            }
        }) as Box<dyn FnMut(f64)>
    );

    web_sys::window().unwrap().request_animation_frame(closure.as_ref().unchecked_ref())?;

    closure.forget(); // Keep closure alive
    Ok(())
}

fn setup_input_listeners(render_state: Rc<RefCell<RenderState>>) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Keyboard event listeners
    {
        let render_state_clone = render_state.clone();
        let keydown_closure = Closure::wrap(
            Box::new(move |event: web_sys::KeyboardEvent| {
                let state = render_state_clone.borrow();
                let key_code = event.code();

                match key_code.as_str() {
                    "Escape" => {
                        // Unlock cursor on ESC
                        let document = web_sys::window().unwrap().document().unwrap();
                        let _ = document.exit_pointer_lock();
                        state.event_handler.set_cursor_locked(false);
                        log("🔓 Cursor unlocked");
                    }
                    "KeyW" | "KeyA" | "KeyS" | "KeyD" => {
                        // Pass raw keyboard event to EventSystem
                        if let Some(parsed_event) = state.event_handler.parse_keyboard_event(&event) {
                            drop(state); // Release borrow
                            let mut state = render_state_clone.borrow_mut();
                            state.program.receive_event(&parsed_event);
                        }
                    }
                    _ => {}
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>
        );

        document.add_event_listener_with_callback(
            "keydown",
            keydown_closure.as_ref().unchecked_ref()
        )?;
        keydown_closure.forget();
    }

    {
        let render_state_clone = render_state.clone();
        let keyup_closure = Closure::wrap(
            Box::new(move |event: web_sys::KeyboardEvent| {
                let state = render_state_clone.borrow();
                let key_code = event.code();

                if matches!(key_code.as_str(), "KeyW" | "KeyA" | "KeyS" | "KeyD") {
                    // Pass raw keyboard event to EventSystem
                    if let Some(parsed_event) = state.event_handler.parse_keyboard_event(&event) {
                        drop(state); // Release borrow
                        let mut state = render_state_clone.borrow_mut();
                        state.program.receive_event(&parsed_event);
                    }
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>
        );

        document.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())?;
        keyup_closure.forget();
    }

    // Canvas click handler for pointer lock
    {
        let render_state_clone = render_state.clone();
        let canvas_click_closure = Closure::wrap(
            Box::new(move |_event: web_sys::MouseEvent| {
                let state = render_state_clone.borrow();
                let _ = state.canvas.request_pointer_lock();
            }) as Box<dyn FnMut(web_sys::MouseEvent)>
        );

        render_state
            .borrow()
            .canvas.add_event_listener_with_callback(
                "click",
                canvas_click_closure.as_ref().unchecked_ref()
            )?;
        canvas_click_closure.forget();
    }

    // Pointer lock change event listener
    {
        let render_state_clone = render_state.clone();
        let pointer_lock_change_closure = Closure::wrap(
            Box::new(move |_event: web_sys::Event| {
                let state = render_state_clone.borrow();
                let document = web_sys::window().unwrap().document().unwrap();

                // Check if pointer is locked to our canvas
                if let Some(locked_element) = document.pointer_lock_element() {
                    if locked_element == state.canvas.clone().into() {
                        state.event_handler.set_cursor_locked(true);
                        log("🔒 Cursor locked - use ESC to unlock");
                    } else {
                        state.event_handler.set_cursor_locked(false);
                        log("🔓 Cursor unlocked");
                    }
                } else {
                    state.event_handler.set_cursor_locked(false);
                    log("🔓 Cursor unlocked");
                }
            }) as Box<dyn FnMut(web_sys::Event)>
        );

        document.add_event_listener_with_callback(
            "pointerlockchange",
            pointer_lock_change_closure.as_ref().unchecked_ref()
        )?;
        pointer_lock_change_closure.forget();
    }

    // Mouse event listener
    {
        let render_state_clone = render_state.clone();
        let mousemove_closure = Closure::wrap(
            Box::new(move |event: web_sys::MouseEvent| {
                let state = render_state_clone.borrow();
                
                // Pass raw mouse event to EventSystem
                if let Some(parsed_event) = state.event_handler.parse_mouse_event(&event) {
                    drop(state); // Release borrow
                    let mut state = render_state_clone.borrow_mut();
                    state.program.receive_event(&parsed_event);
                }
            }) as Box<dyn FnMut(web_sys::MouseEvent)>
        );

        document.add_event_listener_with_callback(
            "mousemove",
            mousemove_closure.as_ref().unchecked_ref()
        )?;
        mousemove_closure.forget();
    }

    log("🎮 Input event listeners setup complete!");
    Ok(())
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}
