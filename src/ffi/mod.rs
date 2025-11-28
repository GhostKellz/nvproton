#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::path::Path;

use libloading::Library;
use thiserror::Error;

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
}

pub type FfiResult<T> = std::result::Result<T, FfiError>;

pub struct NvLatency {
    library: Library,
}

impl NvLatency {
    /// # Safety
    /// The caller must ensure the native library is compatible with the expected ABI.
    pub unsafe fn load<P: AsRef<Path>>(path: P) -> FfiResult<Self> {
        let library = unsafe { Library::new(path.as_ref())? };
        Ok(Self { library })
    }

    pub fn initialize(&self) -> FfiResult<()> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> c_int> =
                self.library.get(b"nvlatency_initialize\0")?;
            let status = func();
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    pub fn shutdown(&self) -> FfiResult<()> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> c_int> =
                self.library.get(b"nvlatency_shutdown\0")?;
            let status = func();
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    pub fn enable_reflex_mode(&self, game_id: &str, mode: u32) -> FfiResult<()> {
        let game_id = CString::new(game_id)?;
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*const c_char, c_uint) -> c_int> =
                self.library.get(b"nvlatency_enable_reflex_mode\0")?;
            let status = func(game_id.as_ptr(), mode);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    pub fn last_error(&self) -> FfiResult<Option<String>> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> *const c_char> =
                self.library.get(b"nvlatency_last_error\0")?;
            let ptr = func();
            if ptr.is_null() {
                return Ok(None);
            }
            let message = CStr::from_ptr(ptr).to_str()?.to_string();
            Ok(Some(message))
        }
    }
}

pub struct NvShader {
    library: Library,
}

impl NvShader {
    /// # Safety
    /// The caller must ensure the native library is compatible with the expected ABI.
    pub unsafe fn load<P: AsRef<Path>>(path: P) -> FfiResult<Self> {
        let library = unsafe { Library::new(path.as_ref())? };
        Ok(Self { library })
    }

    pub fn warm_cache(&self, game_id: &str) -> FfiResult<()> {
        let game_id = CString::new(game_id)?;
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> c_int> =
                self.library.get(b"nvshader_warm_cache\0")?;
            let status = func(game_id.as_ptr());
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    pub fn cleanup_cache(&self, game_id: &str) -> FfiResult<()> {
        let game_id = CString::new(game_id)?;
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> c_int> =
                self.library.get(b"nvshader_cleanup_cache\0")?;
            let status = func(game_id.as_ptr());
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }
}

pub struct NvSync {
    library: Library,
}

impl NvSync {
    /// # Safety
    /// The caller must ensure the native library is compatible with the expected ABI.
    pub unsafe fn load<P: AsRef<Path>>(path: P) -> FfiResult<Self> {
        let library = unsafe { Library::new(path.as_ref())? };
        Ok(Self { library })
    }

    pub fn set_vrr_range(&self, game_id: &str, min_hz: u32, max_hz: u32) -> FfiResult<()> {
        let game_id = CString::new(game_id)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(*const c_char, c_uint, c_uint) -> c_int,
            > = self.library.get(b"nvsync_set_vrr_range\0")?;
            let status = func(game_id.as_ptr(), min_hz, max_hz);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }

    pub fn enable_frame_limiter(&self, game_id: &str, target_fps: u32) -> FfiResult<()> {
        let game_id = CString::new(game_id)?;
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*const c_char, c_uint) -> c_int> =
                self.library.get(b"nvsync_enable_frame_limiter\0")?;
            let status = func(game_id.as_ptr(), target_fps);
            if status != 0 {
                return Err(FfiError::Operation { code: status });
            }
        }
        Ok(())
    }
}

pub fn load_all_from<P: AsRef<Path>>(root: P) -> FfiResult<LoadedLibraries> {
    let root = root.as_ref();
    unsafe {
        Ok(LoadedLibraries {
            latency: NvLatency::load(root.join("libnvlatency.so"))?,
            shader: NvShader::load(root.join("libnvshader.so"))?,
            sync: NvSync::load(root.join("libnvsync.so"))?,
        })
    }
}

pub struct LoadedLibraries {
    pub latency: NvLatency,
    pub shader: NvShader,
    pub sync: NvSync,
}
