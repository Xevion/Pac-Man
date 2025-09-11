//! Cross-platform asset loading abstraction.
//! On desktop, assets are embedded using include_bytes!; on Emscripten, assets are loaded from the filesystem.

use std::borrow::Cow;
use std::iter;

use crate::audio::Sound;
use crate::error::AssetError;

/// Enumeration of all game assets with cross-platform loading support.
///
/// Each variant corresponds to a specific file that can be loaded either from
/// binary-embedded data or embedded filesystem (Emscripten).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asset {
    /// Main sprite atlas containing all game graphics (atlas.png)
    AtlasImage,
    /// Terminal Vector font for text rendering (TerminalVector.ttf)
    Font,
    /// Sound file assets
    SoundFile(Sound),
}

use strum::IntoEnumIterator;

impl Asset {
    #[allow(dead_code)]
    pub fn into_iter() -> AssetIter {
        AssetIter {
            sound_iter: None,
            state: 0,
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct AssetIter {
    sound_iter: Option<iter::Map<<Sound as IntoEnumIterator>::Iterator, fn(Sound) -> Asset>>,
    state: u8,
}

impl Iterator for AssetIter {
    type Item = Asset;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            0 => {
                self.state = 1;
                Some(Asset::AtlasImage)
            }
            1 => {
                self.state = 2;
                Some(Asset::Font)
            }
            2 => self
                .sound_iter
                .get_or_insert_with(|| Sound::iter().map(Asset::SoundFile))
                .next(),
            _ => None,
        }
    }
}

impl Asset {
    /// Returns the relative file path for this asset within the game's asset directory.
    ///
    /// Paths are consistent across platforms and used by the Emscripten backend
    /// for filesystem loading. Desktop builds embed assets directly and don't
    /// use these paths at runtime.
    pub fn path(&self) -> &str {
        use Asset::*;
        match self {
            SoundFile(Sound::Waka(0)) => "sound/pacman/waka/1.ogg",
            SoundFile(Sound::Waka(1)) => "sound/pacman/waka/2.ogg",
            SoundFile(Sound::Waka(2)) => "sound/pacman/waka/3.ogg",
            SoundFile(Sound::Waka(3..=u8::MAX)) => "sound/pacman/waka/4.ogg",
            SoundFile(Sound::PacmanDeath) => "sound/pacman/death.ogg",
            SoundFile(Sound::ExtraLife) => "sound/pacman/extra_life.ogg",
            SoundFile(Sound::Fruit) => "sound/pacman/fruit.ogg",
            SoundFile(Sound::Ghost) => "sound/pacman/ghost.ogg",
            SoundFile(Sound::Beginning) => "sound/begin.ogg",
            SoundFile(Sound::Intermission) => "sound/intermission.ogg",
            AtlasImage => "atlas.png",
            Font => "TerminalVector.ttf",
        }
    }

    /// Loads asset bytes using the appropriate platform-specific method.
    ///
    /// On desktop platforms, returns embedded compile-time data via `rust-embed`.
    /// On Emscripten, loads from the filesystem using the asset's path. The returned
    /// `Cow` allows zero-copy access to embedded data while supporting owned data
    /// when loaded from disk.
    ///
    /// # Errors
    ///
    /// Returns `AssetError::NotFound` if the asset file cannot be located,
    /// or `AssetError::Io` for filesystem I/O failures.
    pub fn get_bytes(&self) -> Result<Cow<'static, [u8]>, AssetError> {
        use tracing::trace;
        trace!(asset = ?self, "Loading game asset");
        let result = self.get_bytes_platform();
        match &result {
            Ok(bytes) => trace!(asset = ?self, size_bytes = bytes.len(), "Asset loaded successfully"),
            Err(e) => trace!(asset = ?self, error = ?e, "Asset loading failed"),
        }
        result
    }

    #[cfg(not(target_os = "emscripten"))]
    fn get_bytes_platform(&self) -> Result<Cow<'static, [u8]>, AssetError> {
        #[derive(rust_embed::Embed)]
        #[folder = "assets/game/"]
        struct EmbeddedAssets;

        let path = self.path();
        EmbeddedAssets::get(path)
            .map(|file| file.data)
            .ok_or_else(|| AssetError::NotFound(path.to_string()))
    }

    #[cfg(target_os = "emscripten")]
    fn get_bytes_platform(&self) -> Result<Cow<'static, [u8]>, AssetError> {
        use sdl2::rwops::RWops;
        use std::io::{self, Read};

        let path = format!("assets/game/{}", self.path());
        let mut rwops = RWops::from_file(&path, "rb").map_err(|_| AssetError::NotFound(self.path().to_string()))?;

        let len = rwops.len().ok_or_else(|| AssetError::NotFound(self.path().to_string()))?;

        let mut buf = vec![0u8; len];
        rwops.read_exact(&mut buf).map_err(|e| AssetError::Io(io::Error::other(e)))?;

        Ok(Cow::Owned(buf))
    }
}
