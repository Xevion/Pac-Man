//! Platform abstraction layer for cross-platform functionality.
pub mod buffered_writer;
pub mod tracing_buffer;

#[cfg(not(target_os = "emscripten"))]
mod desktop;
#[cfg(not(target_os = "emscripten"))]
pub use desktop::*;

#[cfg(target_os = "emscripten")]
pub use emscripten::*;
#[cfg(target_os = "emscripten")]
mod emscripten;
