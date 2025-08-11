#![allow(dead_code)]
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
            Atlas => "atlas.png",
            AtlasJson => "atlas.json",
        }
    }
}

mod imp {
    use super::*;
    use crate::platform::get_platform;

    pub fn get_asset_bytes(asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
        get_platform().get_asset_bytes(asset)
    }
}

pub use imp::get_asset_bytes;
