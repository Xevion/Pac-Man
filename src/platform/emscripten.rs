//! Emscripten platform implementation.

use crate::error::PlatformError;
use crate::formatter::CustomFormatter;
use rand::{rngs::SmallRng, SeedableRng};
use std::ffi::CString;
use std::io::{self, Write};
use std::time::Duration;

// Emscripten FFI functions
extern "C" {
    fn emscripten_sleep(ms: u32);
    fn printf(format: *const u8, ...) -> i32;
}

pub fn sleep(duration: Duration, _focused: bool) {
    unsafe {
        emscripten_sleep(duration.as_millis() as u32);
    }
}

pub fn init_console() -> Result<(), PlatformError> {
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
