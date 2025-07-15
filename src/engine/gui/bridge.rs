/// Platform-specific bridge trait for GUI operations
pub trait PlatformBridge {
    /// Get current time in seconds since initialization
    fn get_current_time(&self) -> f64;
    
    /// Get elapsed time since last call (for frame timing)
    fn get_frame_delta(&mut self) -> f32;
    
    /// Log a message using platform-specific logging
    fn log_message(&self, message: &str);
    
    /// Get screen scale factor for high-DPI displays
    fn get_screen_scale(&self) -> f32;
    
    /// Get platform name for debugging
    fn get_platform_name(&self) -> &'static str;
}
