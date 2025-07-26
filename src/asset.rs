//! Cross-platform asset loading abstraction.
//! On desktop, assets are embedded using include_bytes!; on Emscripten, assets are loaded from the filesystem.

use std::borrow::Cow;
use std::io;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Asset not found: {0}")]
    NotFound(String),
    #[error("Invalid asset format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Asset {
    Wav1,
    Wav2,
    Wav3,
    Wav4,
    FontKonami,
    Atlas,
    AtlasJson,
    // Add more as needed
}

impl Asset {
    #[allow(dead_code)]
    pub fn path(&self) -> &str {
        use Asset::*;
        match self {
            Wav1 => "sound/waka/1.ogg",
            Wav2 => "sound/waka/2.ogg",
            Wav3 => "sound/waka/3.ogg",
            Wav4 => "sound/waka/4.ogg",
            FontKonami => "konami.ttf",
            Atlas => "atlas.png",
            AtlasJson => "atlas.json",
        }
    }
}

#[cfg(not(target_os = "emscripten"))]
mod imp {
    use super::*;
    macro_rules! asset_bytes_enum {
        ( $asset:expr ) => {
            match $asset {
                Asset::Wav1 => Cow::Borrowed(include_bytes!("../assets/game/sound/waka/1.ogg")),
                Asset::Wav2 => Cow::Borrowed(include_bytes!("../assets/game/sound/waka/2.ogg")),
                Asset::Wav3 => Cow::Borrowed(include_bytes!("../assets/game/sound/waka/3.ogg")),
                Asset::Wav4 => Cow::Borrowed(include_bytes!("../assets/game/sound/waka/4.ogg")),
                Asset::FontKonami => Cow::Borrowed(include_bytes!("../assets/game/konami.ttf")),
                Asset::Atlas => Cow::Borrowed(include_bytes!("../assets/game/atlas.png")),
                Asset::AtlasJson => Cow::Borrowed(include_bytes!("../assets/game/atlas.json")),
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
    use sdl2::rwops::RWops;
    use std::io::Read;
    pub fn get_asset_bytes(asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
        let path = format!("assets/game/{}", asset.path());
        let mut rwops = RWops::from_file(&path, "rb").map_err(|_| AssetError::NotFound(asset.path().to_string()))?;
        let len = rwops.len().ok_or_else(|| AssetError::NotFound(asset.path().to_string()))?;
        let mut buf = vec![0u8; len];
        rwops
            .read_exact(&mut buf)
            .map_err(|e| AssetError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(Cow::Owned(buf))
    }
}

pub use imp::get_asset_bytes;
