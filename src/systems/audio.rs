//! Audio system for handling sound playback in the Pac-Man game.
//!
//! This module provides an ECS-based audio system that integrates with SDL2_mixer
//! for playing sound effects. The system uses NonSendMut resources to handle SDL2's
//! main-thread requirements while maintaining Bevy ECS compatibility.

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    resource::Resource,
    system::{NonSendMut, ResMut},
};

use crate::{audio::Audio, error::GameError};

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
    /// Play the "eat" sound when Pac-Man consumes a pellet
    PlayEat,
}

/// Non-send resource wrapper for SDL2 audio system
///
/// This wrapper is needed because SDL2 audio components are not Send,
/// but Bevy ECS requires Send for regular resources. Using NonSendMut
/// allows us to use SDL2 audio on the main thread while integrating
/// with the ECS system.
pub struct AudioResource(pub Audio);

/// System that processes audio events and plays sounds
pub fn audio_system(
    mut audio: NonSendMut<AudioResource>,
    mut audio_state: ResMut<AudioState>,
    mut audio_events: EventReader<AudioEvent>,
    _errors: EventWriter<GameError>,
) {
    // Set mute state if it has changed
    if audio.0.is_muted() != audio_state.muted {
        audio.0.set_mute(audio_state.muted);
    }

    // Process audio events
    for event in audio_events.read() {
        match event {
            AudioEvent::PlayEat => {
                if !audio.0.is_disabled() && !audio_state.muted {
                    audio.0.eat();
                    // Update the sound index for cycling through sounds
                    audio_state.sound_index = (audio_state.sound_index + 1) % 4;
                    // 4 eat sounds available
                }
            }
        }
    }
}
