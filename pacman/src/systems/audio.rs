//! Audio system for handling sound playback in the Pac-Man game.
//!
//! This module provides an ECS-based audio system that integrates with SDL2_mixer
//! for playing sound effects. The system uses NonSendMut resources to handle SDL2's
//! main-thread requirements while maintaining Bevy ECS compatibility.

use bevy_ecs::{
    event::{Event, EventReader},
    resource::Resource,
    system::{NonSendMut, ResMut},
};
use tracing::{debug, trace};

use crate::{audio::Audio, audio::Sound};

/// Resource for tracking audio state
#[derive(Resource, Debug, Clone, Default)]
pub struct AudioState {
    /// Whether audio is currently muted
    pub muted: bool,
    /// Current sound index for cycling through eat sounds
    pub sound_index: usize,
}

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
}

/// Non-send resource wrapper for SDL2 audio system
///
/// This wrapper is needed because SDL2 audio components are not Send,
/// but Bevy ECS requires Send for regular resources. Using NonSendMut
/// allows us to use SDL2 audio on the main thread while integrating
/// with the ECS system.
pub struct AudioResource(pub Audio);

/// System that processes audio events and plays sounds
pub fn audio_system(mut audio: NonSendMut<AudioResource>, mut state: ResMut<AudioState>, mut events: EventReader<AudioEvent>) {
    // Set mute state if it has changed
    if audio.0.is_muted() != state.muted {
        debug!(muted = state.muted, "Audio mute state changed");
        audio.0.set_mute(state.muted);
    }

    // Process audio events
    for event in events.read() {
        match event {
            AudioEvent::Waka => {
                if !audio.0.is_disabled() && !state.muted {
                    trace!(sound_index = state.sound_index, "Playing eat sound");
                    audio.0.waka();
                    // Update the sound index for cycling through sounds
                    state.sound_index = (state.sound_index + 1) % 4;
                    // 4 eat sounds available
                } else {
                    debug!(
                        disabled = audio.0.is_disabled(),
                        muted = state.muted,
                        "Skipping eat sound due to audio state"
                    );
                }
            }
            AudioEvent::PlaySound(sound) => {
                if !audio.0.is_disabled() && !state.muted {
                    trace!(?sound, "Playing sound");
                    audio.0.play(*sound);
                } else {
                    debug!(
                        disabled = audio.0.is_disabled(),
                        muted = state.muted,
                        "Skipping sound due to audio state"
                    );
                }
            }
            AudioEvent::StopAll => {
                if !audio.0.is_disabled() {
                    debug!("Stopping all audio");
                    audio.0.stop_all();
                } else {
                    debug!("Audio disabled, ignoring stop all request");
                }
            }
            AudioEvent::Pause => {
                if !audio.0.is_disabled() {
                    debug!("Pausing all audio");
                    audio.0.pause_all();
                } else {
                    debug!("Audio disabled, ignoring pause all request");
                }
            }
            AudioEvent::Resume => {
                if !audio.0.is_disabled() {
                    debug!("Resuming all audio");
                    audio.0.resume_all();
                } else {
                    debug!("Audio disabled, ignoring resume all request");
                }
            }
        }
    }
}
