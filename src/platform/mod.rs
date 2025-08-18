//! Platform abstraction layer for cross-platform functionality.

use crate::asset::Asset;
use crate::error::{AssetError, PlatformError};
use std::borrow::Cow;
use std::time::Duration;

#[cfg(not(target_os = "emscripten"))]
mod desktop;
#[cfg(target_os = "emscripten")]
mod emscripten;

/// Cross-platform abstraction layer providing unified APIs for platform-specific operations.
pub trait CommonPlatform {
    /// Platform-specific sleep function (required due to Emscripten's non-standard sleep requirements).
    ///
    /// Provides access to current window focus state, useful for changing sleep algorithm conditionally.
    fn sleep(&self, duration: Duration, focused: bool);

    #[allow(dead_code)]
    fn get_time(&self) -> f64;

    /// Configures platform-specific console and debugging output capabilities.
    fn init_console(&self) -> Result<(), PlatformError>;

    /// Retrieves the actual display canvas dimensions.
    #[allow(dead_code)]
    fn get_canvas_size(&self) -> Option<(u32, u32)>;

    /// Loads raw asset data using the appropriate platform-specific method.
    fn get_asset_bytes(&self, asset: Asset) -> Result<Cow<'static, [u8]>, AssetError>;
}

/// Returns the appropriate platform implementation based on compile-time target.
#[allow(dead_code)]
pub fn get_platform() -> &'static dyn CommonPlatform {
    #[cfg(not(target_os = "emscripten"))]
    {
        &desktop::Platform
    }

    #[cfg(target_os = "emscripten")]
    {
        &emscripten::Platform
    }
}
