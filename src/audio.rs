//! This module handles the audio playback for the game.
use std::collections::HashMap;

use crate::asset::Asset;
use sdl2::{
    mixer::{self, Chunk, InitFlag, LoaderRWops, AUDIO_S16LSB, DEFAULT_CHANNELS},
    rwops::RWops,
};
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sound {
    Waka(u8),
    PacmanDeath,
    ExtraLife,
    Fruit,
    Ghost,
    Beginning,
    Intermission,
}

impl IntoEnumIterator for Sound {
    type Iterator = std::vec::IntoIter<Sound>;

    fn iter() -> Self::Iterator {
        vec![
            Sound::Waka(0),
            Sound::Waka(1),
            Sound::Waka(2),
            Sound::Waka(3),
            Sound::PacmanDeath,
            Sound::ExtraLife,
            Sound::Fruit,
            Sound::Ghost,
            Sound::Beginning,
            Sound::Intermission,
        ]
        .into_iter()
    }
}

/// The audio system for the game.
///
/// This struct is responsible for initializing the audio device, loading sounds,
/// and playing them. If audio fails to initialize, it will be disabled and all
/// functions will silently do nothing.
pub struct Audio {
    _mixer_context: Option<mixer::Sdl2MixerContext>,
    sounds: HashMap<Sound, Chunk>,
    next_waka_index: u8,
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
        let frequency = 16_000;
        let format = AUDIO_S16LSB;
        let chunk_size = {
            // 256 is the minimum for Emscripten, but in practice 1024 is much more reliable
            #[cfg(target_os = "emscripten")]
            {
                1024
            }

            // Otherwise, 256 is plenty safe.
            #[cfg(not(target_os = "emscripten"))]
            {
                256
            }
        };

        // Try to open audio, but don't panic if it fails
        if let Err(e) = mixer::open_audio(frequency, format, DEFAULT_CHANNELS, chunk_size) {
            tracing::warn!("Failed to open audio: {}. Audio will be disabled.", e);
            return Self {
                _mixer_context: None,
                sounds: HashMap::new(),
                next_waka_index: 0u8,
                muted: false,
                disabled: true,
            };
        }

        let channels = 4;
        mixer::allocate_channels(4);

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
                    sounds: HashMap::new(),
                    next_waka_index: 0u8,
                    muted: false,
                    disabled: true,
                };
            }
        };

        // Try to load sounds, but don't panic if any fail
        let mut sounds = HashMap::new();
        for (i, sound_type) in Sound::iter().enumerate() {
            let asset = Asset::SoundFile(sound_type);
            match asset.get_bytes() {
                Ok(data) => match RWops::from_bytes(&data) {
                    Ok(rwops) => match rwops.load_wav() {
                        Ok(chunk) => {
                            sounds.insert(sound_type, chunk);
                        }
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

        let death_sound = match Asset::SoundFile(Sound::PacmanDeath).get_bytes() {
            Ok(data) => match RWops::from_bytes(&data) {
                Ok(rwops) => match rwops.load_wav() {
                    Ok(chunk) => Some(chunk),
                    Err(e) => {
                        tracing::warn!("Failed to load death sound from asset API: {}", e);
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to create RWops for death sound: {}", e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Failed to load death sound asset: {}", e);
                None
            }
        };

        // If no sounds loaded successfully, disable audio
        if sounds.is_empty() && death_sound.is_none() {
            tracing::warn!("No sounds loaded successfully. Audio will be disabled.");
            return Self {
                _mixer_context: Some(mixer_context),
                sounds: HashMap::new(),
                next_waka_index: 0u8,
                muted: false,
                disabled: true,
            };
        }

        Audio {
            _mixer_context: Some(mixer_context),
            sounds,
            next_waka_index: 0u8,
            muted: false,
            disabled: false,
        }
    }

    /// Plays the next waka eating sound in the cycle of four variants.
    ///
    /// Automatically rotates through the four eating sound assets. The sound plays on channel 0 and the internal sound index
    /// advances to the next variant. Silently returns if audio is disabled, muted,
    /// or no sounds were loaded successfully.
    pub fn waka(&mut self) {
        if self.disabled || self.muted || self.sounds.is_empty() {
            return;
        }

        if let Some(chunk) = self.sounds.get(&Sound::Waka(self.next_waka_index)) {
            match mixer::Channel::all().play(chunk, 0) {
                Ok(channel) => {
                    tracing::trace!("Playing sound #{} on channel {:?}", self.next_waka_index + 1, channel);
                }
                Err(e) => {
                    tracing::warn!("Could not play sound #{}: {}", self.next_waka_index + 1, e);
                }
            }
        }
        self.next_waka_index = (self.next_waka_index + 1) & 3;
    }

    /// Plays the provided sound effect once.
    pub fn play(&mut self, sound: Sound) {
        if self.disabled || self.muted {
            return;
        }

        if let Some(chunk) = self.sounds.get(&sound) {
            let _ = mixer::Channel::all().play(chunk, 0);
        }
    }

    /// Halts all currently playing audio channels.
    pub fn stop_all(&mut self) {
        if !self.disabled {
            mixer::Channel::all().halt();
        }
    }

    /// Pauses all currently playing audio channels.
    pub fn pause_all(&mut self) {
        if !self.disabled {
            mixer::Channel::all().pause();
        }
    }

    /// Resumes all currently playing audio channels.
    pub fn resume_all(&mut self) {
        if !self.disabled {
            mixer::Channel::all().resume();
        }
    }

    /// Instantly mutes or unmutes all audio channels by adjusting their volume.
    ///
    /// Sets all 4 mixer channels to zero volume when muting, or restores them to
    /// their default volume (32) when unmuting. The mute state is tracked internally
    /// regardless of whether audio is disabled, allowing the state to be preserved.
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

    /// Returns the current mute state regardless of whether audio is functional.
    ///
    /// This tracks the user's mute preference and will return `true` if muted
    /// even when the audio system is disabled due to initialization failures.
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Returns whether the audio system failed to initialize and is non-functional.
    ///
    /// Audio can be disabled due to SDL2_mixer initialization failures, missing
    /// audio device, or failure to load any sound assets. When disabled, all
    /// audio operations become no-ops.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}
