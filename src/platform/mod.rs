//! Platform abstraction layer for cross-platform functionality.

#[cfg(not(target_os = "emscripten"))]
mod desktop;
#[cfg(not(target_os = "emscripten"))]
pub use desktop::*;

/// Tracing buffer is only used on Windows.
#[cfg(target_os = "windows")]
pub mod tracing_buffer;

#[cfg(target_os = "emscripten")]
pub use emscripten::*;
#[cfg(target_os = "emscripten")]
mod emscripten;
