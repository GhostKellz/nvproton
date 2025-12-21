//! FFI bindings for NV Linux Gaming Stack Zig libraries
//!
//! This module provides Rust bindings to:
//! - libnvshader.so - Shader cache management
//! - libnvlatency.so - Reflex and latency control
//! - libnvsync.so - VRR/G-Sync management

#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::path::Path;

use libloading::Library;
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

#[derive(Debug, Error)]
pub enum FfiError {
    #[error("library error: {0}")]
    Library(#[from] libloading::Error),
    #[error("operation returned error code {code}")]
    Operation { code: i32 },
    #[error("ffi string conversion error: {0}")]
    CString(#[from] std::ffi::NulError),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("invalid context")]
    InvalidContext,
    #[error("library not available")]
    NotAvailable,
}

pub type FfiResult<T> = std::result::Result<T, FfiError>;

// =============================================================================
// nvshader - Shader Cache Management
// =============================================================================

/// Pre-warm result from nvshader
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct NvShaderPrewarmResult {
    pub completed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub total: u32,
}

/// Cache statistics from nvshader
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct NvShaderStats {
    pub total_size_bytes: u64,
    pub file_count: u32,
    pub game_count: u32,
    pub dxvk_size: u64,
    pub vkd3d_size: u64,
    pub nvidia_size: u64,
    pub mesa_size: u64,
    pub fossilize_size: u64,
    pub oldest_days: u32,
    pub newest_days: u32,
}

pub struct NvShader {
    library: Library,
    ctx: *mut c_void,
}

impl NvShader {
    /// Load the nvshader library and initialize context
    ///
    /// # Safety
    /// The caller must ensure the native library is compatible with the expected ABI.
    pub unsafe fn load<P: AsRef<Path>>(path: P) -> FfiResult<Self> {
        unsafe {
            let library = Library::new(path.as_ref())?;

            // Initialize context
            let init_fn: libloading::Symbol<unsafe extern "C" fn() -> *mut c_void> =
                library.get(b"nvshader_init\0")?;
            let ctx = init_fn();

            if ctx.is_null() {
                return Err(FfiError::InvalidContext);
            }

            Ok(Self { library, ctx })
        }
    }

    /// Scan for shader caches
    pub fn scan(&self) -> FfiResult<()> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
                self.library.get(b"nvshader_scan\0")?;
            let status = func(self.ctx);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> FfiResult<NvShaderStats> {
        let mut stats = NvShaderStats::default();
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void, *mut NvShaderStats) -> c_int> =
                self.library.get(b"nvshader_get_stats\0")?;
            let status = func(self.ctx, &mut stats);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(stats)
    }

    /// Pre-warm shader cache for a specific game
    pub fn prewarm_game(&self, game_id: &str) -> FfiResult<NvShaderPrewarmResult> {
        let game_id = CString::new(game_id)?;
        let mut result = NvShaderPrewarmResult::default();
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(*mut c_void, *const c_char, *mut NvShaderPrewarmResult) -> c_int,
            > = self.library.get(b"nvshader_prewarm_game\0")?;
            let status = func(self.ctx, game_id.as_ptr(), &mut result);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(result)
    }

    /// Pre-warm all Fossilize shader caches
    pub fn prewarm_all(&self) -> FfiResult<NvShaderPrewarmResult> {
        let mut result = NvShaderPrewarmResult::default();
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(*mut c_void, *mut NvShaderPrewarmResult) -> c_int,
            > = self.library.get(b"nvshader_prewarm_all\0")?;
            let status = func(self.ctx, &mut result);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(result)
    }

    /// Check if pre-warming is available (fossilize_replay found)
    pub fn prewarm_available(&self) -> bool {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> bool> =
                match self.library.get(b"nvshader_prewarm_available\0") {
                    Ok(f) => f,
                    Err(_) => return false,
                };
            func(self.ctx)
        }
    }

