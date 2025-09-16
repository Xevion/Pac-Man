//! Desktop platform implementation.

use std::time::Duration;

use rand::rngs::ThreadRng;
use rust_embed::Embed;

use crate::error::PlatformError;

#[derive(Embed)]
#[folder = "assets/game/"]
struct EmbeddedAssets;

/// Desktop platform implementation.
pub fn sleep(duration: Duration, focused: bool) {
    if focused {
        spin_sleep::sleep(duration);
    } else {
        std::thread::sleep(duration);
    }
}

#[allow(unused_variables)]
pub fn init_console(force_console: bool) -> Result<(), PlatformError> {
    use crate::formatter::CustomFormatter;
    use tracing::Level;
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    // Create a file layer
    let log_file = std::fs::File::create("pacman.log")
        .map_err(|e| PlatformError::ConsoleInit(format!("Failed to create log file: {}", e)))?;
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(log_file)
        .event_format(CustomFormatter)
        .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::DEBUG))
        .boxed();

    #[cfg(windows)]
    {
        // If using windows subsystem, and force_console is true, allocate a new console window
        if force_console && cfg!(not(use_console)) {
            use crate::platform::tracing_buffer::{SwitchableMakeWriter, SwitchableWriter};

            // Setup deferred tracing subscriber that will buffer logs until console is ready
            let switchable_writer = SwitchableWriter::default();
            let make_writer = SwitchableMakeWriter::new(switchable_writer.clone());
            let console_layer = fmt::layer()
                .with_ansi(true)
                .with_writer(make_writer)
                .event_format(CustomFormatter)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::DEBUG))
                .boxed();

            tracing_subscriber::registry()
                .with(console_layer)
                .with(file_layer)
                .with(ErrorLayer::default())
                .init();

            // Enable virtual terminal processing for ANSI colors
            allocate_console()?;
            enable_ansi_support()?;

            switchable_writer
                .switch_to_direct_mode()
                .map_err(|e| PlatformError::ConsoleInit(format!("Failed to switch to direct mode: {}", e)))?;
        } else {
            // Set up tracing subscriber with ANSI colors enabled
            let console_layer = fmt::layer()
                .with_ansi(true)
                .with_writer(std::io::stdout)
                .event_format(CustomFormatter)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::DEBUG))
                .boxed();

            tracing_subscriber::registry()
                .with(console_layer)
                .with(file_layer)
                .with(ErrorLayer::default())
                .init();
        }
    }

    #[cfg(not(windows))]
    {
        // Set up tracing subscriber with ANSI colors enabled
        let console_layer = fmt::layer()
            .with_ansi(true)
            .with_writer(std::io::stdout)
            .event_format(CustomFormatter)
            .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::DEBUG))
            .boxed();

        tracing_subscriber::registry()
            .with(console_layer)
            .with(file_layer)
            .with(ErrorLayer::default())
            .init();
    }

    Ok(())
}

pub fn rng() -> ThreadRng {
    rand::rng()
}

/// Enable ANSI escape sequence support in the Windows console
/// Windows-only
#[cfg(windows)]
fn enable_ansi_support() -> Result<(), PlatformError> {
    use windows::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_ERROR_HANDLE,
        STD_OUTPUT_HANDLE,
    };

    // Enable ANSI processing for stdout
    unsafe {
        let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE)
            .map_err(|e| PlatformError::ConsoleInit(format!("Failed to get stdout handle: {:?}", e)))?;

        let mut console_mode = CONSOLE_MODE(0);
        GetConsoleMode(stdout_handle, &mut console_mode)
            .map_err(|e| PlatformError::ConsoleInit(format!("Failed to get console mode: {:?}", e)))?;

        console_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        SetConsoleMode(stdout_handle, console_mode)
            .map_err(|e| PlatformError::ConsoleInit(format!("Failed to enable ANSI for stdout: {:?}", e)))?;
    }

    // Enable ANSI processing for stderr
    unsafe {
        let stderr_handle = GetStdHandle(STD_ERROR_HANDLE)
            .map_err(|e| PlatformError::ConsoleInit(format!("Failed to get stderr handle: {:?}", e)))?;

        let mut console_mode = CONSOLE_MODE(0);
        GetConsoleMode(stderr_handle, &mut console_mode)
            .map_err(|e| PlatformError::ConsoleInit(format!("Failed to get console mode: {:?}", e)))?;

        console_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        SetConsoleMode(stderr_handle, console_mode)
            .map_err(|e| PlatformError::ConsoleInit(format!("Failed to enable ANSI for stderr: {:?}", e)))?;
    }

    Ok(())
}

/// Allocate a new console window for the process
/// Windows-only
#[cfg(windows)]
fn allocate_console() -> Result<(), PlatformError> {
    use windows::{
        core::PCSTR,
        Win32::{
            Foundation::{GENERIC_READ, GENERIC_WRITE},
            Storage::FileSystem::{CreateFileA, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
            System::Console::{AllocConsole, SetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
        },
    };

    // Allocate a new console for this process
    unsafe { AllocConsole() }.map_err(|e| PlatformError::ConsoleInit(format!("Failed to allocate console: {:?}", e)))?;

    // Note: SetConsoleTitle is not available in the imported modules, skipping title setting

    // Redirect stdout
    let stdout_handle = unsafe {
        let pcstr = PCSTR::from_raw(c"CONOUT$".as_ptr() as *const u8);
        CreateFileA::<PCSTR>(
            pcstr,
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }
    .map_err(|e| PlatformError::ConsoleInit(format!("Failed to create stdout handle: {:?}", e)))?;

    // Redirect stdin
    let stdin_handle = unsafe {
        let pcstr = PCSTR::from_raw(c"CONIN$".as_ptr() as *const u8);
        CreateFileA::<PCSTR>(
            pcstr,
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }
    .map_err(|e| PlatformError::ConsoleInit(format!("Failed to create stdin handle: {:?}", e)))?;

    // Set the standard handles
    unsafe { SetStdHandle(STD_OUTPUT_HANDLE, stdout_handle) }
        .map_err(|e| PlatformError::ConsoleInit(format!("Failed to set stdout handle: {:?}", e)))?;

    unsafe { SetStdHandle(STD_ERROR_HANDLE, stdout_handle) }
        .map_err(|e| PlatformError::ConsoleInit(format!("Failed to set stderr handle: {:?}", e)))?;

    unsafe { SetStdHandle(STD_INPUT_HANDLE, stdin_handle) }
        .map_err(|e| PlatformError::ConsoleInit(format!("Failed to set stdin handle: {:?}", e)))?;

    Ok(())
}
