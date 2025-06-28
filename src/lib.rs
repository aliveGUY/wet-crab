use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use glow::{HasContext, Context, Shader, Program};

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

    unsafe {
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        let vs = compile_shader(&gl, glow::VERTEX_SHADER, include_str!("vertex.glsl"))?;
        let fs = compile_shader(&gl, glow::FRAGMENT_SHADER, include_str!("fragment.glsl"))?;
        let program = link_program(&gl, vs, fs)?;

        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));
        gl.use_program(Some(program));
        gl.clear_color(0.1, 0.2, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.draw_arrays(glow::TRIANGLES, 0, 3);
        gl.use_program(None);
        gl.bind_vertex_array(None);
    }

    log("🔺 Triangle rendered successfully!");
    Ok(())
}

fn compile_shader(gl: &Context, kind: u32, src: &str) -> Result<Shader, JsValue> {
    unsafe {
        let shader = gl.create_shader(kind)?;
        gl.shader_source(shader, src);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            Err(JsValue::from_str(&format!("Shader error: {log}")))
        } else {
            Ok(shader)
        }
    }
}

fn link_program(gl: &Context, vs: Shader, fs: Shader) -> Result<Program, JsValue> {
    unsafe {
        let program = gl.create_program()?;
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        gl.delete_shader(vs);
        gl.delete_shader(fs);
        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            gl.delete_program(program);
            Err(JsValue::from_str(&format!("Link error: {log}")))
        } else {
            Ok(program)
        }
    }
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}