    /// Clean caches older than specified days
    pub fn clean_older_than(&self, days: u32) -> FfiResult<u32> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void, c_uint) -> c_int> =
                self.library.get(b"nvshader_clean_older_than\0")?;
            let removed = func(self.ctx, days);
            if removed < 0 {
                return Err(FfiError::Operation { code: removed });
            }
            Ok(removed as u32)
        }
    }

    /// Validate cache entries
    pub fn validate(&self) -> FfiResult<u32> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
                self.library.get(b"nvshader_validate\0")?;
            let invalid = func(self.ctx);
            if invalid < 0 {
                return Err(FfiError::Operation { code: invalid });
            }
            Ok(invalid as u32)
        }
    }

    /// Get last error message
    pub fn last_error(&self) -> Option<String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> *const c_char> =
                match self.library.get(b"nvshader_get_last_error\0") {
                    Ok(f) => f,
                    Err(_) => return None,
                };
            let ptr = func(self.ctx);
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
        }
    }
}

impl Drop for NvShader {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) = self.library.get::<unsafe extern "C" fn(*mut c_void)>(b"nvshader_destroy\0") {
                func(self.ctx);
            }
        }
    }
}

// =============================================================================
// nvlatency - Reflex and Latency Control
// =============================================================================

/// Reflex mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflexMode {
    Off = 0,
    On = 1,
    Boost = 2,
}

/// Frame timing from nvlatency
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct NvLatencyFrameTimings {
    pub frame_id: u64,
    pub simulation_us: u64,
    pub render_submit_us: u64,
    pub present_us: u64,
    pub total_us: u64,
    pub input_latency_us: u64,
}

/// Metrics from nvlatency
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct NvLatencyMetrics {
    pub total_frames: u64,
    pub avg_frame_time_us: u64,
    pub avg_fps: f32,
    pub fps_1_low: f32,
    pub avg_input_latency_us: u64,
}

pub struct NvLatency {
    library: Library,
    ctx: *mut c_void,
}

impl NvLatency {
    /// Load the nvlatency library and initialize context
    ///
    /// # Safety
    /// The caller must ensure the native library is compatible with the expected ABI.
    pub unsafe fn load<P: AsRef<Path>>(path: P) -> FfiResult<Self> {
        unsafe {
            let library = Library::new(path.as_ref())?;

            // Initialize context (requires Vulkan device, but we pass null for basic init)
            let init_fn: libloading::Symbol<
                unsafe extern "C" fn(*mut c_void, u64, *mut c_void) -> *mut c_void,
            > = library.get(b"nvlat_init\0")?;

            // Pass null for device/swapchain - basic context for mode control only
            let ctx = init_fn(std::ptr::null_mut(), 0, std::ptr::null_mut());

            // Note: ctx may be null if no Vulkan device - that's OK for some operations
            Ok(Self { library, ctx })
        }
    }

    /// Check if Reflex is supported
    pub fn is_supported(&self) -> bool {
        if self.ctx.is_null() {
            return false;
        }
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> bool> =
                match self.library.get(b"nvlat_is_supported\0") {
                    Ok(f) => f,
                    Err(_) => return false,
                };
            func(self.ctx)
        }
    }

    /// Set Reflex mode
    pub fn set_reflex_mode(&self, mode: ReflexMode) -> FfiResult<()> {
        if self.ctx.is_null() {
            return Err(FfiError::InvalidContext);
        }
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
                self.library.get(b"nvlat_set_reflex_mode\0")?;
            let status = func(self.ctx, mode as c_int);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    /// Get current Reflex mode
    pub fn get_reflex_mode(&self) -> ReflexMode {
        if self.ctx.is_null() {
            return ReflexMode::Off;
        }
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
                match self.library.get(b"nvlat_get_reflex_mode\0") {
                    Ok(f) => f,
                    Err(_) => return ReflexMode::Off,
                };
            match func(self.ctx) {
                0 => ReflexMode::Off,
                1 => ReflexMode::On,
                2 => ReflexMode::Boost,
                _ => ReflexMode::Off,
            }
        }
    }

    /// Get metrics
    pub fn get_metrics(&self) -> FfiResult<NvLatencyMetrics> {
        if self.ctx.is_null() {
            return Err(FfiError::InvalidContext);
        }
        let mut metrics = NvLatencyMetrics::default();
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void, *mut NvLatencyMetrics)> =
                self.library.get(b"nvlat_get_metrics\0")?;
            func(self.ctx, &mut metrics);
        }
        Ok(metrics)
    }

    /// Check if NVIDIA GPU is present
    pub fn is_nvidia_gpu(&self) -> bool {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> bool> =
                match self.library.get(b"nvlat_is_nvidia_gpu\0") {
                    Ok(f) => f,
                    Err(_) => return false,
                };
            func()
        }
    }

    /// Get library version
    pub fn get_version(&self) -> u32 {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> u32> =
                match self.library.get(b"nvlat_get_version\0") {
                    Ok(f) => f,
                    Err(_) => return 0,
                };
            func()
        }
    }
}

