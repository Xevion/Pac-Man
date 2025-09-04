//! Desktop platform implementation.

use std::borrow::Cow;
use std::time::Duration;

use rand::rngs::ThreadRng;

use crate::asset::Asset;
use crate::error::{AssetError, PlatformError};

/// Desktop platform implementation.
pub fn sleep(duration: Duration, focused: bool) {
    if focused {
        spin_sleep::sleep(duration);
    } else {
        std::thread::sleep(duration);
    }
}

pub fn init_console() -> Result<(), PlatformError> {
    #[cfg(windows)]
    {
        use crate::platform::tracing_buffer::setup_switchable_subscriber;
        use tracing::{debug, info};
        use windows::Win32::System::Console::GetConsoleWindow;

        // Setup buffered tracing subscriber that will buffer logs until console is ready
        let switchable_writer = setup_switchable_subscriber();

        // Check if we already have a console window
        if unsafe { !GetConsoleWindow().0.is_null() } {
            debug!("Already have a console window");
            return Ok(());
        } else {
            debug!("No existing console window found");
        }

        if let Some(file_type) = is_output_setup()? {
            debug!(r#type = file_type, "Existing output detected");
        } else {
            debug!("No existing output detected");

            // Try to attach to parent console for direct cargo run
            attach_to_parent_console()?;
            info!("Successfully attached to parent console");
        }

        // Now that console is initialized, flush buffered logs and switch to direct output
        debug!("Switching to direct logging mode and flushing buffer...");
        if let Err(error) = switchable_writer.switch_to_direct_mode() {
            use tracing::warn;

            warn!("Failed to flush buffered logs to console: {error:?}");
        }
    }

    Ok(())
}

pub fn get_asset_bytes(asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
    match asset {
        Asset::Wav1 => Ok(Cow::Borrowed(include_bytes!("../../assets/game/sound/waka/1.ogg"))),
        Asset::Wav2 => Ok(Cow::Borrowed(include_bytes!("../../assets/game/sound/waka/2.ogg"))),
        Asset::Wav3 => Ok(Cow::Borrowed(include_bytes!("../../assets/game/sound/waka/3.ogg"))),
        Asset::Wav4 => Ok(Cow::Borrowed(include_bytes!("../../assets/game/sound/waka/4.ogg"))),
        Asset::AtlasImage => Ok(Cow::Borrowed(include_bytes!("../../assets/game/atlas.png"))),
        Asset::Font => Ok(Cow::Borrowed(include_bytes!("../../assets/game/TerminalVector.ttf"))),
    }
}

pub fn rng() -> ThreadRng {
    rand::rng()
}

/* Internal functions */

/// Check if the output stream has been setup by a parent process
/// Windows-only
#[cfg(windows)]
fn is_output_setup() -> Result<Option<&'static str>, PlatformError> {
    use tracing::{debug, warn};

    use windows::Win32::Storage::FileSystem::{
        GetFileType, FILE_TYPE_CHAR, FILE_TYPE_DISK, FILE_TYPE_PIPE, FILE_TYPE_REMOTE, FILE_TYPE_UNKNOWN,
    };

    use windows_sys::Win32::{
        Foundation::INVALID_HANDLE_VALUE,
        System::Console::{GetStdHandle, STD_OUTPUT_HANDLE},
    };

    // Get the process's standard output handle, check if it's invalid
    let handle = match unsafe { GetStdHandle(STD_OUTPUT_HANDLE) } {
        INVALID_HANDLE_VALUE => {
            return Err(PlatformError::ConsoleInit("Invalid handle".to_string()));
        }
        handle => handle,
    };

    // Identify the file type of the handle and whether it's 'well known' (i.e. we trust it to be a reasonable output destination)
    let (well_known, file_type) = match unsafe {
        use windows::Win32::Foundation::HANDLE;
        GetFileType(HANDLE(handle))
    } {
        FILE_TYPE_PIPE => (true, "pipe"),
        FILE_TYPE_CHAR => (true, "char"),
        FILE_TYPE_DISK => (true, "disk"),
        FILE_TYPE_UNKNOWN => (false, "unknown"),
        FILE_TYPE_REMOTE => (false, "remote"),
        unexpected => {
            warn!("Unexpected file type: {unexpected:?}");
            (false, "unknown")
        }
    };

    debug!("File type: {file_type:?}, well known: {well_known}");

    // If it's anything recognizable and valid, assume that a parent process has setup an output stream
    Ok(well_known.then_some(file_type))
}

/// Try to attach to parent console
/// Windows-only
#[cfg(windows)]
fn attach_to_parent_console() -> Result<(), PlatformError> {
    use windows::{
        core::PCSTR,
        Win32::{
            Foundation::{GENERIC_READ, GENERIC_WRITE},
            Storage::FileSystem::{CreateFileA, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
            System::Console::{
                AttachConsole, FreeConsole, SetStdHandle, ATTACH_PARENT_PROCESS, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
            },
        },
    };

    // Attach the process to the parent's console
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }
        .map_err(|e| PlatformError::ConsoleInit(format!("Failed to attach to parent console: {:?}", e)))?;

    let handle = unsafe {
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
    .map_err(|e| PlatformError::ConsoleInit(format!("Failed to create console handle: {:?}", e)))?;

    // Set the console's output and then error handles
    if let Some(handle_error) = unsafe { SetStdHandle(STD_OUTPUT_HANDLE, handle) }
        .map_err(|e| PlatformError::ConsoleInit(format!("Failed to set console output handle: {:?}", e)))
        .and_then(|_| {
            unsafe { SetStdHandle(STD_ERROR_HANDLE, handle) }
                .map_err(|e| PlatformError::ConsoleInit(format!("Failed to set console error handle: {:?}", e)))
        })
        .err()
    {
        // If either set handle call fails, free the console
        unsafe { FreeConsole() }
            // Free the console if the SetStdHandle calls fail
            .map_err(|free_error| {
                PlatformError::ConsoleInit(format!(
                    "Failed to free console after SetStdHandle failed: {free_error:?} ({handle_error:?})"
                ))
            })
            // And then return the original error if the FreeConsole call succeeds
            .and(Err(handle_error))?;
    }

    Ok(())
}
