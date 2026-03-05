use tracing::info;

use bevy_ecs::{
    event::{EventReader, EventWriter},
    resource::Resource,
    system::ResMut,
};

use crate::events::{GameCommand, GameEvent};
use crate::systems::audio::AudioEvent;

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PauseState {
    Inactive,
    Active { remaining_ticks: Option<u32> },
}

impl Default for PauseState {
    fn default() -> Self {
        Self::Inactive
    }
}

impl PauseState {
    pub fn active(&self) -> bool {
        matches!(
            self,
            PauseState::Active { remaining_ticks: None }
                | PauseState::Active {
                    remaining_ticks: Some(1..=u32::MAX)
                }
        )
    }

    /// Ticks the pause state
    /// # Returns
    /// `true` if the state changed significantly (e.g. from active to inactive or vice versa)
    pub fn tick(&mut self) -> bool {
        match self {
            // Permanent states
            PauseState::Active { remaining_ticks: None } | PauseState::Inactive => false,
            // Last tick of the active state
            PauseState::Active {
                remaining_ticks: Some(1),
            } => {
                *self = PauseState::Inactive;
                true
            }
            // Active state with remaining ticks
            PauseState::Active {
                remaining_ticks: Some(ticks),
            } => {
                *self = PauseState::Active {
                    remaining_ticks: Some(*ticks - 1),
                };
                false
            }
        }
    }
}

pub fn handle_pause_command(
    mut events: EventReader<GameEvent>,
    mut pause_state: ResMut<PauseState>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for event in events.read() {
        match event {
            GameEvent::Command(GameCommand::TogglePause) => {
                *pause_state = match *pause_state {
                    PauseState::Active { .. } => {
                        info!("Game resumed");
                        audio_events.write(AudioEvent::Resume);
                        PauseState::Inactive
                    }
                    PauseState::Inactive => {
                        info!("Game paused");
                        audio_events.write(AudioEvent::Pause);
                        PauseState::Active { remaining_ticks: None }
                    }
                }
            }
            GameEvent::Command(GameCommand::SingleTick) => {
                // Single tick should not function while the game is playing
                if matches!(*pause_state, PauseState::Active { remaining_ticks: None }) {
                    return;
                }

                *pause_state = PauseState::Active {
                    remaining_ticks: Some(1),
                };
                audio_events.write(AudioEvent::Resume);
            }
            _ => {}
        }
    }
}

pub fn manage_pause_state_system(mut pause_state: ResMut<PauseState>, mut audio_events: EventWriter<AudioEvent>) {
    let changed = pause_state.tick();

    // If the pause state changed, send the appropriate audio event
    if changed {
        // Since the pause state can never go from inactive to active, the only way to get here is if the game is now paused...
        audio_events.write(AudioEvent::Pause);
    }
}