impl Drop for NvLatency {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe {
                if let Ok(func) = self.library.get::<unsafe extern "C" fn(*mut c_void)>(b"nvlat_destroy\0") {
                    func(self.ctx);
                }
            }
        }
    }
}

// =============================================================================
// nvsync - VRR/G-Sync Management
// =============================================================================

/// VRR mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VrrMode {
    Off = 0,
    GSync = 1,
    GSyncCompatible = 2,
    Vrr = 3,
    Unknown = 4,
}

/// Connection type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    DisplayPort = 0,
    Hdmi = 1,
    Dvi = 2,
    Vga = 3,
    Internal = 4,
    Unknown = 5,
}

/// Display information from nvsync
#[repr(C)]
#[derive(Debug, Clone)]
pub struct NvSyncDisplay {
    pub name: [u8; 64],
    pub connector: [u8; 64],
    pub connection_type: c_int,
    pub current_hz: u32,
    pub min_hz: u32,
    pub max_hz: u32,
    pub vrr_capable: bool,
    pub gsync_capable: bool,
    pub gsync_compatible: bool,
    pub lfc_supported: bool,
    pub vrr_enabled: bool,
    pub current_mode: c_int,
    pub width: u32,
    pub height: u32,
}

impl Default for NvSyncDisplay {
    fn default() -> Self {
        Self {
            name: [0; 64],
            connector: [0; 64],
            connection_type: 5, // Unknown
            current_hz: 60,
            min_hz: 48,
            max_hz: 144,
            vrr_capable: false,
            gsync_capable: false,
            gsync_compatible: false,
            lfc_supported: false,
            vrr_enabled: false,
            current_mode: 0,
            width: 1920,
            height: 1080,
        }
    }
}

impl NvSyncDisplay {
    pub fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
        std::str::from_utf8(&self.name[..end]).unwrap_or("")
    }

    pub fn connector_str(&self) -> &str {
        let end = self.connector.iter().position(|&c| c == 0).unwrap_or(self.connector.len());
        std::str::from_utf8(&self.connector[..end]).unwrap_or("")
    }
}

/// System status from nvsync
#[repr(C)]
#[derive(Debug, Clone)]
pub struct NvSyncStatus {
    pub nvidia_detected: bool,
    pub driver_version: [u8; 32],
    pub display_count: u32,
    pub vrr_capable_count: u32,
    pub vrr_enabled_count: u32,
    pub compositor: [u8; 32],
    pub is_wayland: bool,
}

impl Default for NvSyncStatus {
    fn default() -> Self {
        Self {
            nvidia_detected: false,
            driver_version: [0; 32],
            display_count: 0,
            vrr_capable_count: 0,
            vrr_enabled_count: 0,
            compositor: [0; 32],
            is_wayland: false,
        }
    }
}

impl NvSyncStatus {
    pub fn driver_version_str(&self) -> &str {
        let end = self.driver_version.iter().position(|&c| c == 0).unwrap_or(self.driver_version.len());
        std::str::from_utf8(&self.driver_version[..end]).unwrap_or("")
    }

