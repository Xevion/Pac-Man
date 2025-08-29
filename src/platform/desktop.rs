//! Desktop platform implementation.

use std::borrow::Cow;
use std::time::Duration;

use crate::asset::Asset;
use crate::error::{AssetError, PlatformError};
use crate::platform::CommonPlatform;

/// Desktop platform implementation.
pub struct Platform;

impl CommonPlatform for Platform {
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
            Asset::AtlasImage => Ok(Cow::Borrowed(include_bytes!("../../assets/game/atlas.png"))),
            Asset::Font => Ok(Cow::Borrowed(include_bytes!("../../assets/game/TerminalVector.ttf"))),
        }
    }
}
