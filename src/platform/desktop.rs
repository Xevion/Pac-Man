//! Desktop platform implementation.

use std::borrow::Cow;
use std::time::Duration;

use crate::asset::Asset;
use crate::error::{AssetError, PlatformError};
use crate::platform::Platform;

/// Desktop platform implementation.
pub struct DesktopPlatform;

impl Platform for DesktopPlatform {
    fn sleep(&self, duration: Duration, focused: bool) {
        if focused {
            spin_sleep::sleep(duration);
        } else {
            std::thread::sleep(duration);
        }
    }

    fn get_time(&self) -> f64 {
        std::time::Instant::now().elapsed().as_secs_f64()
    }

    fn init_console(&self) -> Result<(), PlatformError> {
        #[cfg(windows)]
        {
            unsafe {
                use winapi::{
                    shared::ntdef::NULL,
                    um::{
                        fileapi::{CreateFileA, OPEN_EXISTING},
                        handleapi::INVALID_HANDLE_VALUE,
                        processenv::SetStdHandle,
                        winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE},
                        wincon::{AttachConsole, GetConsoleWindow},
                        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
                    },
                };

                if !std::ptr::eq(GetConsoleWindow(), std::ptr::null_mut()) {
                    return Ok(());
                }

                if AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS) != 0 {
                    let handle = CreateFileA(
                        c"CONOUT$".as_ptr(),
                        GENERIC_READ | GENERIC_WRITE,
                        FILE_SHARE_READ | FILE_SHARE_WRITE,
                        std::ptr::null_mut(),
                        OPEN_EXISTING,
                        0,
                        NULL,
                    );

                    if handle != INVALID_HANDLE_VALUE {
                        SetStdHandle(STD_OUTPUT_HANDLE, handle);
                        SetStdHandle(STD_ERROR_HANDLE, handle);
                    }
                }
            }
        }

        Ok(())
    }

    fn get_canvas_size(&self) -> Option<(u32, u32)> {
        None // Desktop doesn't need this
    }

    fn get_asset_bytes(&self, asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
        match asset {
            Asset::Wav1 => Ok(Cow::Borrowed(include_bytes!("../../assets/game/sound/waka/1.ogg"))),
            Asset::Wav2 => Ok(Cow::Borrowed(include_bytes!("../../assets/game/sound/waka/2.ogg"))),
            Asset::Wav3 => Ok(Cow::Borrowed(include_bytes!("../../assets/game/sound/waka/3.ogg"))),
            Asset::Wav4 => Ok(Cow::Borrowed(include_bytes!("../../assets/game/sound/waka/4.ogg"))),
            Asset::Atlas => Ok(Cow::Borrowed(include_bytes!("../../assets/game/atlas.png"))),
        }
    }
}
