#![allow(dead_code)]
//! Cross-platform asset loading abstraction.
//! On desktop, assets are embedded using include_bytes!; on Emscripten, assets are loaded from the filesystem.

use std::borrow::Cow;
use strum_macros::EnumIter;

/// Enumeration of all game assets with cross-platform loading support.
///
/// Each variant corresponds to a specific file that can be loaded either from
/// binary-embedded data or embedded filesystem (Emscripten).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Asset {
    Wav1,
    Wav2,
    Wav3,
    Wav4,
    /// Main sprite atlas containing all game graphics (atlas.png)
    AtlasImage,
    /// Terminal Vector font for text rendering (TerminalVector.ttf)
    Font,
    /// Sound effect for Pac-Man's death
    DeathSound,
}

impl Asset {
    /// Returns the relative file path for this asset within the game's asset directory.
    ///
    /// Paths are consistent across platforms and used by the Emscripten backend
    /// for filesystem loading. Desktop builds embed assets directly and don't
    /// use these paths at runtime.
    #[allow(dead_code)]
    pub fn path(&self) -> &str {
        use Asset::*;
        match self {
            Wav1 => "sound/waka/1.ogg",
            Wav2 => "sound/waka/2.ogg",
            Wav3 => "sound/waka/3.ogg",
            Wav4 => "sound/waka/4.ogg",
            AtlasImage => "atlas.png",
            Font => "TerminalVector.ttf",
            DeathSound => "sound/pacman_death.wav",
        }
    }
}

mod imp {
    use super::*;
    use crate::error::AssetError;
    use crate::platform;
    use tracing::trace;

    /// Loads asset bytes using the appropriate platform-specific method.
    ///
    /// On desktop platforms, returns embedded compile-time data via `include_bytes!`.
    /// On Emscripten, loads from the filesystem using the asset's path. The returned
    /// `Cow` allows zero-copy access to embedded data while supporting owned data
    /// when loaded from disk.
    ///
    /// # Errors
    ///
    /// Returns `AssetError::NotFound` if the asset file cannot be located (Emscripten only),
    /// or `AssetError::Io` for filesystem I/O failures.
    pub fn get_asset_bytes(asset: Asset) -> Result<Cow<'static, [u8]>, AssetError> {
        trace!(asset = ?asset, path = asset.path(), "Loading game asset");
        let result = platform::get_asset_bytes(asset);
        match &result {
            Ok(bytes) => trace!(asset = ?asset, size_bytes = bytes.len(), "Asset loaded successfully"),
            Err(e) => trace!(asset = ?asset, error = ?e, "Asset loading failed"),
        }
        result
    }
}

pub use imp::get_asset_bytes;