    pub fn compositor_str(&self) -> &str {
        let end = self.compositor.iter().position(|&c| c == 0).unwrap_or(self.compositor.len());
        std::str::from_utf8(&self.compositor[..end]).unwrap_or("")
    }
}

/// Frame limiter config
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct NvSyncFrameLimit {
    pub enabled: bool,
    pub target_fps: u32,
    pub mode: c_int, // 0 = GPU, 1 = CPU, 2 = present_wait
}

pub struct NvSync {
    library: Library,
    ctx: *mut c_void,
}

impl NvSync {
    /// Load the nvsync library and initialize context
    ///
    /// # Safety
    /// The caller must ensure the native library is compatible with the expected ABI.
    pub unsafe fn load<P: AsRef<Path>>(path: P) -> FfiResult<Self> {
        unsafe {
            let library = Library::new(path.as_ref())?;

            // Initialize context
            let init_fn: libloading::Symbol<unsafe extern "C" fn() -> *mut c_void> =
                library.get(b"nvsync_init\0")?;
            let ctx = init_fn();

            if ctx.is_null() {
                return Err(FfiError::InvalidContext);
            }

            Ok(Self { library, ctx })
        }
    }

    /// Scan for connected displays
    pub fn scan(&self) -> FfiResult<()> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
                self.library.get(b"nvsync_scan\0")?;
            let status = func(self.ctx);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    /// Get number of displays
    pub fn get_display_count(&self) -> u32 {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
                match self.library.get(b"nvsync_get_display_count\0") {
                    Ok(f) => f,
                    Err(_) => return 0,
                };
            let count = func(self.ctx);
            if count < 0 { 0 } else { count as u32 }
        }
    }

    /// Get display information by index
    pub fn get_display(&self, index: u32) -> FfiResult<NvSyncDisplay> {
        let mut display = NvSyncDisplay::default();
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(*mut c_void, c_uint, *mut NvSyncDisplay) -> c_int,
            > = self.library.get(b"nvsync_get_display\0")?;
            let status = func(self.ctx, index, &mut display);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(display)
    }

    /// Get system VRR status
    pub fn get_status(&self) -> FfiResult<NvSyncStatus> {
        let mut status = NvSyncStatus::default();
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(*mut c_void, *mut NvSyncStatus) -> c_int,
            > = self.library.get(b"nvsync_get_status\0")?;
            let result = func(self.ctx, &mut status);
            if result != 0 {
                return Err(FfiError::Operation { code: result });
            }
        }
        Ok(status)
    }

    /// Enable VRR on a display (None for all displays)
    pub fn enable_vrr(&self, display_name: Option<&str>) -> FfiResult<()> {
        let name_cstring = display_name.map(|s| CString::new(s)).transpose()?;
        let name_ptr = name_cstring.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());

        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int> =
                self.library.get(b"nvsync_enable_vrr\0")?;
            let status = func(self.ctx, name_ptr);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    /// Disable VRR on a display (None for all displays)
    pub fn disable_vrr(&self, display_name: Option<&str>) -> FfiResult<()> {
        let name_cstring = display_name.map(|s| CString::new(s)).transpose()?;
        let name_ptr = name_cstring.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());

        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int> =
                self.library.get(b"nvsync_disable_vrr\0")?;
            let status = func(self.ctx, name_ptr);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    /// Set frame rate limit (0 to disable)
    pub fn set_frame_limit(&self, target_fps: u32) -> FfiResult<()> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void, c_uint) -> c_int> =
                self.library.get(b"nvsync_set_frame_limit\0")?;
            let status = func(self.ctx, target_fps);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    /// Get frame limit configuration
    pub fn get_frame_limit(&self) -> FfiResult<NvSyncFrameLimit> {
        let mut config = NvSyncFrameLimit::default();
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(*mut c_void, *mut NvSyncFrameLimit) -> c_int,
            > = self.library.get(b"nvsync_get_frame_limit\0")?;
            let status = func(self.ctx, &mut config);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(config)
    }

    /// Check if NVIDIA GPU is present
    pub fn is_nvidia_gpu(&self) -> bool {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> bool> =
                match self.library.get(b"nvsync_is_nvidia_gpu\0") {
                    Ok(f) => f,
                    Err(_) => return false,
                };
            func()
        }
    }

    /// Check if running under Wayland
    pub fn is_wayland(&self) -> bool {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> bool> =
                match self.library.get(b"nvsync_is_wayland\0") {
                    Ok(f) => f,
                    Err(_) => return false,
                };
            func()
        }
    }

    /// Get last error message
    pub fn last_error(&self) -> Option<String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_void) -> *const c_char> =
                match self.library.get(b"nvsync_get_last_error\0") {
                    Ok(f) => f,
                    Err(_) => return None,
                };
            let ptr = func(self.ctx);
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
        }
    }
}

