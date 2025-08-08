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
        if !self.disabled {
            let channels = 4;
            let volume = if mute { 0 } else { 32 };
            for i in 0..channels {
                mixer::Channel(i).set_volume(volume);
            }
        }

        self.muted = mute;
    }

    /// Returns `true` if the audio is muted.
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Returns `true` if the audio system is disabled.
    #[allow(dead_code)]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}
