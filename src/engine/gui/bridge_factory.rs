use super::bridge::PlatformBridge;

#[cfg(target_arch = "wasm32")]
use super::wasm_bridge::WasmBridge;

#[cfg(not(target_arch = "wasm32"))]
use super::native_bridge::NativeBridge;

/// Factory function to create the appropriate platform bridge
pub fn create_platform_bridge() -> Box<dyn PlatformBridge> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(WasmBridge::new())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(NativeBridge::new())
    }
}

/// Helper function to get platform info without creating a full bridge
pub fn get_platform_info() -> &'static str {
    #[cfg(target_arch = "wasm32")]
    return "WASM";
    
    #[cfg(target_os = "linux")]
    return "Linux";
    
    #[cfg(target_os = "windows")]
    return "Windows";
    
    #[cfg(target_os = "macos")]
    return "macOS";
    
    #[cfg(not(any(target_arch = "wasm32", target_os = "linux", target_os = "windows", target_os = "macos")))]
    return "Unknown";
}
