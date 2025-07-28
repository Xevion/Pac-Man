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
