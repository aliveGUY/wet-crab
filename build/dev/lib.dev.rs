use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use glow::Context;

mod index;
use index::Program;

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    log("🦀 Rust WASM module loaded successfully!");
    
    if let Err(e) = render() {
        web_sys::console::error_1(&e);
    }
}

fn render() -> Result<(), JsValue> {
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
    
    program.render()
        .map_err(|e| JsValue::from_str(&e))?;
    
    log("🔺 Triangle rendered successfully!");
    Ok(())
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}
