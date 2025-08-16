use bevy_ecs::{
    event::{EventReader, EventWriter},
    prelude::ResMut,
    query::With,
    system::Query,
};

use crate::{
    error::GameError,
    events::{GameCommand, GameEvent},
    systems::components::{AudioState, GlobalState, PlayerControlled},
    systems::debug::DebugState,
    systems::movement::Movable,
};

// Handles player input and control
pub fn player_system(
    mut events: EventReader<GameEvent>,
    mut state: ResMut<GlobalState>,
    mut debug_state: ResMut<DebugState>,
    mut audio_state: ResMut<AudioState>,
    mut players: Query<&mut Movable, With<PlayerControlled>>,
    mut errors: EventWriter<GameError>,
) {
    // Get the player's movable component (ensuring there is only one player)
    let mut movable = match players.single_mut() {
        Ok(movable) => movable,
        Err(e) => {
            errors.write(GameError::InvalidState(format!(
                "No/multiple entities queried for player system: {}",
                e
            )));
            return;
        }
    };

    // Handle events
    for event in events.read() {
        if let GameEvent::Command(command) = event {
            match command {
                GameCommand::MovePlayer(direction) => {
                    movable.requested_direction = Some(*direction);
                }
                GameCommand::Exit => {
                    state.exit = true;
                }
                GameCommand::ToggleDebug => {
                    *debug_state = debug_state.next();
                }
                GameCommand::MuteAudio => {
                    audio_state.muted = !audio_state.muted;
                    tracing::info!("Audio {}", if audio_state.muted { "muted" } else { "unmuted" });
                }
                _ => {}
            }
        }
    }
}
