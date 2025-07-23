//! Cross-platform asset loading abstraction.
//! On desktop, assets are embedded using include_bytes!; on Emscripten, assets are loaded from the filesystem.

use std::borrow::Cow;
use std::io;

#[derive(Debug)]
pub enum AssetError {
    Io(io::Error),
}

impl From<io::Error> for AssetError {
    fn from(e: io::Error) -> Self {
        AssetError::Io(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Asset {
    Wav1,
    Wav2,
    Wav3,
    Wav4,
    Pacman,
    Pellet,
    Energizer,
    Map,
    FontKonami,
    GhostBody,
    GhostEyes,
    // Add more as needed
}

#[cfg(not(target_os = "emscripten"))]
mod imp {
    use super::*;
    macro_rules! asset_bytes_enum {
        ( $asset:expr ) => {
            match $asset {
                Asset::Wav1 => Cow::Borrowed(include_bytes!("../assets/wav/1.ogg")),
                Asset::Wav2 => Cow::Borrowed(include_bytes!("../assets/wav/2.ogg")),
                Asset::Wav3 => Cow::Borrowed(include_bytes!("../assets/wav/3.ogg")),
                Asset::Wav4 => Cow::Borrowed(include_bytes!("../assets/wav/4.ogg")),
                Asset::Pacman => Cow::Borrowed(include_bytes!("../assets/32/pacman.png")),
                Asset::Pellet => Cow::Borrowed(include_bytes!("../assets/24/pellet.png")),
                Asset::Energizer => Cow::Borrowed(include_bytes!("../assets/24/energizer.png")),
                Asset::Map => Cow::Borrowed(include_bytes!("../assets/map.png")),
                Asset::FontKonami => Cow::Borrowed(include_bytes!("../assets/font/konami.ttf")),
                Asset::GhostBody => Cow::Borrowed(include_bytes!("../assets/32/ghost_body.png")),
                Asset::GhostEyes => Cow::Borrowed(include_bytes!("../assets/32/ghost_eyes.png")),
            }
        };
    }
    pub fn get_asset_bytes(asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
        Ok(asset_bytes_enum!(asset))
    }
}

#[cfg(target_os = "emscripten")]
mod imp {
    use super::*;
    use std::fs;
    use std::path::Path;
    pub fn get_asset_bytes(asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
        let path = Path::new("assets").join(asset.path());
        let bytes = fs::read(&path)?;
        Ok(Cow::Owned(bytes))
    }
}

pub use imp::get_asset_bytes;
