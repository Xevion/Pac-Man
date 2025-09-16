//! This module handles the audio playback for the game.
use std::collections::HashMap;

use crate::asset::Asset;
use anyhow::{anyhow, Result};
use sdl2::{
    mixer::{self, Chunk, InitFlag, LoaderRWops, AUDIO_S16LSB},
    rwops::RWops,
};
use strum::IntoEnumIterator;

const AUDIO_FREQUENCY: i32 = 16_000;
const AUDIO_CHANNELS: i32 = 4;
const DEFAULT_VOLUME: u8 = 32;
const WAKA_SOUND_COUNT: u8 = 4;

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
        let mut sounds = vec![
            Sound::PacmanDeath,
            Sound::ExtraLife,
            Sound::Fruit,
            Sound::Ghost,
            Sound::Beginning,
            Sound::Intermission,
        ];
        sounds.extend((0..WAKA_SOUND_COUNT).map(Sound::Waka));
        sounds.into_iter()
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
    state: AudioState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioState {
    Enabled { volume: u8 },
    Muted { previous_volume: u8 },
    Disabled,
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
        match Self::try_new() {
            Ok(audio) => audio,
            Err(e) => {
                tracing::warn!("Failed to initialize audio: {}. Audio will be disabled.", e);
                Self {
                    _mixer_context: None,
                    sounds: HashMap::new(),
                    next_waka_index: 0,
                    state: AudioState::Disabled,
                }
            }
        }
    }

    fn try_new() -> Result<Self> {
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
        mixer::open_audio(AUDIO_FREQUENCY, format, AUDIO_CHANNELS, chunk_size)
            .map_err(|e| anyhow!("Failed to open audio: {}", e))?;

        mixer::allocate_channels(AUDIO_CHANNELS);

        // set channel volume
        for i in 0..AUDIO_CHANNELS {
            mixer::Channel(i).set_volume(DEFAULT_VOLUME as i32);
        }

        // Try to initialize mixer, but don't panic if it fails
        let mixer_context = mixer::init(InitFlag::OGG).map_err(|e| anyhow!("Failed to initialize SDL2_mixer: {}", e))?;

        // Try to load sounds, but don't panic if any fail
        let sounds: HashMap<Sound, Chunk> = Sound::iter()
            .filter_map(|sound_type| match Self::load_sound(sound_type) {
                Ok(chunk) => Some((sound_type, chunk)),
                Err(e) => {
                    tracing::warn!("Failed to load sound {:?}: {}", sound_type, e);
                    None
                }
            })
            .collect();

        // If no sounds loaded successfully, disable audio
        if sounds.is_empty() {
            return Err(anyhow!("No sounds loaded successfully"));
        }

        Ok(Audio {
            _mixer_context: Some(mixer_context),
            sounds,
            next_waka_index: 0u8,
            state: AudioState::Enabled { volume: DEFAULT_VOLUME },
        })
    }

    fn load_sound(sound_type: Sound) -> Result<Chunk> {
        let asset = Asset::SoundFile(sound_type);
        let data = asset
            .get_bytes()
            .map_err(|e| anyhow!("Failed to get bytes for {:?}: {}", sound_type, e))?;
        let rwops = RWops::from_bytes(&data).map_err(|e| anyhow!("Failed to create RWops for {:?}: {}", sound_type, e))?;
        rwops
            .load_wav()
            .map_err(|e| anyhow!("Failed to load wav for {:?}: {}", sound_type, e))
    }

    /// Plays the next waka eating sound in the cycle of four variants.
    ///
    /// Automatically rotates through the four eating sound assets. The sound plays on channel 0 and the internal sound index
    /// advances to the next variant. Silently returns if audio is disabled, muted,
    /// or no sounds were loaded successfully.
    pub fn waka(&mut self) {
        if !matches!(self.state, AudioState::Enabled { .. }) {
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
        self.next_waka_index = (self.next_waka_index + 1) % WAKA_SOUND_COUNT;
    }

    /// Plays the provided sound effect once.
    pub fn play(&mut self, sound: Sound) {
        if !matches!(self.state, AudioState::Enabled { .. }) {
            return;
        }

        if let Some(chunk) = self.sounds.get(&sound) {
            let _ = mixer::Channel::all().play(chunk, 0);
        }
    }

    /// Halts all currently playing audio channels.
    pub fn stop_all(&mut self) {
        if self.state != AudioState::Disabled {
            mixer::Channel::all().halt();
        }
    }

    /// Pauses all currently playing audio channels.
    pub fn pause_all(&mut self) {
        if self.state != AudioState::Disabled {
            mixer::Channel::all().pause();
        }
    }

    /// Resumes all currently playing audio channels.
    pub fn resume_all(&mut self) {
        if self.state != AudioState::Disabled {
            mixer::Channel::all().resume();
        }
    }

    /// Instantly mutes or unmutes all audio channels by adjusting their volume.
    ///
    /// Sets all 4 mixer channels to zero volume when muting, or restores them to
    /// their default volume (32) when unmuting. The mute state is tracked internally
    /// regardless of whether audio is disabled, allowing the state to be preserved.
    pub fn set_mute(&mut self, mute: bool) {
        match (mute, self.state) {
            // Mute
            (true, AudioState::Enabled { volume }) => {
                self.state = AudioState::Muted { previous_volume: volume };
                for i in 0..AUDIO_CHANNELS {
                    mixer::Channel(i).set_volume(0);
                }
            }
            // Unmute
            (false, AudioState::Muted { previous_volume }) => {
                self.state = AudioState::Enabled { volume: previous_volume };
                for i in 0..AUDIO_CHANNELS {
                    mixer::Channel(i).set_volume(previous_volume as i32);
                }
            }
            _ => {}
        }
    }

    /// Returns the current mute state regardless of whether audio is functional.
    ///
    /// This tracks the user's mute preference and will return `true` if muted
    /// even when the audio system is disabled due to initialization failures.
    pub fn is_muted(&self) -> bool {
        matches!(self.state, AudioState::Muted { .. })
    }

    /// Returns whether the audio system failed to initialize and is non-functional.
    ///
    /// Audio can be disabled due to SDL2_mixer initialization failures, missing
    /// audio device, or failure to load any sound assets. When disabled, all
    /// audio operations become no-ops.
    pub fn is_disabled(&self) -> bool {
        matches!(self.state, AudioState::Disabled)
    }
}
