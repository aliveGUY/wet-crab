use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use glow::Context;
use std::rc::Rc;
use std::cell::RefCell;

mod index;
use index::Program;

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
    
    let program = Program::new(gl)
        .map_err(|e| JsValue::from_str(&e))?;
    
    let render_state = Rc::new(RefCell::new(RenderState {
        program,
        canvas,
        start_time: 0.0, // Will be set on first frame
        last_frame_time: 0.0,
    }));
    
    // Start the animation loop
    request_animation_frame(render_state.clone())?;
    
    log("🔺 Continuous rendering loop started!");
    Ok(())
}

fn request_animation_frame(render_state: Rc<RefCell<RenderState>>) -> Result<(), JsValue> {
    let render_state_clone = render_state.clone();
    
    let closure = Closure::wrap(Box::new(move |current_time: f64| {
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
        if let Err(e) = state.program.render(
            canvas_width, 
            canvas_height, 
            elapsed_time
        ) {
            web_sys::console::error_1(&format!("Render error: {}", e).into());
            return;
        }
        
        // Request next frame
        drop(state); // Release borrow before recursive call
        if let Err(e) = request_animation_frame(render_state.clone()) {
            web_sys::console::error_1(&e);
        }
    }) as Box<dyn FnMut(f64)>);
    
    web_sys::window()
        .unwrap()
        .request_animation_frame(closure.as_ref().unchecked_ref())?;
    
    closure.forget(); // Keep closure alive
    Ok(())
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}
