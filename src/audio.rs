use sdl2::{
    mixer::{self, Chunk, InitFlag, LoaderRWops, DEFAULT_FORMAT},
    rwops::RWops,
};

// Embed sound files directly into the executable
const SOUND_1_DATA: &[u8] = include_bytes!("../assets/wav/1.ogg");
const SOUND_2_DATA: &[u8] = include_bytes!("../assets/wav/2.ogg");
const SOUND_3_DATA: &[u8] = include_bytes!("../assets/wav/3.ogg");
const SOUND_4_DATA: &[u8] = include_bytes!("../assets/wav/4.ogg");

const SOUND_DATA: [&[u8]; 4] = [SOUND_1_DATA, SOUND_2_DATA, SOUND_3_DATA, SOUND_4_DATA];

pub struct Audio {
    _mixer_context: mixer::Sdl2MixerContext,
    sounds: Vec<Chunk>,
    next_sound_index: usize,
}

impl Audio {
    pub fn new() -> Self {
        let frequency = 44100;
        let format = DEFAULT_FORMAT;
        let channels = 4;
        let chunk_size = 128;
        mixer::open_audio(frequency, format, 1, chunk_size).expect("Failed to open audio");
        mixer::allocate_channels(channels);

        // set channel volume
        for i in 0..channels {
            mixer::Channel(i as i32).set_volume(32);
        }

        let mixer_context = mixer::init(InitFlag::OGG).expect("Failed to initialize SDL2_mixer");

        let sounds: Vec<Chunk> = SOUND_DATA
            .iter()
            .enumerate()
            .map(|(i, data)| {
                let rwops = RWops::from_bytes(data)
                    .expect(&format!("Failed to create RWops for sound {}", i + 1));
                rwops.load_wav().expect(&format!(
                    "Failed to load sound {} from embedded data",
                    i + 1
                ))
            })
            .collect();

        Audio {
            _mixer_context: mixer_context,
            sounds,
            next_sound_index: 0,
        }
    }

    pub fn eat(&mut self) {
        if let Some(chunk) = self.sounds.get(self.next_sound_index) {
            match mixer::Channel(0).play(chunk, 0) {
                Ok(channel) => {
                    tracing::info!(
                        "Playing sound #{} on channel {:?}",
                        self.next_sound_index + 1,
                        channel
                    );
                }
                Err(e) => {
                    tracing::warn!("Could not play sound #{}: {}", self.next_sound_index + 1, e);
                }
            }
        }
        self.next_sound_index = (self.next_sound_index + 1) % self.sounds.len();
    }
}
