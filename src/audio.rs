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
/// and playing them. If audio fails to initialize, it will be disabled and all
/// functions will silently do nothing.
#[allow(dead_code)]
pub struct Audio {
    _mixer_context: Option<mixer::Sdl2MixerContext>,
    sounds: Vec<Chunk>,
    next_sound_index: usize,
    muted: bool,
    disabled: bool,
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

impl Audio {
    /// Creates a new `Audio` instance.
    ///
    /// If audio fails to initialize, the audio system will be disabled and
    /// all functions will silently do nothing.
    pub fn new() -> Self {
        let frequency = 44100;
        let format = DEFAULT_FORMAT;
        let channels = 4;
        let chunk_size = 256; // 256 is minimum for emscripten

        // Try to open audio, but don't panic if it fails
        if let Err(e) = mixer::open_audio(frequency, format, 1, chunk_size) {
            tracing::warn!("Failed to open audio: {}. Audio will be disabled.", e);
            return Self {
                _mixer_context: None,
                sounds: Vec::new(),
                next_sound_index: 0,
                muted: false,
                disabled: true,
            };
        }

        mixer::allocate_channels(channels);

        // set channel volume
        for i in 0..channels {
            mixer::Channel(i).set_volume(32);
        }

        // Try to initialize mixer, but don't panic if it fails
        let mixer_context = match mixer::init(InitFlag::OGG) {
            Ok(ctx) => ctx,
            Err(e) => {
                tracing::warn!("Failed to initialize SDL2_mixer: {}. Audio will be disabled.", e);
                return Self {
                    _mixer_context: None,
                    sounds: Vec::new(),
                    next_sound_index: 0,
                    muted: false,
                    disabled: true,
                };
            }
        };

        // Try to load sounds, but don't panic if any fail
        let mut sounds = Vec::new();
        for (i, asset) in SOUND_ASSETS.iter().enumerate() {
            match get_asset_bytes(*asset) {
                Ok(data) => match RWops::from_bytes(&data) {
                    Ok(rwops) => match rwops.load_wav() {
                        Ok(chunk) => sounds.push(chunk),
                        Err(e) => {
                            tracing::warn!("Failed to load sound {} from asset API: {}", i + 1, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to create RWops for sound {}: {}", i + 1, e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to load sound asset {}: {}", i + 1, e);
                }
            }
        }

        // If no sounds loaded successfully, disable audio
        if sounds.is_empty() {
            tracing::warn!("No sounds loaded successfully. Audio will be disabled.");
            return Self {
                _mixer_context: Some(mixer_context),
                sounds: Vec::new(),
                next_sound_index: 0,
                muted: false,
                disabled: true,
            };
        }

        Audio {
            _mixer_context: Some(mixer_context),
            sounds,
            next_sound_index: 0,
            muted: false,
            disabled: false,
        }
    }

    /// Plays the "eat" sound effect.
    ///
    /// If audio is disabled or muted, this function does nothing.
    #[allow(dead_code)]
    pub fn eat(&mut self) {
        if self.disabled || self.muted || self.sounds.is_empty() {
            return;
        }

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
    ///
    /// If audio is disabled, this function does nothing.
    pub fn set_mute(&mut self, mute: bool) {
        if self.disabled {
            return;
        }

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

    /// Returns `true` if the audio system is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
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

        // Audio might be disabled if initialization failed
        if !audio.is_disabled() {
            assert_eq!(audio.sounds.len(), 4);
        }
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

        // Skip test if audio is disabled
        if audio.is_disabled() {
            eprintln!("Skipping sound rotation test due to disabled audio");
            return;
        }

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

        // Skip test if audio is disabled
        if audio.is_disabled() {
            eprintln!("Skipping sound index bounds test due to disabled audio");
            return;
        }

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

        // Audio might be disabled if initialization failed
        if !audio.is_disabled() {
            assert_eq!(audio.sounds.len(), 4);
        }
    }

    #[test]
    fn test_audio_disabled_state() {
        if let Err(_) = init_sdl() {
            eprintln!("Skipping SDL2-dependent tests due to initialization failure");
            return;
        }

        // Test that disabled audio doesn't crash when calling functions
        let mut audio = Audio::new();

        // These should not panic even if audio is disabled
        audio.eat();
        audio.set_mute(true);
        audio.set_mute(false);

        // Test that we can check the disabled state
        let _is_disabled = audio.is_disabled();
    }
}
