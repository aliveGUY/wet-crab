use super::bridge::PlatformBridge;
use std::time::Instant;

pub struct NativeBridge {
    start_time: Instant,
    last_frame_time: Instant,
}

impl NativeBridge {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
        }
    }
}

impl PlatformBridge for NativeBridge {
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
        println!("{}", message);
    }
    
    fn get_screen_scale(&self) -> f32 {
        // Default scale factor for native platforms
        // This could be enhanced to detect actual DPI in the future
        1.0
    }
    
    fn get_platform_name(&self) -> &'static str {
        #[cfg(target_os = "linux")]
        return "Linux";
        
        #[cfg(target_os = "windows")]
        return "Windows";
        
        #[cfg(target_os = "macos")]
        return "macOS";
        
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        return "Native";
    }
}
