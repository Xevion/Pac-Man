//! Emscripten platform implementation.

use std::borrow::Cow;
use std::time::Duration;

use crate::asset::Asset;
use crate::error::{AssetError, PlatformError};
use rand::{rngs::SmallRng, SeedableRng};

// Emscripten FFI functions
extern "C" {
    fn emscripten_get_now() -> f64;
    fn emscripten_sleep(ms: u32);
    fn emscripten_get_element_css_size(target: *const u8, width: *mut f64, height: *mut f64) -> i32;
}

pub fn sleep(duration: Duration, _focused: bool) {
    unsafe {
        emscripten_sleep(duration.as_millis() as u32);
    }
}

pub fn get_time() -> f64 {
    unsafe { emscripten_get_now() }
}

pub fn init_console() -> Result<(), PlatformError> {
    Ok(()) // No-op for Emscripten
}

pub fn requires_console() -> bool {
    false
}

pub fn get_canvas_size() -> Option<(u32, u32)> {
    let mut width = 0.0;
    let mut height = 0.0;

    unsafe {
        emscripten_get_element_css_size(c"canvas".as_ptr().cast(), &mut width, &mut height);
        if width == 0.0 || height == 0.0 {
            return None;
        }
    }
    Some((width as u32, height as u32))
}

pub fn get_asset_bytes(asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
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

pub fn rng() -> SmallRng {
    SmallRng::from_os_rng()
}
