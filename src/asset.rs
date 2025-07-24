//! Cross-platform asset loading abstraction.
//! On desktop, assets are embedded using include_bytes!; on Emscripten, assets are loaded from the filesystem.

use std::borrow::Cow;
use std::io;
use thiserror::Error;

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
    Pacman,
    Pellet,
    Energizer,
    Map,
    FontKonami,
    GhostBody,
    GhostEyes,
    // Add more as needed
}

impl Asset {
    pub fn path(&self) -> &str {
        use Asset::*;
        match self {
            Wav1 => "wav/1.ogg",
            Wav2 => "wav/2.ogg",
            Wav3 => "wav/3.ogg",
            Wav4 => "wav/4.ogg",
            Pacman => "32/pacman.png",
            Pellet => "24/pellet.png",
            Energizer => "24/energizer.png",
            Map => "map.png",
            FontKonami => "font/konami.ttf",
            GhostBody => "32/ghost_body.png",
            GhostEyes => "32/ghost_eyes.png",
        }
    }
}

#[cfg(not(target_os = "emscripten"))]
mod imp {
    use super::*;
    macro_rules! asset_bytes_enum {
        ( $asset:expr ) => {
            match $asset {
                Asset::Wav1 => Cow::Borrowed(include_bytes!("../assets/game/wav/1.ogg")),
                Asset::Wav2 => Cow::Borrowed(include_bytes!("../assets/game/wav/2.ogg")),
                Asset::Wav3 => Cow::Borrowed(include_bytes!("../assets/game/wav/3.ogg")),
                Asset::Wav4 => Cow::Borrowed(include_bytes!("../assets/game/wav/4.ogg")),
                Asset::Pacman => Cow::Borrowed(include_bytes!("../assets/game/32/pacman.png")),
                Asset::Pellet => Cow::Borrowed(include_bytes!("../assets/game/24/pellet.png")),
                Asset::Energizer => Cow::Borrowed(include_bytes!("../assets/game/24/energizer.png")),
                Asset::Map => Cow::Borrowed(include_bytes!("../assets/game/map.png")),
                Asset::FontKonami => Cow::Borrowed(include_bytes!("../assets/game/font/konami.ttf")),
                Asset::GhostBody => Cow::Borrowed(include_bytes!("../assets/game/32/ghost_body.png")),
                Asset::GhostEyes => Cow::Borrowed(include_bytes!("../assets/game/32/ghost_eyes.png")),
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
