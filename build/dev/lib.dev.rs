use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use glow::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

mod index;
use index::Program;
use index::event_system::{Event, EventType};

// Platform-specific shader source functions for Web/WASM
#[no_mangle]
pub fn get_vertex_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/vertex_animated.glsl");
    source.replace("#VERSION", "#version 300 es\nprecision mediump float;")
}

#[no_mangle]
pub fn get_fragment_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/fragment_animated.glsl");
    source.replace("#VERSION", "#version 300 es\nprecision mediump float;")
}

#[no_mangle]
pub fn get_static_vertex_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/vertex_static.glsl");
    source.replace("#VERSION", "#version 300 es\nprecision mediump float;")
}

#[no_mangle]
pub fn get_static_fragment_shader_source() -> String {
    let source = include_str!("../src/assets/shaders/fragment_static.glsl");
    source.replace("#VERSION", "#version 300 es\nprecision mediump float;")
}

struct InputState {
    pressed_keys: HashSet<String>,
    last_mouse_pos: Option<(f64, f64)>,
    last_direction: String,
}

impl InputState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            last_mouse_pos: None,
            last_direction: "idle".to_string(),
        }
    }

    fn calculate_direction(&self) -> String {
        let w = self.pressed_keys.contains("KeyW");
        let a = self.pressed_keys.contains("KeyA");
        let s = self.pressed_keys.contains("KeyS");
        let d = self.pressed_keys.contains("KeyD");

        // Apply cancellation logic
        let forward = w && !s;
        let back = s && !w;
        let left = a && !d;
        let right = d && !a;

        match (forward, back, left, right) {
            (true, false, true, false) => "forward-left".to_string(),
            (true, false, false, true) => "forward-right".to_string(),
            (false, true, true, false) => "back-left".to_string(),
            (false, true, false, true) => "back-right".to_string(),
            (true, false, false, false) => "forward".to_string(),
            (false, true, false, false) => "back".to_string(),
            (false, false, true, false) => "left".to_string(),
            (false, false, false, true) => "right".to_string(),
            _ => "idle".to_string(),
        }
    }

    fn mouse_delta_to_quaternion(delta_x: f64, delta_y: f64) -> [f32; 4] {
        let sensitivity = 0.002;
        let yaw = (delta_x * sensitivity) as f32;
        let pitch = (delta_y * sensitivity) as f32;

        // Create quaternion from yaw and pitch
        let cos_yaw = (yaw * 0.5).cos();
        let sin_yaw = (yaw * 0.5).sin();
        let cos_pitch = (pitch * 0.5).cos();
        let sin_pitch = (pitch * 0.5).sin();

        // Combine yaw and pitch quaternions
        [
            cos_yaw * cos_pitch,
            sin_yaw * cos_pitch,
            cos_yaw * sin_pitch,
            -sin_yaw * sin_pitch,
        ]
    }
}

struct RenderState {
    program: Program,
    canvas: HtmlCanvasElement,
    start_time: f64,
    last_frame_time: f64,
    input_state: InputState,
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
        input_state: InputState::new(),
    }));
    
    // Setup input event listeners
    setup_input_listeners(render_state.clone())?;
    
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

fn setup_input_listeners(render_state: Rc<RefCell<RenderState>>) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Keyboard event listeners
    {
        let render_state_clone = render_state.clone();
        let keydown_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            let mut state = render_state_clone.borrow_mut();
            let key_code = event.code();
            
            if matches!(key_code.as_str(), "KeyW" | "KeyA" | "KeyS" | "KeyD") {
                state.input_state.pressed_keys.insert(key_code);
                
                let new_direction = state.input_state.calculate_direction();
                if new_direction != state.input_state.last_direction {
                    state.input_state.last_direction = new_direction.clone();
                    
                    let event = Event {
                        event_type: EventType::Move,
                        payload: Box::new(new_direction),
                    };
                    
                    state.program.receive_event(&event);
                }
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        document.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();
    }

    {
        let render_state_clone = render_state.clone();
        let keyup_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            let mut state = render_state_clone.borrow_mut();
            let key_code = event.code();
            
            if matches!(key_code.as_str(), "KeyW" | "KeyA" | "KeyS" | "KeyD") {
                state.input_state.pressed_keys.remove(&key_code);
                
                let new_direction = state.input_state.calculate_direction();
                if new_direction != state.input_state.last_direction {
                    state.input_state.last_direction = new_direction.clone();
                    
                    let event = Event {
                        event_type: EventType::Move,
                        payload: Box::new(new_direction),
                    };
                    
                    state.program.receive_event(&event);
                }
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        document.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())?;
        keyup_closure.forget();
    }

    // Mouse event listener
    {
        let render_state_clone = render_state.clone();
        let mousemove_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mut state = render_state_clone.borrow_mut();
            let current_pos = (event.client_x() as f64, event.client_y() as f64);
            
            if let Some(last_pos) = state.input_state.last_mouse_pos {
                let delta_x = current_pos.0 - last_pos.0;
                let delta_y = current_pos.1 - last_pos.1;
                
                // Only send event if there's significant movement
                if delta_x.abs() > 1.0 || delta_y.abs() > 1.0 {
                    let quaternion = InputState::mouse_delta_to_quaternion(delta_x, delta_y);
                    
                    let event = Event {
                        event_type: EventType::RotateCamera,
                        payload: Box::new(quaternion),
                    };
                    
                    state.program.receive_event(&event);
                }
            }
            
            state.input_state.last_mouse_pos = Some(current_pos);
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);

        document.add_event_listener_with_callback("mousemove", mousemove_closure.as_ref().unchecked_ref())?;
        mousemove_closure.forget();
    }

    log("🎮 Input event listeners setup complete!");
    Ok(())
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}
