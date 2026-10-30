//! Audio playback against SDL's `dummy` driver: the mixer runs for real but nothing is audible.

use assert2::{assert, check};
use pacman::audio::{Audio, Sound};
use sdl2::Sdl;
use sdl2::mixer::{self, Channel};
use strum::IntoEnumIterator;

mod common;

/// Opens audio on the dummy driver. The returned SDL context must outlive the test, since
/// dropping it shuts the audio subsystem down underneath the mixer.
fn open_audio() -> (Audio, Sdl) {
    common::silence_audio();
    let sdl = sdl2::init().unwrap();
    let driver = sdl.audio().unwrap().current_audio_driver();
    assert!(driver == "dummy", "tests must never reach a real audio device");

    let audio = Audio::new();
    assert!(audio.is_available(), "the dummy driver should always open");
    (audio, sdl)
}

fn channel_volumes() -> Vec<i32> {
    (0..mixer::allocate_channels(-1)).map(|i| Channel(i).get_volume()).collect()
}

#[test]
fn every_sound_loads_and_plays() {
    let (mut audio, _sdl) = open_audio();
    for sound in Sound::iter() {
        audio.stop_all();
        audio.play(sound);
        check!(mixer::get_playing_channels_number() == 1, "{sound:?} did not start playing");
    }
}

#[test]
fn each_waka_starts_a_channel() {
    let (mut audio, _sdl) = open_audio();
    for expected in 1..=4 {
        audio.waka();
        check!(mixer::get_playing_channels_number() == expected);
    }
}

#[test]
fn stop_all_halts_every_channel() {
    let (mut audio, _sdl) = open_audio();
    audio.play(Sound::Beginning);
    audio.play(Sound::Intermission);
    assert!(mixer::get_playing_channels_number() == 2);

    audio.stop_all();
    check!(mixer::get_playing_channels_number() == 0);
}

#[test]
fn pause_and_resume_apply_to_playing_channels() {
    let (mut audio, _sdl) = open_audio();
    audio.play(Sound::Beginning);

    audio.pause_all();
    check!(mixer::get_paused_channels_number() == 1);

    audio.resume_all();
    check!(mixer::get_paused_channels_number() == 0);
    check!(mixer::get_playing_channels_number() == 1);
}

#[test]
fn mute_zeroes_volume_blocks_playback_and_unmute_restores() {
    let (mut audio, _sdl) = open_audio();
    let original = channel_volumes();
    assert!(original.iter().all(|&v| v > 0));

    audio.set_mute(true);
    check!(audio.is_muted());
    check!(channel_volumes().iter().all(|&v| v == 0));

    audio.play(Sound::Fruit);
    audio.waka();
    check!(mixer::get_playing_channels_number() == 0);

    audio.set_mute(false);
    check!(!audio.is_muted());
    check!(channel_volumes() == original);
}
