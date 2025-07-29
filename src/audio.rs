//! This module handles the audio playback for the game.
use crate::asset::{get_asset_bytes, Asset};
use sdl2::{
    mixer::{self, Chunk, InitFlag, LoaderRWops, DEFAULT_FORMAT},
    rwops::RWops,
};

const SOUND_ASSETS: [Asset; 4] = [Asset::Wav1, Asset::Wav2, Asset::Wav3, Asset::Wav4];

/// The audio system for the game.
///
/// This struct is responsible for initializing the audio device, loading sounds,
/// and playing them.
#[allow(dead_code)]
pub struct Audio {
    _mixer_context: mixer::Sdl2MixerContext,
    sounds: Vec<Chunk>,
    next_sound_index: usize,
    muted: bool,
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

impl Audio {
    /// Creates a new `Audio` instance.
    pub fn new() -> Self {
        let frequency = 44100;
        let format = DEFAULT_FORMAT;
        let channels = 4;
        let chunk_size = 256; // 256 is minimum for emscripten

        mixer::open_audio(frequency, format, 1, chunk_size).expect("Failed to open audio");
        mixer::allocate_channels(channels);

        // set channel volume
        for i in 0..channels {
            mixer::Channel(i).set_volume(32);
        }

        let mixer_context = mixer::init(InitFlag::OGG).expect("Failed to initialize SDL2_mixer");

        let sounds: Vec<Chunk> = SOUND_ASSETS
            .iter()
            .enumerate()
            .map(|(i, asset)| {
                let data = get_asset_bytes(*asset).expect("Failed to load sound asset");
                let rwops = RWops::from_bytes(&data).unwrap_or_else(|_| panic!("Failed to create RWops for sound {}", i + 1));
                rwops
                    .load_wav()
                    .unwrap_or_else(|_| panic!("Failed to load sound {} from asset API", i + 1))
            })
            .collect();

        Audio {
            _mixer_context: mixer_context,
            sounds,
            next_sound_index: 0,
            muted: false,
        }
    }

    /// Plays the "eat" sound effect.
    #[allow(dead_code)]
    pub fn eat(&mut self) {
        if let Some(chunk) = self.sounds.get(self.next_sound_index) {
            match mixer::Channel(0).play(chunk, 0) {
                Ok(channel) => {
                    tracing::trace!("Playing sound #{} on channel {:?}", self.next_sound_index + 1, channel);
                }
                Err(e) => {
                    tracing::warn!("Could not play sound #{}: {}", self.next_sound_index + 1, e);
                }
            }
        }
        self.next_sound_index = (self.next_sound_index + 1) % self.sounds.len();
    }

    /// Instantly mute or unmute all channels.
    pub fn set_mute(&mut self, mute: bool) {
        let channels = 4;
        let volume = if mute { 0 } else { 32 };
        for i in 0..channels {
            mixer::Channel(i).set_volume(volume);
        }
        self.muted = mute;
    }

    /// Returns `true` if the audio is muted.
    pub fn is_muted(&self) -> bool {
        self.muted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_sdl() -> Result<(), String> {
        INIT.call_once(|| {
            if let Err(e) = sdl2::init() {
                eprintln!("Failed to initialize SDL2: {}", e);
            }
        });
        Ok(())
    }

    #[test]
    fn test_sound_assets_array() {
        assert_eq!(SOUND_ASSETS.len(), 4);
        assert_eq!(SOUND_ASSETS[0], Asset::Wav1);
        assert_eq!(SOUND_ASSETS[1], Asset::Wav2);
        assert_eq!(SOUND_ASSETS[2], Asset::Wav3);
        assert_eq!(SOUND_ASSETS[3], Asset::Wav4);
    }

    #[test]
    fn test_audio_asset_paths() {
        // Test that all sound assets have valid paths
        for asset in SOUND_ASSETS.iter() {
            let path = asset.path();
            assert!(!path.is_empty());
            assert!(path.contains("sound/waka/"));
            assert!(path.ends_with(".ogg"));
        }
    }

    // Only run SDL2-dependent tests if SDL2 initialization succeeds
    #[test]
    fn test_audio_basic_functionality() {
        if let Err(_) = init_sdl() {
            eprintln!("Skipping SDL2-dependent tests due to initialization failure");
            return;
        }

        // Test basic audio creation
        let audio = Audio::new();
        assert_eq!(audio.is_muted(), false);
        assert_eq!(audio.next_sound_index, 0);
        assert_eq!(audio.sounds.len(), 4);
    }

    #[test]
    fn test_audio_mute_functionality() {
        if let Err(_) = init_sdl() {
            eprintln!("Skipping SDL2-dependent tests due to initialization failure");
            return;
        }

        let mut audio = Audio::new();

        // Test mute/unmute
        assert_eq!(audio.is_muted(), false);
        audio.set_mute(true);
        assert_eq!(audio.is_muted(), true);
        audio.set_mute(false);
        assert_eq!(audio.is_muted(), false);
    }

    #[test]
    fn test_audio_sound_rotation() {
        if let Err(_) = init_sdl() {
            eprintln!("Skipping SDL2-dependent tests due to initialization failure");
            return;
        }

        let mut audio = Audio::new();
        let initial_index = audio.next_sound_index;

        // Test sound rotation
        for i in 0..4 {
            audio.eat();
            assert_eq!(audio.next_sound_index, (initial_index + i + 1) % 4);
        }

        assert_eq!(audio.next_sound_index, initial_index);
    }

    #[test]
    fn test_audio_sound_index_bounds() {
        if let Err(_) = init_sdl() {
            eprintln!("Skipping SDL2-dependent tests due to initialization failure");
            return;
        }

        let audio = Audio::new();
        assert!(audio.next_sound_index < audio.sounds.len());
    }

    #[test]
    fn test_audio_default_impl() {
        if let Err(_) = init_sdl() {
            eprintln!("Skipping SDL2-dependent tests due to initialization failure");
            return;
        }

        let audio = Audio::default();
        assert_eq!(audio.is_muted(), false);
        assert_eq!(audio.next_sound_index, 0);
        assert_eq!(audio.sounds.len(), 4);
    }
}
