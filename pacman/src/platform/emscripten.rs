//! Emscripten platform implementation.

use crate::error::PlatformError;
use crate::formatter::CustomFormatter;
use rand::{rngs::SmallRng, SeedableRng};
use std::ffi::CString;
use std::io::{self, Write};
use std::time::Duration;

use std::ffi::c_void;
use std::os::raw::c_int;

/// Callback function type for emscripten main loop
pub type EmMainLoopCallback = unsafe extern "C" fn(*mut c_void);

// Emscripten FFI functions
extern "C" {
    fn emscripten_sleep(ms: u32);
    fn printf(format: *const u8, ...) -> i32;

    /// Set up a browser-friendly main loop with argument passing.
    /// - `func`: callback to run each frame
    /// - `arg`: user data pointer passed to callback
    /// - `fps`: target FPS (0 = use requestAnimationFrame)
    /// - `simulate_infinite_loop`: if 1, never returns (standard for games)
    pub fn emscripten_set_main_loop_arg(func: EmMainLoopCallback, arg: *mut c_void, fps: c_int, simulate_infinite_loop: c_int);

    /// Cancel the currently running main loop.
    /// After calling this, the loop callback will no longer be invoked.
    pub fn emscripten_cancel_main_loop();

    /// Execute JavaScript code from Rust
    fn emscripten_run_script(script: *const i8);
}

/// Execute a JavaScript snippet from Rust.
/// Useful for signaling events to the frontend.
pub fn run_script(script: &str) {
    if let Ok(cstr) = CString::new(script) {
        unsafe {
            emscripten_run_script(cstr.as_ptr());
        }
    }
}

pub fn sleep(duration: Duration, _focused: bool) {
    unsafe {
        emscripten_sleep(duration.as_millis() as u32);
    }
}

/// Yields control to browser event loop without delay.
/// Allows page transitions, animations, and events to process during initialization.
/// Uses ASYNCIFY to pause/resume WASM execution.
pub fn yield_to_browser() {
    unsafe {
        emscripten_sleep(0);
    }
}

pub fn init_console(_force_console: bool) -> Result<(), PlatformError> {
    use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

    // Set up a custom tracing subscriber that writes directly to emscripten console
    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(|| EmscriptenConsoleWriter)
                .with_ansi(false)
                .event_format(CustomFormatter),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")));

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| PlatformError::ConsoleInit(format!("Failed to set tracing subscriber: {}", e)))?;

    Ok(())
}

/// A writer that outputs to the browser console via printf (redirected by emscripten)
struct EmscriptenConsoleWriter;

impl Write for EmscriptenConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            if let Ok(cstr) = CString::new(s.trim_end_matches('\n')) {
                let format_str = CString::new("%s\n").unwrap();
                unsafe {
                    printf(format_str.as_ptr().cast(), cstr.as_ptr());
                }
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn rng() -> SmallRng {
    SmallRng::from_os_rng()
}
