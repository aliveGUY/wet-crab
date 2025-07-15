use super::bridge::PlatformBridge;
use web_time::Instant;

pub struct WasmBridge {
    start_time: Instant,
    last_frame_time: Instant,
}

impl WasmBridge {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
        }
    }
}

impl PlatformBridge for WasmBridge {
    fn get_current_time(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
    
    fn get_frame_delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        delta
    }
    
    fn log_message(&self, message: &str) {
        web_sys::console::log_1(&message.into());
    }
    
    fn get_screen_scale(&self) -> f32 {
        // Get device pixel ratio from browser
        web_sys::window()
            .and_then(|w| w.device_pixel_ratio().into())
            .unwrap_or(1.0) as f32
    }
    
    fn get_platform_name(&self) -> &'static str {
        "WASM"
    }
}