impl Drop for NvSync {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) = self.library.get::<unsafe extern "C" fn(*mut c_void)>(b"nvsync_destroy\0") {
                func(self.ctx);
            }
        }
    }
}

// =============================================================================
// Library Loading Helpers
// =============================================================================

/// Standard library search paths
pub const LIB_PATHS: &[&str] = &[
    "/usr/lib/nvproton",
    "/usr/local/lib/nvproton",
    "/usr/lib",
    "/usr/local/lib",
];

/// Try to load a library from standard paths
pub fn find_library(name: &str) -> Option<std::path::PathBuf> {
    // Check XDG data dir first
    if let Some(data_dir) = dirs::data_local_dir() {
        let path = data_dir.join("nvproton/lib").join(name);
        if path.exists() {
            return Some(path);
        }
    }

    // Check standard paths
    for base in LIB_PATHS {
        let path = std::path::Path::new(base).join(name);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Load nvshader from standard paths
pub fn load_nvshader() -> FfiResult<NvShader> {
    let path = find_library("libnvshader.so").ok_or(FfiError::NotAvailable)?;
    unsafe { NvShader::load(&path) }
}

/// Load nvlatency from standard paths
pub fn load_nvlatency() -> FfiResult<NvLatency> {
    let path = find_library("libnvlatency.so").ok_or(FfiError::NotAvailable)?;
    unsafe { NvLatency::load(&path) }
}

/// Load nvsync from standard paths
pub fn load_nvsync() -> FfiResult<NvSync> {
    let path = find_library("libnvsync.so").ok_or(FfiError::NotAvailable)?;
    unsafe { NvSync::load(&path) }
}

/// All loaded libraries
pub struct LoadedLibraries {
    pub shader: Option<NvShader>,
    pub latency: Option<NvLatency>,
    pub sync: Option<NvSync>,
}

impl LoadedLibraries {
    /// Load all available libraries from standard paths
    pub fn load_available() -> Self {
        Self {
            shader: load_nvshader().ok(),
            latency: load_nvlatency().ok(),
            sync: load_nvsync().ok(),
        }
    }

    /// Load all libraries from a specific root directory
    pub fn load_from<P: AsRef<Path>>(root: P) -> FfiResult<Self> {
        let root = root.as_ref();
        Ok(Self {
            shader: unsafe { NvShader::load(root.join("libnvshader.so")).ok() },
            latency: unsafe { NvLatency::load(root.join("libnvlatency.so")).ok() },
            sync: unsafe { NvSync::load(root.join("libnvsync.so")).ok() },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_str() {
        let mut display = NvSyncDisplay::default();
        display.name[..4].copy_from_slice(b"DP-0");
        assert_eq!(display.name_str(), "DP-0");
    }

    #[test]
    fn test_status_driver_version_str() {
        let mut status = NvSyncStatus::default();
        status.driver_version[..10].copy_from_slice(b"580.105.08");
        assert_eq!(status.driver_version_str(), "580.105.08");
    }
}
