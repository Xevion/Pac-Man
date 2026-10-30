//! Audio falls back to a silent no-op when no device can be opened.
//!
//! Kept in its own test binary because SDL picks the audio driver once per process.

use assert2::{assert, check};
use pacman::audio::{Audio, Sound};

#[test]
fn unavailable_driver_disables_audio_and_every_call_is_a_no_op() {
    sdl2::hint::set("SDL_AUDIODRIVER", "pacman-test-missing-driver");
    let mut audio = Audio::new();
    assert!(!audio.is_available());

    audio.play(Sound::Beginning);
    audio.waka();
    audio.pause_all();
    audio.resume_all();
    audio.stop_all();

    audio.set_mute(true);
    check!(!audio.is_muted());
}
