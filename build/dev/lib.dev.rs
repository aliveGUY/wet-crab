use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ HtmlCanvasElement, WebGl2RenderingContext, KeyboardEvent, MouseEvent };
use glow::Context;
use std::cell::RefCell;
use std::rc::Rc;

mod index;
use index::{ Program, GlobalEventSystem, BrowserEventHandler };

struct RenderState {
    program: Program,
    canvas: HtmlCanvasElement,
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

    // Initialize GlobalEventSystem with BrowserEventHandler
    let browser_handler = Box::new(BrowserEventHandler::new());
    GlobalEventSystem::initialize(browser_handler);

    let program = Program::new(gl).map_err(|e| JsValue::from_str(&e))?;

    let render_state = Rc::new(
        RefCell::new(RenderState {
            program,
            canvas,
            start_time: 0.0,
            last_frame_time: 0.0,
        })
    );

    {
        let keydown_closure = Closure::<dyn FnMut(KeyboardEvent)>::wrap(
            Box::new(move |ke: KeyboardEvent| {
                GlobalEventSystem::receive_native_keyboard_event(&ke);
            })
        );
        document.add_event_listener_with_callback(
            "keydown",
            keydown_closure.as_ref().unchecked_ref()
        )?;
        keydown_closure.forget();
    }

    {
        let keyup_closure = Closure::<dyn FnMut(KeyboardEvent)>::wrap(
            Box::new(move |ke: KeyboardEvent| {
                GlobalEventSystem::receive_native_keyboard_event(&ke);
            })
        );
        document.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())?;
        keyup_closure.forget();
    }

    {
        let mousemove_closure = Closure::<dyn FnMut(MouseEvent)>::wrap(
            Box::new(move |me: MouseEvent| {
                GlobalEventSystem::receive_native_mouse_event(&me);
            })
        );
        document.add_event_listener_with_callback(
            "mousemove",
            mousemove_closure.as_ref().unchecked_ref()
        )?;
        mousemove_closure.forget();
    }

    request_animation_frame(render_state)?;

    log("🔺 Continuous rendering loop started!");
    Ok(())
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
