//! Audio system for handling sound playback in the Pac-Man game.
//!
//! This module provides an ECS-based audio system that integrates with SDL2_mixer
//! for playing sound effects. The system uses NonSendMut resources to handle SDL2's
//! main-thread requirements while maintaining Bevy ECS compatibility.

use bevy_ecs::{
    event::{Event, EventReader},
    system::NonSendMut,
};
use tracing::{debug, trace};

use crate::{audio::Audio, audio::Sound};

/// Events for triggering audio playback
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEvent {
    /// Play a specific sound effect
    PlaySound(Sound),
    /// Play the cycling waka sound variant
    Waka,
    /// Stop all currently playing sounds
    StopAll,
    /// Pause all sounds
    Pause,
    /// Resume all sounds
    Resume,
    /// Toggle mute on/off
    ToggleMute,
}

/// Non-send resource wrapper for SDL2 audio system
///
/// This wrapper is needed because SDL2 audio components are not Send,
/// but Bevy ECS requires Send for regular resources. Using NonSendMut
/// allows us to use SDL2 audio on the main thread while integrating
/// with the ECS system.
pub struct AudioResource(pub Audio);

/// System that processes audio events and plays sounds.
///
/// The `Audio` resource's internal state is the single source of truth for mute,
/// volume, and waka cycling. This system simply dispatches events to it.
pub fn audio_system(mut audio: NonSendMut<AudioResource>, mut events: EventReader<AudioEvent>) {
    for event in events.read() {
        match event {
            AudioEvent::Waka => {
                trace!("Playing eat sound");
                audio.0.waka();
            }
            AudioEvent::PlaySound(sound) => {
                trace!(?sound, "Playing sound");
                audio.0.play(*sound);
            }
            AudioEvent::StopAll => {
                debug!("Stopping all audio");
                audio.0.stop_all();
            }
            AudioEvent::Pause => {
                debug!("Pausing all audio");
                audio.0.pause_all();
            }
            AudioEvent::Resume => {
                debug!("Resuming all audio");
                audio.0.resume_all();
            }
            AudioEvent::ToggleMute => {
                let muted = !audio.0.is_muted();
                audio.0.set_mute(muted);
                debug!(muted, "Audio mute toggled");
            }
        }
    }
}
