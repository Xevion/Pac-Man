//! Emscripten platform implementation.

use std::borrow::Cow;
use std::time::Duration;

use crate::asset::Asset;
use crate::error::{AssetError, PlatformError};
use crate::platform::Platform;

/// Emscripten platform implementation.
pub struct EmscriptenPlatform;

impl Platform for EmscriptenPlatform {
    fn sleep(&self, duration: Duration) {
        unsafe {
            emscripten_sleep(duration.as_millis() as u32);
        }
    }

    fn get_time(&self) -> f64 {
        unsafe { emscripten_get_now() }
    }

    fn init_console(&self) -> Result<(), PlatformError> {
        Ok(()) // No-op for Emscripten
    }

    fn get_canvas_size(&self) -> Option<(u32, u32)> {
        Some(unsafe { get_canvas_size() })
    }

    fn get_asset_bytes(&self, asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
        use sdl2::rwops::RWops;
        use std::io::Read;

        let path = format!("assets/game/{}", asset.path());
        let mut rwops = RWops::from_file(&path, "rb").map_err(|_| AssetError::NotFound(asset.path().to_string()))?;

        let len = rwops.len().ok_or_else(|| AssetError::NotFound(asset.path().to_string()))?;

        let mut buf = vec![0u8; len];
        rwops
            .read_exact(&mut buf)
            .map_err(|e| AssetError::Io(std::io::Error::other(e)))?;

        Ok(Cow::Owned(buf))
    }
}

// Emscripten FFI functions
extern "C" {
    fn emscripten_get_now() -> f64;
    fn emscripten_sleep(ms: u32);
    fn emscripten_get_element_css_size(target: *const u8, width: *mut f64, height: *mut f64) -> i32;
}

unsafe fn get_canvas_size() -> (u32, u32) {
    let mut width = 0.0;
    let mut height = 0.0;
    emscripten_get_element_css_size(c"canvas".as_ptr().cast(), &mut width, &mut height);
    (width as u32, height as u32)
}
