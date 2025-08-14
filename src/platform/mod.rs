//! Platform abstraction layer for cross-platform functionality.

use crate::asset::Asset;
use crate::error::{AssetError, PlatformError};
use std::borrow::Cow;
use std::time::Duration;

pub mod desktop;
pub mod emscripten;

/// Platform abstraction trait that defines cross-platform functionality.
pub trait Platform {
    /// Sleep for the specified duration using platform-appropriate method.
    fn sleep(&self, duration: Duration, focused: bool);

    /// Get the current time in seconds since some reference point.
    /// This is available for future use in timing and performance monitoring.
    #[allow(dead_code)]
    fn get_time(&self) -> f64;

    /// Initialize platform-specific console functionality.
    fn init_console(&self) -> Result<(), PlatformError>;

    /// Get canvas size for platforms that need it (e.g., Emscripten).
    /// This is available for future use in responsive design.
    #[allow(dead_code)]
    fn get_canvas_size(&self) -> Option<(u32, u32)>;

    /// Load asset bytes using platform-appropriate method.
    fn get_asset_bytes(&self, asset: Asset) -> Result<Cow<'static, [u8]>, AssetError>;
}

/// Get the current platform implementation.
#[allow(dead_code)]
pub fn get_platform() -> &'static dyn Platform {
    static DESKTOP: desktop::DesktopPlatform = desktop::DesktopPlatform;
    static EMSCRIPTEN: emscripten::EmscriptenPlatform = emscripten::EmscriptenPlatform;

    #[cfg(not(target_os = "emscripten"))]
    {
        &DESKTOP
    }

    #[cfg(target_os = "emscripten")]
    {
        &EMSCRIPTEN
    }
}
